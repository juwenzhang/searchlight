//! # Searchlight - 高性能中英文全文搜索引擎
//!
//! `searchlight` 是一个功能丰富的 Rust 全文搜索引擎库，支持：
//!
//! - **中英文分词**: 中文 n-gram 分词 + 英文/数字分词
//! - **倒排索引**: 高效的检索数据结构
//! - **布尔查询**: AND/OR/NOT 组合查询
//! - **模糊匹配**: 基于编辑距离的容错搜索
//! - **拼音搜索**: 用拼音搜索中文内容
//! - **精准短语搜索**: 双引号短语匹配
//! - **BM25 排序**: 专业的相关性排序算法
//! - **搜索高亮**: 结果自动生成高亮片段
//! - **批量检索**: 一次处理多个查询
//! - **自动补全**: 基于前缀的搜索建议
//!
//! ## 快速开始
//!
//! ```rust
//! use searchlight::SearchEngine;
//!
//! let mut engine = SearchEngine::new();
//!
//! // 索引文档
//! engine.index("Rust 是一门现代系统编程语言，安全且高效");
//! engine.index("Go 语言以其简洁和并发性能著称");
//! engine.index("Python 是数据科学和 AI 领域的首选语言");
//!
//! // 搜索
//! let results = engine.search("编程语言");
//! for r in &results {
//!     println!("[{}] {}", r.score, r.snippet.as_deref().unwrap_or(&r.document));
//! }
//!
//! // 拼音搜索
//! let results = engine.search_pinyin("biancheng");
//!
//! // 模糊搜索
//! let results = engine.search_fuzzy("programing", 2);
//! ```

mod fuzzy;
mod highlighter;
mod index;
mod pinyin;
mod query;
mod rank;
mod tokenizer;
#[cfg(feature = "wasm")]
mod wasm_api;

use std::collections::HashSet;

/// A search result containing the matched document and metadata
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SearchResult {
    /// Document ID in the index
    pub doc_id: usize,
    /// Relevance score (higher = more relevant)
    pub score: f64,
    /// Original document text
    pub document: String,
    /// Highlighted snippet (if highlighting was requested)
    pub snippet: Option<String>,
    /// Match positions in the document [(start, end), ...]
    pub match_positions: Vec<(usize, usize)>,
    /// Matched terms
    pub matched_terms: Vec<String>,
}

/// Search options for fine-grained control
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SearchOptions {
    /// Enable fuzzy matching
    pub fuzzy: bool,
    /// Maximum edit distance for fuzzy matching
    pub max_edit_distance: usize,
    /// Enable pinyin search (for Chinese content)
    pub use_pinyin: bool,
    /// Generate highlighted snippets
    pub highlight: bool,
    /// Maximum number of results to return
    pub limit: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            fuzzy: false,
            max_edit_distance: 2,
            use_pinyin: false,
            highlight: false,
            limit: 20,
        }
    }
}

/// The main search engine.
///
/// Manages document indexing, searching with various modes, and result ranking.
pub struct SearchEngine {
    index: InvertedIndex,
    ranking: Ranker,
    query_parser: QueryParser,
    highlighter: Highlighter,
    pinyin_index: PinyinIndex,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new() -> Self {
        SearchEngine {
            index: InvertedIndex::new(),
            ranking: Ranker::new(),
            query_parser: QueryParser::new(),
            highlighter: Highlighter::new(),
            pinyin_index: PinyinIndex::new(),
        }
    }

    /// Create a new engine with custom BM25 parameters
    pub fn with_bm25_params(k1: f64, b: f64) -> Self {
        SearchEngine {
            index: InvertedIndex::new(),
            ranking: Ranker::with_params(crate::rank::Bm25Params { k1, b }),
            query_parser: QueryParser::new(),
            highlighter: Highlighter::new(),
            pinyin_index: PinyinIndex::new(),
        }
    }

    /// Create a new engine with custom highlighter config
    pub fn with_highlighter_config(config: HighlighterConfig) -> Self {
        SearchEngine {
            index: InvertedIndex::new(),
            ranking: Ranker::new(),
            query_parser: QueryParser::new(),
            highlighter: Highlighter::with_config(config),
            pinyin_index: PinyinIndex::new(),
        }
    }

    // ==================== Indexing ====================

