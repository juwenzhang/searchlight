use std::collections::HashSet;

use crate::cache::{fuzzy_cache_key, SearchCache};
use crate::fuzzy;
use crate::index::InvertedIndex;
use crate::pinyin::PinyinIndex;
use crate::query::{ParsedQuery, QueryOp};
use crate::SearchOptions;

/// Limits applied during query evaluation.
#[derive(Debug, Clone, Copy)]
pub struct ExecutorLimits {
    pub max_candidates: usize,
    pub max_fuzzy_terms_scan: usize,
    pub max_fuzzy_results_per_term: usize,
}

impl Default for ExecutorLimits {
    fn default() -> Self {
        ExecutorLimits {
            max_candidates: 10_000,
            max_fuzzy_terms_scan: 5_000,
            max_fuzzy_results_per_term: 32,
        }
    }
}

/// Context passed to the query executor.
pub struct ExecutorContext<'a> {
    pub index: &'a InvertedIndex,
    pub pinyin_index: &'a PinyinIndex,
    pub cache: &'a SearchCache,
    pub index_generation: u64,
    pub options: &'a SearchOptions,
    pub parsed: &'a ParsedQuery,
    pub query: &'a str,
    pub limits: ExecutorLimits,
}

/// Result of evaluating a query AST against the index.
#[derive(Debug, Default)]
pub struct QueryEvaluation {
    pub candidates: HashSet<usize>,
    pub fuzzy_terms: HashSet<String>,
}

/// Evaluates `QueryOp` AST with correct boolean semantics.
pub struct QueryExecutor;

impl QueryExecutor {
    pub fn evaluate(ctx: &ExecutorContext<'_>, op: &QueryOp) -> QueryEvaluation {
        let mut evaluation = QueryEvaluation::default();
        evaluation.candidates = Self::evaluate_op(ctx, op, &mut evaluation.fuzzy_terms);
        evaluation
    }

    /// Whether a single document satisfies the query AST.
    pub fn document_matches(ctx: &ExecutorContext<'_>, op: &QueryOp, doc_id: usize) -> bool {
        Self::evaluate_op(ctx, op, &mut HashSet::new()).contains(&doc_id)
    }

    /// Collect document IDs that match a pinyin query string.
    pub fn pinyin_candidates(ctx: &ExecutorContext<'_>, query: &str) -> HashSet<usize> {
        let mut docs = HashSet::new();
        let pinyin_results = ctx.pinyin_index.search_by_pinyin_detailed(query);
        for chinese_term in pinyin_results.keys() {
            if docs.len() >= ctx.limits.max_candidates {
                break;
            }
            extend_docs_from_term(ctx, chinese_term, &mut docs);
        }
        docs
    }

    fn evaluate_op(
        ctx: &ExecutorContext<'_>,
        op: &QueryOp,
        fuzzy_terms: &mut HashSet<String>,
    ) -> HashSet<usize> {
        match op {
            QueryOp::Term(term) => {
                let distance = if ctx.options.fuzzy {
                    ctx.options.max_edit_distance
                } else {
                    0
                };
                docs_for_term(ctx, term, distance, fuzzy_terms)
            }
            QueryOp::Phrase(terms) => docs_for_phrase(ctx, terms, fuzzy_terms),
            QueryOp::Fuzzy(term, distance) => {
                docs_for_term(ctx, term, (*distance).min(3), fuzzy_terms)
            }
            QueryOp::Prefix(prefix) => docs_for_prefix(ctx, prefix),
            QueryOp::CharMatch(chars) => {
                let sets: Vec<HashSet<usize>> = chars
                    .iter()
                    .map(|c| docs_for_term(ctx, &c.to_string(), 0, fuzzy_terms))
                    .collect();
                intersect_sets(&sets, ctx.limits.max_candidates)
            }
            QueryOp::And(children) => {
                let mut include_sets = Vec::new();
                let mut exclude_sets = Vec::new();

                for child in children {
                    match child {
                        QueryOp::Not(inner) => {
                            exclude_sets.push(Self::evaluate_op(ctx, inner, fuzzy_terms));
                        }
                        _ => include_sets.push(Self::evaluate_op(ctx, child, fuzzy_terms)),
                    }
                }

                let mut result = if include_sets.is_empty() {
                    HashSet::new()
                } else {
                    intersect_sets(&include_sets, ctx.limits.max_candidates)
                };

                for exclude in exclude_sets {
                    result = result.difference(&exclude).copied().collect::<HashSet<_>>();
                }

                truncate_set(&mut result, ctx.limits.max_candidates);
                result
            }
            QueryOp::Or(children) => {
                let sets: Vec<HashSet<usize>> = children
                    .iter()
                    .filter(|c| !matches!(c, QueryOp::Not(_)))
                    .map(|c| Self::evaluate_op(ctx, c, fuzzy_terms))
                    .collect();
                union_sets(&sets, ctx.limits.max_candidates)
            }
            QueryOp::Not(_) => HashSet::new(),
        }
    }
}

