use crate::executor::ExecutorContext;
use crate::query::QueryOp;

/// Why a document matched the query (retrieval-level explain for agents).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchReason {
    /// Machine-readable reason code, e.g. `term`, `fuzzy`, `phrase`, `and`, `or`, `not`, `pinyin`
    pub code: String,
    /// Human-readable explanation
    pub message: String,
    /// Related query/index terms
    pub terms: Vec<String>,
}

impl MatchReason {
    pub fn new(code: impl Into<String>, message: impl Into<String>, terms: Vec<String>) -> Self {
        MatchReason {
            code: code.into(),
            message: message.into(),
            terms,
        }
    }

    pub fn term(term: &str) -> Self {
        Self::new(
            "term",
            format!("document contains term '{term}'"),
            vec![term.to_string()],
        )
    }

    pub fn fuzzy(query: &str, matched: &str) -> Self {
        Self::new(
            "fuzzy",
            format!("fuzzy match: '{query}' matched index term '{matched}'"),
            vec![query.to_string(), matched.to_string()],
        )
    }

    pub fn phrase(terms: &[String]) -> Self {
        let joined = terms.join(" ");
        Self::new(
            "phrase",
            format!("document contains consecutive phrase '{joined}'"),
            terms.to_vec(),
        )
    }

    pub fn prefix(prefix: &str, matched_terms: &[String]) -> Self {
        Self::new(
            "prefix",
            format!("document matches prefix '{prefix}'"),
            matched_terms.to_vec(),
        )
    }

    pub fn char_match(chars: &[char]) -> Self {
        let terms: Vec<String> = chars.iter().map(|c| c.to_string()).collect();
        Self::new(
            "char",
            format!("document contains characters: {}", terms.join(", ")),
            terms,
        )
    }

    pub fn and_clause(child_count: usize) -> Self {
        Self::new(
            "and",
            format!("document satisfies AND of {child_count} clauses"),
            vec![],
        )
    }

    pub fn or_clause() -> Self {
        Self::new(
            "or",
            "document satisfies at least one OR branch".to_string(),
            vec![],
        )
    }

    pub fn not_excluded(terms: &[String]) -> Self {
        Self::new(
            "not",
            format!(
                "document does not match excluded terms: {}",
                terms.join(", ")
            ),
            terms.to_vec(),
        )
    }

    pub fn pinyin(query: &str) -> Self {
        Self::new(
            "pinyin",
            format!("document matched pinyin input '{query}'"),
            vec![query.to_string()],
        )
    }

    pub fn score_component(component: &str, value: f64) -> Self {
        Self::new(
            "score",
            format!("ranking bonus from {component}: {value:.4}"),
            vec![component.to_string()],
        )
    }
}

/// Collect retrieval-level match reasons for a document against a query AST.
pub fn explain_document(
    ctx: &ExecutorContext<'_>,
    op: &QueryOp,
    doc_id: usize,
) -> Vec<MatchReason> {
    if !crate::executor::QueryExecutor::document_matches(ctx, op, doc_id) {
        return vec![];
    }

    let mut reasons = Vec::new();
    collect_reasons(ctx, op, doc_id, &mut reasons);
    dedup_reasons(&mut reasons);
    reasons
}

fn collect_reasons(
    ctx: &ExecutorContext<'_>,
    op: &QueryOp,
    doc_id: usize,
    reasons: &mut Vec<MatchReason>,
) {
    match op {
        QueryOp::Term(term) => explain_term(
            ctx,
            doc_id,
            term,
            ctx.options
                .fuzzy
                .then_some(ctx.options.max_edit_distance)
                .unwrap_or(0),
            reasons,
        ),
        QueryOp::Fuzzy(term, distance) => {
            explain_term(ctx, doc_id, term, (*distance).min(3), reasons)
        }
        QueryOp::Phrase(terms) => {
            if crate::executor::phrase_matches_public(ctx.index, doc_id, terms) {
                reasons.push(MatchReason::phrase(terms));
            }
        }
        QueryOp::Prefix(prefix) => {
            let matched: Vec<String> = ctx
                .index
                .terms_with_prefix(prefix)
                .into_iter()
                .filter(|t| ctx.index.term_freq_in_doc(t, doc_id) > 0)
                .collect();
            if !matched.is_empty() {
                reasons.push(MatchReason::prefix(prefix, &matched));
            }
        }
        QueryOp::CharMatch(chars) => {
            if chars
                .iter()
                .all(|c| ctx.index.term_freq_in_doc(&c.to_string(), doc_id) > 0)
            {
                reasons.push(MatchReason::char_match(chars));
            }
        }
        QueryOp::And(children) => {
            let mut includes = Vec::new();
            let mut excludes = Vec::new();
            for child in children {
                match child {
                    QueryOp::Not(inner) => excludes.push(inner.as_ref()),
                    _ => includes.push(child),
                }
            }

            if !includes.is_empty() {
                reasons.push(MatchReason::and_clause(includes.len()));
                for child in includes {
                    collect_reasons(ctx, child, doc_id, reasons);
                }
            }

            for exclude in excludes {
                let excluded_terms = collect_op_terms(exclude);
                if !crate::executor::QueryExecutor::document_matches(ctx, exclude, doc_id) {
                    reasons.push(MatchReason::not_excluded(&excluded_terms));
                }
            }
        }
        QueryOp::Or(children) => {
            let matching: Vec<_> = children
                .iter()
                .filter(|c| !matches!(c, QueryOp::Not(_)))
                .filter(|c| crate::executor::QueryExecutor::document_matches(ctx, c, doc_id))
                .collect();

            if !matching.is_empty() {
                reasons.push(MatchReason::or_clause());
                for child in matching {
                    collect_reasons(ctx, child, doc_id, reasons);
                }
            }
        }
        QueryOp::Not(_) => {}
    }
}

