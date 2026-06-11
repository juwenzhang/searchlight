use crate::tokenizer::{self, Token, TokenKind};
use std::collections::HashMap;

/// Position information for a term in a document
#[derive(Debug, Clone)]
pub struct TermPosition {
    pub doc_id: usize,
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}

/// Statistics for a term across all documents
#[derive(Debug, Clone, Default)]
pub struct TermStats {
    /// Number of documents containing this term
    pub doc_frequency: usize,
    /// Total term frequency across all docs
    pub total_frequency: usize,
}

/// The inverted index: term -> list of (document_id, positions)
#[derive(Debug, Clone, Default)]
pub struct InvertedIndex {
    /// Term -> list of positions in documents
    posting_lists: HashMap<String, Vec<TermPosition>>,
    /// Term statistics
    term_stats: HashMap<String, TermStats>,
    /// Document ID -> original text
    documents: HashMap<usize, String>,
    /// Document ID -> parsed tokens
    doc_tokens: HashMap<usize, Vec<Token>>,
    /// Document ID -> term frequency map
    doc_term_freqs: HashMap<usize, HashMap<String, usize>>,
    /// Next document ID
    next_id: usize,
    /// Total number of documents
    doc_count: usize,
    /// Average document length (in tokens)
    avg_doc_length: f64,
    /// Total token count across all docs
    total_tokens: usize,
}

impl InvertedIndex {
    pub fn new() -> Self {
        InvertedIndex::default()
    }

    /// Add a document to the index
    pub fn add_document(&mut self, text: &str) -> usize {
        let doc_id = self.next_id;
        self.next_id += 1;

        let tokens = tokenizer::tokenize(text);
        let mut term_freq: HashMap<String, usize> = HashMap::new();

        // Build positions and frequencies
        for token in &tokens {
            self.posting_lists
                .entry(token.text.clone())
                .or_default()
                .push(TermPosition {
                    doc_id,
                    start: token.start,
                    end: token.end,
                    kind: token.kind,
                });

            *term_freq.entry(token.text.clone()).or_insert(0) += 1;
        }

        // Update term stats
        for term in term_freq.keys() {
            let stats = self.term_stats.entry(term.clone()).or_default();
            stats.doc_frequency += 1;
            stats.total_frequency += term_freq[term];
        }

        // Store document
        self.documents.insert(doc_id, text.to_string());
        self.doc_tokens.insert(doc_id, tokens.clone());
        self.doc_term_freqs.insert(doc_id, term_freq);
        self.doc_count += 1;
        self.total_tokens += tokens.len();
        self.avg_doc_length = self.total_tokens as f64 / self.doc_count as f64;

        doc_id
    }