fn docs_for_term(
    ctx: &ExecutorContext<'_>,
    term: &str,
    max_distance: usize,
    fuzzy_terms: &mut HashSet<String>,
) -> HashSet<usize> {
    let mut docs = HashSet::new();
    extend_docs_from_term(ctx, term, &mut docs);

    if max_distance > 0 {
        for expanded in fuzzy_expansion(ctx, term, max_distance) {
            fuzzy_terms.insert(expanded.clone());
            extend_docs_from_term(ctx, &expanded, &mut docs);
            if docs.len() >= ctx.limits.max_candidates {
                break;
            }
        }
    }

    truncate_set(&mut docs, ctx.limits.max_candidates);
    docs
}

fn docs_for_phrase(
    ctx: &ExecutorContext<'_>,
    terms: &[String],
    fuzzy_terms: &mut HashSet<String>,
) -> HashSet<usize> {
    if terms.is_empty() {
        return HashSet::new();
    }

    let sets: Vec<HashSet<usize>> = terms
        .iter()
        .map(|t| docs_for_term(ctx, t, 0, fuzzy_terms))
        .collect();
    let mut docs = intersect_sets(&sets, ctx.limits.max_candidates);

    docs.retain(|&doc_id| phrase_matches(ctx.index, doc_id, terms));
    truncate_set(&mut docs, ctx.limits.max_candidates);
    docs
}

fn docs_for_prefix(ctx: &ExecutorContext<'_>, prefix: &str) -> HashSet<usize> {
    let mut docs = HashSet::new();
    for term in ctx.index.terms_with_prefix(prefix) {
        extend_docs_from_term(ctx, &term, &mut docs);
        if docs.len() >= ctx.limits.max_candidates {
            break;
        }
    }
    docs
}

fn extend_docs_from_term(ctx: &ExecutorContext<'_>, term: &str, docs: &mut HashSet<usize>) {
    if docs.len() >= ctx.limits.max_candidates {
        return;
    }
    if let Some(postings) = ctx.index.posting_list(term) {
        for posting in postings {
            if docs.len() >= ctx.limits.max_candidates {
                break;
            }
            docs.insert(posting.doc_id);
        }
    }
}

fn fuzzy_expansion(ctx: &ExecutorContext<'_>, term: &str, max_distance: usize) -> Vec<String> {
    let cache_key = fuzzy_cache_key(ctx.index_generation, term, max_distance);

    if ctx.options.enable_cache {
        if let Some(cached) = ctx.cache.get_fuzzy_expansion(cache_key) {
            return cached;
        }
    }

    let all_terms = ctx
        .index
        .terms_with_prefix_limited("", ctx.limits.max_fuzzy_terms_scan);
    let matches = fuzzy::fuzzy_match_limited(
        term,
        &all_terms,
        max_distance,
        ctx.limits.max_fuzzy_results_per_term,
    );
    let terms: Vec<String> = matches.into_iter().map(|m| m.term).collect();

    if ctx.options.enable_cache {
        ctx.cache.put_fuzzy_expansion(cache_key, terms.clone());
    }

    terms
}

pub(crate) fn phrase_matches_public(
    index: &InvertedIndex,
    doc_id: usize,
    phrase: &[String],
) -> bool {
    phrase_matches(index, doc_id, phrase)
}

pub(crate) fn fuzzy_expansion_public(
    ctx: &ExecutorContext<'_>,
    term: &str,
    max_distance: usize,
) -> Vec<String> {
    fuzzy_expansion(ctx, term, max_distance)
}

