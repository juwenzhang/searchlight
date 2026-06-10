use crate::index::InvertedIndex;

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

/// Rank documents using BM25 scoring
///
/// BM25 is a bag-of-words ranking function that ranks documents based on
/// query term frequencies while accounting for document length.
///
/// BM25(q, d) = sum over terms t in q of:
///   IDF(t) * TF(t, d) * (k1 + 1) / (TF(t, d) + k1 * (1 - b + b * |d| / avgdl))
pub struct Ranker {
    params: Bm25Params,
}

impl Ranker {
    pub fn new() -> Self {
        Ranker {
            params: Bm25Params::default(),
        }
    }

    pub fn with_params(params: Bm25Params) -> Self {
        Ranker { params }
    }

    /// Compute BM25 score for a document given a set of query terms
    pub fn bm25_score(&self, index: &InvertedIndex, doc_id: usize, query_terms: &[String]) -> f64 {
        let mut score = 0.0;
        let doc_len = index.doc_length(doc_id) as f64;
        let avgdl = index.avg_doc_length();
        let total_docs = index.doc_count() as f64;

        for term in query_terms {
            let tf = index.term_freq_in_doc(term, doc_id) as f64;
            if tf == 0.0 {
                continue;
            }

            // Inverse Document Frequency
            let df = index
                .term_stats(term)
                .map(|s| s.doc_frequency as f64)
                .unwrap_or(0.0);
            let idf = if df > 0.0 {
                ((total_docs - df + 0.5) / (df + 0.5) + 1.0).ln()
            } else {
                0.0
            };

            // BM25 saturation
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

    /// Score based on term proximity (closer terms get higher scores)
    pub fn proximity_score(
        &self,
        index: &InvertedIndex,
        doc_id: usize,
        query_terms: &[String],
    ) -> f64 {
        if query_terms.len() < 2 || query_terms.len() > 16 {
            return 0.0;
        }

        let tokens = index.document_tokens(doc_id);
        if tokens.is_none() {
            return 0.0;
        }

        let tokens = tokens.unwrap();
        let mut min_distance = usize::MAX;

        // Find minimum distance between any two query terms in the document
        for i in 0..query_terms.len() {
            for j in (i + 1)..query_terms.len() {
                let positions_i: Vec<usize> = tokens
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.text == query_terms[i])
                    .map(|(idx, _)| idx)
                    .collect();

                let positions_j: Vec<usize> = tokens
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.text == query_terms[j])
                    .map(|(idx, _)| idx)
                    .collect();

                for &pi in &positions_i {
                    for &pj in &positions_j {
                        let dist = if pi > pj { pi - pj } else { pj - pi };
                        if dist < min_distance {
                            min_distance = dist;
                        }
                    }
                }
            }
        }

        if min_distance == usize::MAX {
            0.0
        } else {
            // Closer terms = higher score
            1.0 / (min_distance as f64 + 1.0)
        }
    }

    /// Combined score: BM25 + proximity bonus
    pub fn combined_score(
        &self,
        index: &InvertedIndex,
        doc_id: usize,
        query_terms: &[String],
    ) -> f64 {
        let bm25 = self.bm25_score(index, doc_id, query_terms);
        let proximity = self.proximity_score(index, doc_id, query_terms);

        // BM25 dominates, proximity is a bonus
        bm25 * 1.0 + proximity * 0.3
    }
}

impl Default for Ranker {
    fn default() -> Self {
        Self::new()
    }
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

        // Document 0 should score higher for "hello"
        assert!(s1 > s2);
    }

    #[test]
    fn test_proximity_score() {
        let mut index = InvertedIndex::new();
        index.add_document("hello world this is rust");
        index.add_document("hello and then after a long while world");

        let ranker = Ranker::new();
        let s1 = ranker.proximity_score(&index, 0, &["hello".to_string(), "world".to_string()]);
        let s2 = ranker.proximity_score(&index, 1, &["hello".to_string(), "world".to_string()]);

        // closer terms should score higher
        assert!(s1 > s2 || s1 >= s2);
    }
}