    /// Batch add documents
    pub fn add_documents<I, S>(&mut self, texts: I) -> Vec<usize>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        texts
            .into_iter()
            .map(|t| self.add_document(t.as_ref()))
            .collect()
    }

    /// Remove a document from the index
    pub fn remove_document(&mut self, doc_id: usize) -> bool {
        if !self.documents.contains_key(&doc_id) {
            return false;
        }

        // Remove from posting lists
        for positions in self.posting_lists.values_mut() {
            positions.retain(|p| p.doc_id != doc_id);
        }
        // Clean up empty posting lists
        self.posting_lists.retain(|_, v| !v.is_empty());

        // Update term stats
        if let Some(term_freq) = self.doc_term_freqs.remove(&doc_id) {
            for (term, freq) in &term_freq {
                if let Some(stats) = self.term_stats.get_mut(term) {
                    stats.doc_frequency = stats.doc_frequency.saturating_sub(1);
                    stats.total_frequency = stats.total_frequency.saturating_sub(*freq);
                }
            }
        }
        self.term_stats.retain(|_, s| s.doc_frequency > 0);

        // Remove token data
        if let Some(tokens) = self.doc_tokens.remove(&doc_id) {
            self.total_tokens -= tokens.len();
        }

        self.documents.remove(&doc_id);
        self.doc_count -= 1;

        if self.doc_count > 0 {
            self.avg_doc_length = self.total_tokens as f64 / self.doc_count as f64;
        } else {
            self.avg_doc_length = 0.0;
        }

        true
    }

    /// Get term statistics
    pub fn term_stats(&self, term: &str) -> Option<&TermStats> {
        self.term_stats.get(term)
    }

    /// Get posting list for a term
    pub fn posting_list(&self, term: &str) -> Option<&Vec<TermPosition>> {
        self.posting_lists.get(term)
    }

    /// Get all terms that start with a prefix (for autocomplete)
    pub fn terms_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.terms_with_prefix_limited(prefix, usize::MAX)
    }

    /// Get terms with prefix but cap result size to avoid accidental memory spikes.
    pub fn terms_with_prefix_limited(&self, prefix: &str, limit: usize) -> Vec<String> {
        if limit == 0 {
            return vec![];
        }

        self.term_stats
            .keys()
            .filter(|t| t.starts_with(prefix))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get document text by ID
    pub fn document(&self, doc_id: usize) -> Option<&str> {
        self.documents.get(&doc_id).map(|s| s.as_str())
    }

    /// Get tokens for a document
    pub fn document_tokens(&self, doc_id: usize) -> Option<&Vec<Token>> {
        self.doc_tokens.get(&doc_id)
    }

    /// Get document length in tokens
    pub fn doc_length(&self, doc_id: usize) -> usize {
        self.doc_tokens.get(&doc_id).map(|t| t.len()).unwrap_or(0)
    }

    /// Number of documents in the index
    pub fn doc_count(&self) -> usize {
        self.doc_count
    }

    /// Average document length
    pub fn avg_doc_length(&self) -> f64 {
        self.avg_doc_length
    }

    /// Total number of unique terms
    pub fn term_count(&self) -> usize {
        self.term_stats.len()
    }

    /// Get all document IDs
    pub fn doc_ids(&self) -> Vec<usize> {
        self.documents.keys().copied().collect()
    }

    /// Get term frequency in a specific document
    pub fn term_freq_in_doc(&self, term: &str, doc_id: usize) -> usize {
        self.doc_term_freqs
            .get(&doc_id)
            .and_then(|tf| tf.get(term))
            .copied()
            .unwrap_or(0)
    }

    /// Get all term frequencies for a specific document.
    pub fn document_term_frequencies(&self, doc_id: usize) -> Option<&HashMap<String, usize>> {
        self.doc_term_freqs.get(&doc_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_add_and_search() {
        let mut idx = InvertedIndex::new();
        // "语言" in this doc — jieba combines "编程语言" as one token, so "语言" alone is NOT here
        idx.add_document("Rust是一门现代编程语言");
        // "语言" is a separate token here
        idx.add_document("Go语言也很流行");
        // "语言" is a separate token here
        idx.add_document("Python是一门易学的语言");

        assert_eq!(idx.doc_count(), 3);

        // The low-memory Chinese n-gram tokenizer can find "语言" inside "编程语言".
        let postings = idx.posting_list("语言").unwrap();
        let doc_ids: HashSet<usize> = postings.iter().map(|p| p.doc_id).collect();
        assert_eq!(doc_ids.len(), 3);
    }

    #[test]
    fn test_remove_document() {
        let mut idx = InvertedIndex::new();
        let id = idx.add_document("hello world");
        assert!(idx.remove_document(id));
        assert_eq!(idx.doc_count(), 0);
        assert!(idx.posting_list("hello").is_none());
    }

    #[test]
    fn test_prefix_search() {
        let mut idx = InvertedIndex::new();
        idx.add_document("programming in Rust");
        idx.add_document("programming in Go");
        idx.add_document("python development");

        let terms = idx.terms_with_prefix("pro");
        assert!(terms.contains(&"programming".to_string()));
    }

    #[test]
    fn test_batch_add() {
        let mut idx = InvertedIndex::new();
        let ids = idx.add_documents(["hello world", "foo bar", "baz qux"]);
        assert_eq!(ids.len(), 3);
        assert_eq!(idx.doc_count(), 3);
    }
}