fn phrase_matches(index: &InvertedIndex, doc_id: usize, phrase: &[String]) -> bool {
    if phrase.len() < 2 {
        return phrase
            .first()
            .map(|t| index.term_freq_in_doc(t, doc_id) > 0)
            .unwrap_or(false);
    }

    let Some(tokens) = index.document_tokens(doc_id) else {
        return false;
    };

    let token_texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
    let phrase_refs: Vec<&str> = phrase.iter().map(String::as_str).collect();
    let window = phrase.len();

    token_texts
        .windows(window)
        .any(|w| w == phrase_refs.as_slice())
}

fn intersect_sets(sets: &[HashSet<usize>], max: usize) -> HashSet<usize> {
    if sets.is_empty() {
        return HashSet::new();
    }

    let mut result = sets[0].clone();
    for set in sets.iter().skip(1) {
        result = result.intersection(set).copied().collect();
        if result.is_empty() {
            return result;
        }
    }

    truncate_set(&mut result, max);
    result
}

fn union_sets(sets: &[HashSet<usize>], max: usize) -> HashSet<usize> {
    let mut result = HashSet::new();
    for set in sets {
        for &doc_id in set {
            if result.len() >= max {
                return result;
            }
            result.insert(doc_id);
        }
    }
    result
}

fn truncate_set(set: &mut HashSet<usize>, max: usize) {
    if set.len() <= max {
        return;
    }
    let keep: HashSet<usize> = set.iter().take(max).copied().collect();
    *set = keep;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pinyin::PinyinIndex;
    use crate::query::QueryParser;
    use crate::SearchCache;

    struct Fixture {
        index: InvertedIndex,
        pinyin_index: PinyinIndex,
        cache: SearchCache,
    }

    impl Fixture {
        fn index(&mut self, text: &str) {
            self.index.add_document(text);
        }

        fn eval(&self, query: &str, options: SearchOptions) -> HashSet<usize> {
            let parser = QueryParser::new();
            let parsed = parser.parse(query);
            let ctx = ExecutorContext {
                index: &self.index,
                pinyin_index: &self.pinyin_index,
                cache: &self.cache,
                index_generation: 0,
                options: &options,
                parsed: &parsed,
                query,
                limits: ExecutorLimits::default(),
            };
            QueryExecutor::evaluate(&ctx, &parsed.root).candidates
        }
    }

    #[test]
    fn test_and_intersection() {
        let mut fx = Fixture {
            index: InvertedIndex::new(),
            pinyin_index: PinyinIndex::new(),
            cache: SearchCache::new(),
        };
        fx.index("Rust and Python are great");
        fx.index("Python is easy");
        fx.index("Rust is memory safe");

        assert_eq!(
            fx.eval("Rust AND Python", SearchOptions::default()).len(),
            1
        );
    }

    #[test]
    fn test_or_union() {
        let mut fx = Fixture {
            index: InvertedIndex::new(),
            pinyin_index: PinyinIndex::new(),
            cache: SearchCache::new(),
        };
        fx.index("only rust here");
        fx.index("only python here");

        assert_eq!(fx.eval("rust OR python", SearchOptions::default()).len(), 2);
    }

    #[test]
    fn test_not_exclusion() {
        let mut fx = Fixture {
            index: InvertedIndex::new(),
            pinyin_index: PinyinIndex::new(),
            cache: SearchCache::new(),
        };
        fx.index("rust programming");
        fx.index("python programming");

        assert_eq!(
            fx.eval("programming -python", SearchOptions::default())
                .len(),
            1
        );
    }

    #[test]
    fn test_phrase_consecutive() {
        let mut fx = Fixture {
            index: InvertedIndex::new(),
            pinyin_index: PinyinIndex::new(),
            cache: SearchCache::new(),
        };
        fx.index("I love rust programming");
        fx.index("rust is great for programming");

        assert_eq!(
            fx.eval("\"rust programming\"", SearchOptions::default())
                .len(),
            1
        );
    }

    #[test]
    fn test_implicit_and_multi_term() {
        let mut fx = Fixture {
            index: InvertedIndex::new(),
            pinyin_index: PinyinIndex::new(),
            cache: SearchCache::new(),
        };
        fx.index("hello world");
        fx.index("hello there");
        fx.index("world alone");

        assert_eq!(fx.eval("hello world", SearchOptions::default()).len(), 1);
    }
}