    /// Index a single document. Returns the assigned document ID.
    ///
    /// ```rust
    /// # use searchlight::SearchEngine;
    /// let mut engine = SearchEngine::new();
    /// let id = engine.index("Hello world from Rust!");
    /// ```
    pub fn index(&mut self, text: &str) -> usize {
        let tokens = crate::tokenizer::tokenize(text);

        // Update pinyin index for Chinese terms
        for token in &tokens {
            if token.kind == crate::tokenizer::TokenKind::Chinese {
                self.pinyin_index.add_term(&token.text);
            }
        }

        self.index.add_document(text)
    }

    /// Batch index multiple documents. Returns assigned document IDs.
    ///
    /// ```rust
    /// # use searchlight::SearchEngine;
    /// let mut engine = SearchEngine::new();
    /// let ids = engine.index_batch([
    ///     "Python 是 AI 开发的首选",
    ///     "Go 语言并发性能优秀",
    ///     "Rust 内存安全零成本抽象",
    /// ]);
    /// ```
    pub fn index_batch<I, S>(&mut self, texts: I) -> Vec<usize>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        texts.into_iter().map(|t| self.index(t.as_ref())).collect()
    }

    /// Remove a document from the index
    pub fn remove(&mut self, doc_id: usize) -> bool {
        self.index.remove_document(doc_id)
    }

    /// Get the total number of documents in the index
    pub fn doc_count(&self) -> usize {
        self.index.doc_count()
    }

    /// Get a document by ID
    pub fn get_document(&self, doc_id: usize) -> Option<&str> {
        self.index.document(doc_id)
    }

    // ==================== Searching ====================

    /// Basic full-text search with automatic query parsing.
    ///
    /// Supports plain terms, boolean operators, phrase queries, etc.
    ///
    /// ```rust
    /// # use searchlight::SearchEngine;
    /// let mut engine = SearchEngine::new();
    /// engine.index("Rust 是一门现代系统编程语言");
    /// engine.index("Python 是数据科学领域的语言");
    ///
    /// let results = engine.search("编程语言");
    /// assert!(!results.is_empty());
    /// ```
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        self.search_with_options(query, &SearchOptions::default())
    }

    /// Search with custom options
    pub fn search_with_options(&self, query: &str, options: &SearchOptions) -> Vec<SearchResult> {
        const MAX_QUERY_CHARS: usize = 512;
        const MAX_QUERY_TERMS: usize = 64;
        const MAX_CANDIDATES: usize = 10_000;
        const MAX_FUZZY_TERMS_SCAN: usize = 5_000;

        if query.chars().count() > MAX_QUERY_CHARS {
            return vec![];
        }

        let parsed = self.query_parser.parse(query);

        // Collect included terms and excluded (NOT) terms separately
        let mut include_terms = self.collect_include_terms(&parsed.root);
        let mut exclude_terms = self.collect_exclude_terms(&parsed.root);
        include_terms.truncate(MAX_QUERY_TERMS);
        exclude_terms.truncate(MAX_QUERY_TERMS);

        if include_terms.is_empty() && !options.use_pinyin && !parsed.use_pinyin {
            return vec![];
        }

        // --- Determine candidate documents ---
        let mut candidates: HashSet<usize> = HashSet::new();

        if options.use_pinyin || parsed.use_pinyin {
            let pinyin_results = self.pinyin_index.search_by_pinyin_detailed(query);
            for chinese_term in pinyin_results.keys() {
                if let Some(postings) = self.index.posting_list(chinese_term) {
                    for p in postings {
                        if candidates.len() >= MAX_CANDIDATES {
                            break;
                        }
                        candidates.insert(p.doc_id);
                    }
                }
            }
        }

        for term in &include_terms {
            if candidates.len() >= MAX_CANDIDATES {
                break;
            }
            if let Some(postings) = self.index.posting_list(term) {
                for p in postings {
                    if candidates.len() >= MAX_CANDIDATES {
                        break;
                    }
                    candidates.insert(p.doc_id);
                }
            }
        }

        // Fuzzy expansion — bounded scan and bounded result expansion
        let mut expanded_terms = include_terms.clone();
        let mut fuzzy_terms: HashSet<String> = HashSet::new();

        if options.fuzzy {
            let all_terms = self
                .index
                .terms_with_prefix_limited("", MAX_FUZZY_TERMS_SCAN);
            for term in &include_terms {
                let fuzzy_results =
                    fuzzy::fuzzy_match_limited(term, &all_terms, options.max_edit_distance, 32);
                for fm in &fuzzy_results {
                    if fuzzy_terms.len() >= MAX_QUERY_TERMS {
                        break;
                    }
                    fuzzy_terms.insert(fm.term.clone());
                    if let Some(postings) = self.index.posting_list(&fm.term) {
                        for p in postings {
                            if candidates.len() >= MAX_CANDIDATES {
                                break;
                            }
                            candidates.insert(p.doc_id);
                        }
                    }
                }
            }
        }

        // Merge expanded fuzzy terms into scoring terms
        for ft in &fuzzy_terms {
            if !expanded_terms.contains(ft) {
                expanded_terms.push(ft.clone());
            }
        }

        // --- Build exclude set ---
        let mut exclude_docs: HashSet<usize> = HashSet::new();
        for term in &exclude_terms {
            if let Some(postings) = self.index.posting_list(term) {
                for p in postings {
                    exclude_docs.insert(p.doc_id);
                }
            }
        }

        // --- Score and filter candidates ---
        let mut results: Vec<SearchResult> = Vec::new();
        let scoring_terms: &[String] = if options.fuzzy {
            &expanded_terms
        } else {
            &include_terms
        };

        for &doc_id in &candidates {
            // Skip documents that match exclude terms
            if exclude_docs.contains(&doc_id) {
                continue;
            }

            let score = if options.use_pinyin || parsed.use_pinyin {
                let bm25 = self.ranking.bm25_score(&self.index, doc_id, scoring_terms);
                let mut pinyin_bonus = 0.0;
                let doc_text = self.index.document(doc_id).unwrap_or("");
                if pinyin_matches(query, doc_text) {
                    pinyin_bonus = 2.0;
                }
                bm25 + pinyin_bonus
            } else {
                self.ranking
                    .combined_score(&self.index, doc_id, scoring_terms)
            };

            if score > 0.0 {
                let (matched_terms, match_positions) =
                    self.get_match_info(doc_id, scoring_terms, options);

                let snippet = if options.highlight {
                    self.highlighter
                        .highlight(&self.index, doc_id, scoring_terms)
                        .map(|s| s.highlighted)
                } else {
                    None
                };

                results.push(SearchResult {
                    doc_id,
                    score,
                    document: self.index.document(doc_id).unwrap_or("").to_string(),
                    snippet,
                    match_positions,
                    matched_terms,
                });
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(options.limit);
        results
    }

    /// Batch search: execute multiple queries and return results for each.
    ///
    /// ```rust
    /// # use searchlight::SearchEngine;
    /// let mut engine = SearchEngine::new();
    /// engine.index("Rust 编程语言");
    /// engine.index("Python 数据科学");
    ///
    /// let batch = engine.batch_search(["Rust", "Python", "Java"]);
    /// assert_eq!(batch.len(), 3);
    /// // batch[0] = results for "Rust"
    /// // batch[1] = results for "Python"
    /// // batch[2] = results for "Java"
    /// ```
    pub fn batch_search<I, S>(&self, queries: I) -> Vec<Vec<SearchResult>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        queries
            .into_iter()
            .map(|q| self.search(q.as_ref()))
            .collect()
    }

    /// Batch search with custom options
    pub fn batch_search_with_options<I, S>(
        &self,
        queries: I,
        options: &SearchOptions,
    ) -> Vec<Vec<SearchResult>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        queries
            .into_iter()
            .map(|q| self.search_with_options(q.as_ref(), options))
            .collect()
    }

    // ==================== Specialized Searches ====================

    /// Pinyin search: search Chinese content using pinyin input
    ///
    /// ```rust
    /// # use searchlight::SearchEngine;
    /// let mut engine = SearchEngine::new();
    /// engine.index("编程语言 Rust");
    /// engine.index("Python 数据分析");
    ///
    /// // Search "biancheng" (programming in pinyin)
    /// let results = engine.search_pinyin("biancheng");
    /// ```
    pub fn search_pinyin(&self, pinyin_query: &str) -> Vec<SearchResult> {
        let options = SearchOptions {
            use_pinyin: true,
            highlight: false,
            ..SearchOptions::default()
        };
        self.search_with_options(pinyin_query, &options)
    }

    /// Fuzzy search: find documents with terms similar to the query
    ///
    /// ```rust
    /// # use searchlight::SearchEngine;
    /// let mut engine = SearchEngine::new();
    /// engine.index("Hello programming world");
    /// engine.index("Hallo programing world");
    ///
    /// // "progamming" is misspelled, but fuzzy match finds "programming"
    /// let results = engine.search_fuzzy("progamming", 2);
    /// ```
    pub fn search_fuzzy(&self, query: &str, max_distance: usize) -> Vec<SearchResult> {
        let options = SearchOptions {
            fuzzy: true,
            max_edit_distance: max_distance,
            ..SearchOptions::default()
        };
        self.search_with_options(query, &options)
    }

    /// Phrase search: find documents containing an exact phrase
    ///
    /// ```rust
    /// # use searchlight::SearchEngine;
    /// let mut engine = SearchEngine::new();
    /// engine.index("I love Rust programming");
    /// engine.index("Rust is great for programming");
    ///
    /// let results = engine.search_phrase("Rust programming");
    /// ```
    pub fn search_phrase(&self, phrase: &str) -> Vec<SearchResult> {
        self.search(&format!("\"{}\"", phrase))
    }

    // ==================== Autocomplete / Suggestions ====================

    /// Get autocomplete suggestions based on a prefix.
    ///
    /// Returns terms in the index that start with the given prefix,
    /// ordered by document frequency (most common first).
    ///
    /// ```rust
    /// # use searchlight::SearchEngine;
    /// let mut engine = SearchEngine::new();
    /// engine.index("Rust programming language");
    /// engine.index("Python programming language");
    ///
    /// let suggestions = engine.suggest("pro");
    /// assert!(suggestions.contains(&"programming".to_string()));
    /// ```
    pub fn suggest(&self, prefix: &str) -> Vec<String> {
        let mut terms = self.index.terms_with_prefix(&prefix.to_lowercase());
        // Sort by document frequency (most frequent first)
        terms.sort_by(|a, b| {
            let freq_a = self
                .index
                .term_stats(a)
                .map(|s| s.doc_frequency)
                .unwrap_or(0);
            let freq_b = self
                .index
                .term_stats(b)
                .map(|s| s.doc_frequency)
                .unwrap_or(0);
            freq_b.cmp(&freq_a)
        });
        terms.truncate(10);
        terms
    }

    /// Get search suggestions including pinyin matches for Chinese text
    pub fn suggest_with_pinyin(&self, prefix: &str) -> Vec<String> {
        let mut suggestions = self.suggest(prefix);

        // Also suggest Chinese terms that match the pinyin prefix
        let pinyin_results = self.pinyin_index.search_by_pinyin(prefix);
        suggestions.extend(pinyin_results);

        // Deduplicate
        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(10);

        suggestions
    }

    // ==================== Utility ====================

    /// Get all document IDs in the index
    pub fn doc_ids(&self) -> Vec<usize> {
        self.index.doc_ids()
    }

    /// Clear the entire index
    pub fn clear(&mut self) {
        self.index = InvertedIndex::new();
        self.pinyin_index = PinyinIndex::new();
    }

    /// Get term frequency statistics
    pub fn term_statistics(&self, term: &str) -> Option<(usize, usize)> {
        self.index
            .term_stats(term)
            .map(|s| (s.doc_frequency, s.total_frequency))
    }

    // ==================== Internal Helpers ====================

    /// Collect included terms (NOT terms are excluded)
    fn collect_include_terms(&self, op: &QueryOp) -> Vec<String> {
        let mut terms = Vec::new();
        self.collect_terms_recursive(op, &mut terms, true);
        // Deduplicate
        terms.sort();
        terms.dedup();
        terms
    }

    /// Collect excluded (NOT) terms only
    fn collect_exclude_terms(&self, op: &QueryOp) -> Vec<String> {
        let mut terms = Vec::new();
        self.collect_exclude_recursive(op, &mut terms);
        terms.sort();
        terms.dedup();
        terms
    }

    fn collect_terms_recursive(&self, op: &QueryOp, terms: &mut Vec<String>, include: bool) {
        if !include {
            return;
        }
        match op {
            QueryOp::Term(t) => {
                terms.push(t.clone());
            }
            QueryOp::Phrase(phrase_terms) => {
                terms.extend(phrase_terms.clone());
            }
            QueryOp::Fuzzy(t, _) => {
                terms.push(t.clone());
            }
            QueryOp::Prefix(t) => {
                let matched = self.index.terms_with_prefix(t);
                terms.extend(matched);
            }
            QueryOp::CharMatch(chars) => {
                for c in chars {
                    terms.push(c.to_string());
                }
            }
            QueryOp::And(children) | QueryOp::Or(children) => {
                for child in children {
                    match child {
                        QueryOp::Not(_) => { /* skip — collected by exclude path */ }
                        _ => self.collect_terms_recursive(child, terms, true),
                    }
                }
            }
            QueryOp::Not(_) => { /* skip — exclude terms handled separately */ }
        }
    }

    fn collect_exclude_recursive(&self, op: &QueryOp, terms: &mut Vec<String>) {
        match op {
            QueryOp::Not(child) => {
                // Collect terms inside the NOT
                self.collect_not_child(child, terms);
            }
            QueryOp::And(children) | QueryOp::Or(children) => {
                for child in children {
                    self.collect_exclude_recursive(child, terms);
                }
            }
            _ => {}
        }
    }

    fn collect_not_child(&self, op: &QueryOp, terms: &mut Vec<String>) {
        match op {
            QueryOp::Term(t) => {
                terms.push(t.clone());
            }
            QueryOp::Phrase(phrase_terms) => {
                terms.extend(phrase_terms.clone());
            }
            QueryOp::Fuzzy(t, _) => {
                terms.push(t.clone());
            }
            QueryOp::Prefix(t) => {
                let matched = self.index.terms_with_prefix(t);
                terms.extend(matched);
            }
            QueryOp::And(children) | QueryOp::Or(children) => {
                for child in children {
                    self.collect_not_child(child, terms);
                }
            }
            QueryOp::Not(child) => {
                self.collect_not_child(child, terms);
            }
            _ => {}
        }
    }

    /// Extract match positions and matched terms for a document
    fn get_match_info(
        &self,
        doc_id: usize,
        query_terms: &[String],
        options: &SearchOptions,
    ) -> (Vec<String>, Vec<(usize, usize)>) {
        let mut matched_terms = Vec::new();
        let mut positions = Vec::new();

        for term in query_terms {
            let mut found = false;

            // Exact match
            if let Some(postings) = self.index.posting_list(term) {
                for pos in postings {
                    if pos.doc_id == doc_id {
                        positions.push((pos.start, pos.end));
                        found = true;
                    }
                }
            }

            // Fuzzy expansion (bounded)
            if !found && options.fuzzy {
                let all_terms = self.index.terms_with_prefix_limited("", 5_000);
                let fuzzy_results =
                    fuzzy::fuzzy_match_limited(term, &all_terms, options.max_edit_distance, 32);
                for fm in &fuzzy_results {
                    if let Some(postings) = self.index.posting_list(&fm.term) {
                        for pos in postings {
                            if pos.doc_id == doc_id {
                                positions.push((pos.start, pos.end));
                                matched_terms.push(fm.term.clone());
                            }
                        }
                    }
                }
            }

            if found {
                matched_terms.push(term.clone());
            }
        }

        // Deduplicate
        matched_terms.sort();
        matched_terms.dedup();
        positions.sort_by_key(|p| p.0);
        positions.dedup();

        (matched_terms, positions)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub use fuzzy::{
    damerau_levenshtein, fuzzy_match, fuzzy_match_limited, fuzzy_match_with_prefix, is_fuzzy_match,
    jaccard_similarity, lcs_similarity, levenshtein_distance, FuzzyMatch,
};
pub use highlighter::{Highlighter, HighlighterConfig, Snippet};
pub use index::InvertedIndex;
pub use pinyin::{pinyin_matches, PinyinConverter, PinyinIndex};
pub use query::{ParsedQuery, QueryOp, QueryParser};
pub use rank::{Bm25Params, Ranker};
pub use tokenizer::{
    contains_chinese, tokenize, tokenize_chars, tokenize_ngrams, Token, TokenKind,
};

#[cfg(feature = "wasm")]
pub use wasm_api::SearchEngine as WasmSearchEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_search() {
        let mut engine = SearchEngine::new();
        engine.index("Rust 是一门现代系统编程语言");
        engine.index("Go 语言也很流行");
        engine.index("Python 是数据科学领域的首选语言");

        let results = engine.search("编程语言");
        println!("Results: {:?}", results);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_chinese_search() {
        let mut engine = SearchEngine::new();
        engine.index("我爱北京天安门，天安门上太阳升");
        engine.index("上海是一个国际化大都市");
        engine.index("北京是中国的首都");

        let results = engine.search("北京");
        println!("Results: {:?}", results);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_boolean_search() {
        let mut engine = SearchEngine::new();
        engine.index("Rust and Python are great languages");
        engine.index("Python is easy to learn");
        engine.index("Rust is memory safe");

        // AND query
        let results = engine.search("Rust AND Python");
        println!("Results: {:?}", results);
        assert!(!results.is_empty());
        assert!(results[0].document.contains("Rust"));
        assert!(results[0].document.contains("Python"));
    }

    #[test]
    fn test_exclusion_search() {
        let mut engine = SearchEngine::new();
        engine.index("Rust programming language");
        engine.index("Python programming language");
        engine.index("JavaScript programming language");

        let results = engine.search("programming -Python");
        println!("Results: {:?}", results);
        // Should exclude the document with "Python"
        for r in &results {
            assert!(!r.document.to_lowercase().contains("python"));
        }
    }

    #[test]
    fn test_batch_search() {
        let mut engine = SearchEngine::new();
        engine.index("Rust systems programming");
        engine.index("Python data science");
        engine.index("Go concurrent programming");

        let batch = engine.batch_search(["rust", "python", "java"]);
        println!("Batch results: {:?}", batch);
        assert_eq!(batch.len(), 3);
        assert!(!batch[0].is_empty()); // "rust" finds something
        assert!(!batch[1].is_empty()); // "python" finds something
        assert!(batch[2].is_empty()); // "java" finds nothing
    }

    #[test]
    fn test_fuzzy_search() {
        let mut engine = SearchEngine::new();
        engine.index("programming in Rust");
        engine.index("writing Python code");

        let results = engine.search_fuzzy("programing", 2);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_pinyin_search() {
        let mut engine = SearchEngine::new();
        engine.index("编程语言 Rust 很强大");
        engine.index("Python 数据分析");

        let results = engine.search_pinyin("biancheng");
        println!("Results: {:?}", results);
        // Should find the document containing "编程"
        assert!(!results.is_empty());
    }

    #[test]
    fn test_autocomplete() {
        let mut engine = SearchEngine::new();
        engine.index("programming in Rust");
        engine.index("programming in Python");
        engine.index("Rust language");

        let suggestions = engine.suggest("pro");
        println!("Suggestions: {:?}", suggestions);
        assert!(suggestions.contains(&"programming".to_string()));
    }

    #[test]
    fn test_index_batch() {
        let mut engine = SearchEngine::new();
        let ids = engine.index_batch(["hello world", "foo bar", "baz qux hello"]);
        println!("Ids: {:?}", ids);
        assert_eq!(ids.len(), 3);
        assert_eq!(engine.doc_count(), 3);
    }

    #[test]
    fn test_remove_document() {
        let mut engine = SearchEngine::new();
        let id = engine.index("temporary document");
        println!("Id: {:?}", id);
        assert!(engine.remove(id));
        assert!(engine.search("temporary").is_empty());
    }

    #[test]
    fn test_phrase_search() {
        let mut engine = SearchEngine::new();
        engine.index("Rust programming is fun");
        engine.index("Programming in Rust is great");

        let results = engine.search_phrase("Rust programming");
        println!("Results: {:?}", results);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_with_highlight() {
        let mut engine = SearchEngine::new();
        engine.index("Rust 是一门现代系统编程语言，注重内存安全");

        let options = SearchOptions {
            highlight: true,
            ..SearchOptions::default()
        };
        let results = engine.search_with_options("编程语言", &options);
        println!("Results: {:?}", results);
        assert!(!results.is_empty());
        if let Some(snippet) = &results[0].snippet {
            assert!(snippet.contains("<em>"));
        }
    }

    #[test]
    fn test_statistics() {
        let mut engine = SearchEngine::new();
        engine.index("hello world hello");
        engine.index("hello foo");

        let stats = engine.term_statistics("hello");
        println!("Stats: {:?}", stats);
        assert!(stats.is_some());
        let (df, tf) = stats.unwrap();
        assert_eq!(df, 2); // appears in 2 docs
        assert_eq!(tf, 3); // total 3 times
    }
}
