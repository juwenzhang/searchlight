use crate::index::InvertedIndex;

/// A highlighted snippet from a document
#[derive(Debug, Clone)]
pub struct Snippet {
    pub doc_id: usize,
    pub highlighted: String,
    pub start: usize,
    pub end: usize,
    pub original: String,
}

#[derive(Debug, Clone)]
pub struct HighlighterConfig {
    pub context_before: usize,
    pub context_after: usize,
    pub max_snippet_len: usize,
    pub highlight_open: String,
    pub highlight_close: String,
}

impl Default for HighlighterConfig {
    fn default() -> Self {
        HighlighterConfig {
            context_before: 50,
            context_after: 50,
            max_snippet_len: 200,
            highlight_open: "<em>".to_string(),
            highlight_close: "</em>".to_string(),
        }
    }
}

pub struct Highlighter {
    config: HighlighterConfig,
}

impl Highlighter {
    pub fn new() -> Self {
        Highlighter {
            config: HighlighterConfig::default(),
        }
    }

    pub fn with_config(config: HighlighterConfig) -> Self {
        Highlighter { config }
    }

    /// Generate a highlighted snippet for a document.
    /// Spans from the inverted index use BYTE offsets — we work entirely in byte space.
    pub fn highlight(
        &self,
        index: &InvertedIndex,
        doc_id: usize,
        query_terms: &[String],
    ) -> Option<Snippet> {
        let doc_text = index.document(doc_id)?;

        // Collect byte-offset match spans
        let mut spans: Vec<(usize, usize)> = Vec::new();
        for term in query_terms {
            if let Some(postings) = index.posting_list(term) {
                for pos in postings {
                    if pos.doc_id == doc_id {
                        spans.push((pos.start, pos.end));
                    }
                }
            }
        }

        if spans.is_empty() {
            return None;
        }

        spans.sort_by_key(|s| s.0);
        let merged = merge_spans(&spans);

        // Build the highlighted full text
        let mut result = String::new();
        let mut cursor: usize = 0; // byte cursor in doc_text

        for &(m_start, m_end) in &merged {
            // Text before this match
            if cursor < m_start {
                result.push_str(&doc_text[cursor..m_start]);
            }
            // Highlighted match
            result.push_str(&self.config.highlight_open);
            result.push_str(&doc_text[m_start..m_end]);
            result.push_str(&self.config.highlight_close);
            cursor = m_end;
        }
        // Remaining text after the last match
        if cursor < doc_text.len() {
            result.push_str(&doc_text[cursor..]);
        }

        Some(Snippet {
            doc_id,
            highlighted: result,
            start: merged.first().map(|s| s.0).unwrap_or(0),
            end: merged.last().map(|s| s.1).unwrap_or(doc_text.len()),
            original: doc_text.to_string(),
        })
    }

    /// Generate snippets for multiple documents
    pub fn highlight_many(
        &self,
        index: &InvertedIndex,
        doc_ids: &[usize],
        query_terms: &[String],
    ) -> Vec<Snippet> {
        doc_ids
            .iter()
            .filter_map(|&id| self.highlight(index, id, query_terms))
            .collect()
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Merge overlapping byte-offset spans
fn merge_spans(spans: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if spans.is_empty() {
        return vec![];
    }
    let mut merged: Vec<(usize, usize)> = vec![spans[0]];
    for &(start, end) in &spans[1..] {
        let last = merged.last_mut().unwrap();
        if start < last.1 {
            last.1 = last.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_basic() {
        let mut index = InvertedIndex::new();
        index.add_document("Rust是一门现代系统编程语言，安全且高效");

        let highlighter = Highlighter::new();
        let snippet = highlighter
            .highlight(&index, 0, &["rust".to_string(), "编程语言".to_string()])
            .unwrap();

        assert!(snippet.highlighted.contains("<em>"));
        assert!(snippet.highlighted.contains("Rust"));
        assert!(snippet.highlighted.contains("编程语言"));
    }

    #[test]
    fn test_merge_spans() {
        let spans = vec![(0, 5), (3, 8), (10, 15)];
        let merged = merge_spans(&spans);
        assert_eq!(merged, vec![(0, 8), (10, 15)]);
    }

    #[test]
    fn test_highlight_chinese() {
        let mut index = InvertedIndex::new();
        index.add_document("我爱北京天安门，天安门上太阳升");

        let highlighter = Highlighter::new();
        let snippet = highlighter
            .highlight(&index, 0, &["北京".to_string(), "天安门".to_string()])
            .unwrap();

        assert!(snippet.highlighted.contains("<em>北京</em>"));
    }
}
