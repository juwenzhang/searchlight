use std::cell::RefCell;
use std::num::NonZeroUsize;

use lru::LruCache;

use crate::index::InvertedIndex;

const IDF_CACHE_CAPACITY: usize = 4096;

/// A single search hit for ranking purposes
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Hit {
    pub doc_id: usize,
    pub term: String,
    pub term_freq: usize,
}

/// BM25 ranking parameters
#[derive(Debug, Clone)]
pub struct Bm25Params {
    /// k1: term frequency saturation parameter (default 1.2)
    pub k1: f64,
    /// b: length normalization parameter (default 0.75)
    pub b: f64,
}

impl Default for Bm25Params {
    fn default() -> Self {
        Bm25Params { k1: 1.2, b: 0.75 }
    }
}

/// Input for the combined scoring pipeline.
pub struct ScoringInput<'a> {
    pub query_terms: &'a [String],
    pub phrases: &'a [Vec<String>],
    pub pinyin_query: Option<&'a str>,
    pub use_pinyin: bool,
}

/// Per-component score breakdown (useful for debugging and agent explain).
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreBreakdown {
    pub bm25: f64,
    pub proximity: f64,
    pub coverage: f64,
    pub phrase: f64,
    pub pinyin: f64,
    pub total: f64,
}

/// Rank documents using BM25 scoring with LRU-cached IDF and composite bonuses.
pub struct Ranker {
    params: Bm25Params,
    idf_cache: RefCell<LruCache<String, f64>>,
    cached_total_docs: RefCell<usize>,
}

impl Ranker {
    pub fn new() -> Self {
        Ranker {
            params: Bm25Params::default(),
            idf_cache: RefCell::new(LruCache::new(
                NonZeroUsize::new(IDF_CACHE_CAPACITY).unwrap(),
            )),
            cached_total_docs: RefCell::new(0),
        }
    }

    pub fn with_params(params: Bm25Params) -> Self {
        Ranker {
            params,
            idf_cache: RefCell::new(LruCache::new(
                NonZeroUsize::new(IDF_CACHE_CAPACITY).unwrap(),
            )),
            cached_total_docs: RefCell::new(0),
        }
    }

    /// Clear cached IDF values (call after index mutations).
    pub fn invalidate_caches(&self) {
        self.idf_cache.borrow_mut().clear();
        *self.cached_total_docs.borrow_mut() = 0;
    }

    fn idf(&self, index: &InvertedIndex, term: &str) -> f64 {
        let total_docs = index.doc_count();
        if total_docs == 0 {
            return 0.0;
        }

        if *self.cached_total_docs.borrow() != total_docs {
            self.idf_cache.borrow_mut().clear();
            *self.cached_total_docs.borrow_mut() = total_docs;
        }

        if let Some(&cached) = self.idf_cache.borrow_mut().get(term) {
            return cached;
        }

        let df = index
            .term_stats(term)
            .map(|s| s.doc_frequency as f64)
            .unwrap_or(0.0);
        let total = total_docs as f64;
        let idf = if df > 0.0 {
            ((total - df + 0.5) / (df + 0.5) + 1.0).ln()
        } else {
            0.0
        };

        self.idf_cache.borrow_mut().put(term.to_string(), idf);
        idf
    }

    /// Compute BM25 score for a document given a set of query terms.
    pub fn bm25_score(&self, index: &InvertedIndex, doc_id: usize, query_terms: &[String]) -> f64 {
        let doc_len = index.doc_length(doc_id) as f64;
        let avgdl = index.avg_doc_length();
        if avgdl == 0.0 {
            return 0.0;
        }

        let mut score = 0.0;
        for term in query_terms {
            let tf = index.term_freq_in_doc(term, doc_id) as f64;
            if tf == 0.0 {
                continue;
            }

            let idf = self.idf(index, term);
            let numerator = tf * (self.params.k1 + 1.0);
            let denominator =
                tf + self.params.k1 * (1.0 - self.params.b + self.params.b * doc_len / avgdl);
            score += idf * numerator / denominator;
        }

        score
    }