fn explain_term(
    ctx: &ExecutorContext<'_>,
    doc_id: usize,
    term: &str,
    max_distance: usize,
    reasons: &mut Vec<MatchReason>,
) {
    if ctx.index.term_freq_in_doc(term, doc_id) > 0 {
        reasons.push(MatchReason::term(term));
        return;
    }

    if max_distance == 0 {
        return;
    }

    for expanded in crate::executor::fuzzy_expansion_public(ctx, term, max_distance) {
        if ctx.index.term_freq_in_doc(&expanded, doc_id) > 0 {
            reasons.push(MatchReason::fuzzy(term, &expanded));
        }
    }
}

fn collect_op_terms(op: &QueryOp) -> Vec<String> {
    match op {
        QueryOp::Term(t) | QueryOp::Fuzzy(t, _) => vec![t.clone()],
        QueryOp::Phrase(terms) => terms.clone(),
        QueryOp::Prefix(p) => vec![format!("{p}*")],
        QueryOp::CharMatch(chars) => chars.iter().map(|c| c.to_string()).collect(),
        QueryOp::And(children) | QueryOp::Or(children) => {
            children.iter().flat_map(collect_op_terms).collect()
        }
        QueryOp::Not(inner) => collect_op_terms(inner),
    }
}

fn dedup_reasons(reasons: &mut Vec<MatchReason>) {
    let mut seen = std::collections::HashSet::new();
    reasons.retain(|reason| {
        let key = (reason.code.clone(), reason.message.clone());
        seen.insert(key)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::{ExecutorContext, ExecutorLimits};
    use crate::index::InvertedIndex;
    use crate::pinyin::PinyinIndex;
    use crate::query::QueryParser;
    use crate::SearchCache;
    use crate::SearchOptions;

    fn explain(query: &str, doc_id: usize) -> Vec<MatchReason> {
        let mut index = InvertedIndex::new();
        index.add_document("Rust and Python are great");
        index.add_document("Python only");

        let parser = QueryParser::new();
        let parsed = parser.parse(query);
        let cache = SearchCache::new();
        let options = SearchOptions::default();
        let pinyin_index = PinyinIndex::new();
        let ctx = ExecutorContext {
            index: &index,
            pinyin_index: &pinyin_index,
            cache: &cache,
            index_generation: 0,
            options: &options,
            parsed: &parsed,
            query,
            limits: ExecutorLimits::default(),
        };
        explain_document(&ctx, &parsed.root, doc_id)
    }

    #[test]
    fn test_explain_and() {
        let reasons = explain("Rust AND Python", 0);
        assert!(reasons.iter().any(|r| r.code == "and"));
        assert!(reasons.iter().any(|r| r.code == "term"));
    }

    #[test]
    fn test_explain_phrase() {
        let mut index = InvertedIndex::new();
        index.add_document("I love rust programming");
        let parser = QueryParser::new();
        let parsed = parser.parse("\"rust programming\"");
        let cache = SearchCache::new();
        let options = SearchOptions::default();
        let pinyin_index = PinyinIndex::new();
        let ctx = ExecutorContext {
            index: &index,
            pinyin_index: &pinyin_index,
            cache: &cache,
            index_generation: 0,
            options: &options,
            parsed: &parsed,
            query: "\"rust programming\"",
            limits: ExecutorLimits::default(),
        };
        let reasons = explain_document(&ctx, &parsed.root, 0);
        assert!(reasons.iter().any(|r| r.code == "phrase"));
    }
}