    /// Simple TF-IDF score (alternative to BM25)
    pub fn tfidf_score(&self, index: &InvertedIndex, doc_id: usize, query_terms: &[String]) -> f64 {
        let mut score = 0.0;
        let total_docs = index.doc_count() as f64;
        if total_docs == 0.0 {
            return 0.0;
        }

        for term in query_terms {
            let tf = index.term_freq_in_doc(term, doc_id) as f64;
            if tf == 0.0 {
                continue;
            }

            let df = index
                .term_stats(term)
                .map(|s| s.doc_frequency as f64)
                .unwrap_or(0.0);
            let idf = if df > 0.0 {
                (total_docs / df).ln()
            } else {
                0.0
            };

            score += (1.0 + tf.ln()) * idf;
        }

        score
    }

    /// Term proximity: average inverse distance across all query-term pairs.
    pub fn proximity_score(
        &self,
        index: &InvertedIndex,
        doc_id: usize,
        query_terms: &[String],
    ) -> f64 {
        if query_terms.len() < 2 || query_terms.len() > 16 {
            return 0.0;
        }

        let Some(tokens) = index.document_tokens(doc_id) else {
            return 0.0;
        };

        let mut pair_scores = 0.0;
        let mut pair_count = 0usize;

        for i in 0..query_terms.len() {
            for j in (i + 1)..query_terms.len() {
                let mut min_distance = usize::MAX;

                for (idx, token) in tokens.iter().enumerate() {
                    if token.text != query_terms[i] {
                        continue;
                    }
                    for (jdx, other) in tokens.iter().enumerate() {
                        if other.text != query_terms[j] {
                            continue;
                        }
                        let dist = idx.abs_diff(jdx);
                        if dist < min_distance {
                            min_distance = dist;
                        }
                    }
                }

                if min_distance != usize::MAX {
                    pair_scores += 1.0 / (min_distance as f64 + 1.0);
                    pair_count += 1;
                }
            }
        }

        if pair_count == 0 {
            0.0
        } else {
            pair_scores / pair_count as f64
        }
    }

    /// Bonus when query terms appear in order as consecutive tokens (phrase match).
    pub fn phrase_score(
        &self,
        index: &InvertedIndex,
        doc_id: usize,
        phrases: &[Vec<String>],
    ) -> f64 {
        if phrases.is_empty() {
            return 0.0;
        }

        let Some(tokens) = index.document_tokens(doc_id) else {
            return 0.0;
        };

        let token_texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        let mut bonus = 0.0;

        for phrase in phrases {
            if phrase.len() < 2 {
                continue;
            }
            if phrase_windows(&token_texts, phrase) > 0 {
                bonus += 1.5 * phrase.len() as f64;
            }
        }

        bonus
    }

    /// Fraction of query terms that appear in the document, scaled by BM25.
    pub fn coverage_bonus(
        &self,
        index: &InvertedIndex,
        doc_id: usize,
        query_terms: &[String],
        bm25: f64,
    ) -> f64 {
        if query_terms.is_empty() || bm25 <= 0.0 {
            return 0.0;
        }

        let matched = query_terms
            .iter()
            .filter(|t| index.term_freq_in_doc(t, doc_id) > 0)
            .count();

        let ratio = matched as f64 / query_terms.len() as f64;
        ratio * bm25 * 0.2
    }

    /// Full composite score with breakdown.
    pub fn score_document(
        &self,
        index: &InvertedIndex,
        doc_id: usize,
        input: &ScoringInput<'_>,
        pinyin_matches: bool,
    ) -> ScoreBreakdown {
        let bm25 = self.bm25_score(index, doc_id, input.query_terms);
        let proximity = self.proximity_score(index, doc_id, input.query_terms);
        let coverage = self.coverage_bonus(index, doc_id, input.query_terms, bm25);
        let phrase = self.phrase_score(index, doc_id, input.phrases);

        let proximity_weight = 0.45 * (1.0 + bm25.ln_1p().min(2.0));
        let pinyin = if input.use_pinyin && pinyin_matches {
            2.0 + bm25 * 0.1
        } else {
            0.0
        };

        let total = bm25 + proximity * proximity_weight + coverage + phrase + pinyin;

        ScoreBreakdown {
            bm25,
            proximity: proximity * proximity_weight,
            coverage,
            phrase,
            pinyin,
            total,
        }
    }

    /// Combined score: BM25 + proximity + coverage + phrase bonuses.
    pub fn combined_score(
        &self,
        index: &InvertedIndex,
        doc_id: usize,
        query_terms: &[String],
    ) -> f64 {
        let input = ScoringInput {
            query_terms,
            phrases: &[],
            pinyin_query: None,
            use_pinyin: false,
        };
        self.score_document(index, doc_id, &input, false).total
    }
}

impl Default for Ranker {
    fn default() -> Self {
        Self::new()
    }
}

fn phrase_windows(token_texts: &[&str], phrase: &[String]) -> usize {
    if phrase.is_empty() || token_texts.len() < phrase.len() {
        return 0;
    }

    let phrase_refs: Vec<&str> = phrase.iter().map(String::as_str).collect();
    let window = phrase.len();
    let mut count = 0;

    for start in 0..=token_texts.len().saturating_sub(window) {
        if token_texts[start..start + window] == phrase_refs[..] {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_empty() {
        let index = InvertedIndex::new();
        let ranker = Ranker::new();
        let score = ranker.bm25_score(&index, 0, &["test".to_string()]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_bm25_matching() {
        let mut index = InvertedIndex::new();
        index.add_document("hello world hello");
        index.add_document("foo bar baz");

        let ranker = Ranker::new();
        let s1 = ranker.bm25_score(&index, 0, &["hello".to_string()]);
        let s2 = ranker.bm25_score(&index, 1, &["hello".to_string()]);

        assert!(s1 > s2);
    }

    #[test]
    fn test_idf_cache_reuse() {
        let mut index = InvertedIndex::new();
        index.add_document("hello world");

        let ranker = Ranker::new();
        let _ = ranker.bm25_score(&index, 0, &["hello".to_string()]);
        let _ = ranker.bm25_score(&index, 0, &["hello".to_string()]);
        assert!(!ranker.idf_cache.borrow().is_empty());
    }

    #[test]
    fn test_proximity_score() {
        let mut index = InvertedIndex::new();
        index.add_document("hello world this is rust");
        index.add_document("hello and then after a long while world");

        let ranker = Ranker::new();
        let s1 = ranker.proximity_score(&index, 0, &["hello".to_string(), "world".to_string()]);
        let s2 = ranker.proximity_score(&index, 1, &["hello".to_string(), "world".to_string()]);

        assert!(s1 > s2);
    }

    #[test]
    fn test_phrase_score() {
        let mut index = InvertedIndex::new();
        index.add_document("I love rust programming");
        index.add_document("rust is great for programming");

        let ranker = Ranker::new();
        let phrase = vec!["rust".to_string(), "programming".to_string()];
        let s1 = ranker.phrase_score(&index, 0, &[phrase.clone()]);
        let s2 = ranker.phrase_score(&index, 1, &[phrase]);

        assert!(s1 > s2);
    }

    #[test]
    fn test_coverage_bonus() {
        let mut index = InvertedIndex::new();
        index.add_document("rust python go");
        index.add_document("rust only");

        let ranker = Ranker::new();
        let terms = vec!["rust".to_string(), "python".to_string(), "go".to_string()];
        let bm25_full = ranker.bm25_score(&index, 0, &terms);
        let bm25_partial = ranker.bm25_score(&index, 1, &terms);

        let cov_full = ranker.coverage_bonus(&index, 0, &terms, bm25_full);
        let cov_partial = ranker.coverage_bonus(&index, 1, &terms, bm25_partial);

        assert!(cov_full > cov_partial);
    }
}
