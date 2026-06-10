use crate::tokenizer::is_chinese_char;
use pinyin::ToPinyin;
use std::collections::HashMap;

/// Pinyin representation options for a Chinese character
#[derive(Debug, Clone)]
pub struct PinyinToken {
    /// Full pinyin with tone number (e.g., "ni3")
    pub full: String,
    /// Pinyin without tone (e.g., "ni")
    pub plain: String,
    /// Initial consonant (e.g., "n")
    pub initial: String,
}

/// Convert Chinese text to pinyin tokens
pub struct PinyinConverter;

impl PinyinConverter {
    /// Convert a string to pinyin tokens, preserving non-Chinese text
    pub fn to_pinyin(text: &str) -> Vec<PinyinToken> {
        let mut results = Vec::new();

        for ch in text.chars() {
            if is_chinese_char(ch) {
                if let Some(pinyin) = ch.to_pinyin() {
                    let full = pinyin.with_tone().to_string();
                    let plain = pinyin.plain().to_string();
                    let initial = pinyin.first_letter().to_string();

                    results.push(PinyinToken {
                        full,
                        plain,
                        initial,
                    });
                }
            }
        }

        results
    }

    /// Get plain pinyin (without tones) for text
    pub fn to_plain_pinyin(text: &str) -> String {
        let tokens = Self::to_pinyin(text);
        tokens
            .iter()
            .map(|t| t.plain.as_str())
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Get first letters only (for acronym matching, e.g., "bj" for "北京")
    pub fn to_first_letters(text: &str) -> String {
        let tokens = Self::to_pinyin(text);
        tokens
            .iter()
            .map(|t| t.initial.as_str())
            .collect::<Vec<&str>>()
            .join("")
    }

    /// Get full pinyin with tone numbers
    pub fn to_full_pinyin(text: &str) -> String {
        let tokens = Self::to_pinyin(text);
        tokens
            .iter()
            .map(|t| t.full.as_str())
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

/// Build a pinyin search index: maps pinyin representations to original terms
#[derive(Debug, Clone, Default)]
pub struct PinyinIndex {
    /// Map from pinyin (plain) -> list of original Chinese terms
    pinyin_to_terms: HashMap<String, Vec<String>>,
    /// Map from first letters -> list of original Chinese terms
    initial_to_terms: HashMap<String, Vec<String>>,
}

impl PinyinIndex {
    pub fn new() -> Self {
        PinyinIndex::default()
    }

    /// Add a Chinese term's pinyin mapping
    pub fn add_term(&mut self, chinese_term: &str) {
        // Avoid indexing extremely long terms into pinyin maps; this prevents accidental
        // high-cardinality memory growth on huge Chinese paragraphs.
        if chinese_term.chars().count() > 32 {
            return;
        }

        let plain = PinyinConverter::to_plain_pinyin(chinese_term);
        let initials = PinyinConverter::to_first_letters(chinese_term);
        let term = chinese_term.to_string();

        if !plain.is_empty() {
            let terms = self.pinyin_to_terms.entry(plain).or_default();
            if !terms.contains(&term) {
                terms.push(term.clone());
            }
        }
        if !initials.is_empty() {
            let terms = self.initial_to_terms.entry(initials).or_default();
            if !terms.contains(&term) {
                terms.push(term);
            }
        }
    }

    /// Build pinyin index from a list of Chinese terms
    pub fn build_from_terms<I, S>(&mut self, terms: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for term in terms {
            self.add_term(term.as_ref());
        }
    }

    /// Search by pinyin: given a pinyin query, find matching Chinese terms
    pub fn search_by_pinyin(&self, pinyin_query: &str) -> Vec<String> {
        let query_plain = pinyin_query.to_lowercase().replace(' ', "");
        if query_plain.is_empty() {
            return vec![];
        }
        let mut results = Vec::new();

        // Exact match on plain pinyin
        if let Some(terms) = self.pinyin_to_terms.get(&query_plain) {
            results.extend(terms.clone());
        }

        // Exact match on initials
        if let Some(terms) = self.initial_to_terms.get(&query_plain) {
            results.extend(terms.clone());
        }

        // Prefix match on plain pinyin
        for (pinyin, terms) in &self.pinyin_to_terms {
            let pinyin_no_spaces = pinyin.replace(' ', "");
            if pinyin_no_spaces.starts_with(&query_plain) {
                results.extend(terms.clone());
            }
        }

        // Prefix match on initials
        for (initials, terms) in &self.initial_to_terms {
            if initials.starts_with(&query_plain) {
                results.extend(terms.clone());
            }
        }

        // Deduplicate
        results.sort();
        results.dedup();
        results
    }

    /// Search by pinyin, returning a map of matched original terms -> pinyin
    pub fn search_by_pinyin_detailed(&self, pinyin_query: &str) -> HashMap<String, String> {
        let query_plain = pinyin_query.to_lowercase().replace(' ', "");
        if query_plain.is_empty() {
            return HashMap::new();
        }
        let mut results = HashMap::new();

        for (pinyin, terms) in &self.pinyin_to_terms {
            let pinyin_no_spaces = pinyin.replace(' ', "");
            if pinyin_no_spaces.contains(&query_plain) || pinyin_no_spaces.starts_with(&query_plain)
            {
                for term in terms {
                    results
                        .entry(term.clone())
                        .or_insert_with(|| pinyin_no_spaces.clone());
                }
            }
        }

        results
    }
}

/// Check if a pinyin query matches Chinese text
pub fn pinyin_matches(query_pinyin: &str, chinese_text: &str) -> bool {
    let query_lower = query_pinyin.to_lowercase().replace(' ', "");
    if query_lower.is_empty() {
        return false;
    }

    let plain = PinyinConverter::to_plain_pinyin(chinese_text).replace(' ', "");
    let initials = PinyinConverter::to_first_letters(chinese_text);

    plain.contains(&query_lower) || initials.contains(&query_lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_plain_pinyin() {
        let pinyin = PinyinConverter::to_plain_pinyin("你好");
        assert!(!pinyin.is_empty());
        assert!(pinyin.contains("ni") || pinyin.contains("hao"));
    }

    #[test]
    fn test_to_first_letters() {
        let initials = PinyinConverter::to_first_letters("北京");
        assert_eq!(initials, "bj");
    }

    #[test]
    fn test_pinyin_index_search() {
        let mut idx = PinyinIndex::new();
        idx.add_term("北京");
        idx.add_term("背景");
        idx.add_term("上海");

        // Search by full pinyin
        let results = idx.search_by_pinyin("beijing");
        assert!(results.contains(&"北京".to_string()));
    }

    #[test]
    fn test_pinyin_index_initials() {
        let mut idx = PinyinIndex::new();
        idx.add_term("北京");
        idx.add_term("背景");

        // Search by initials only
        let results = idx.search_by_pinyin("bj");
        assert!(results.contains(&"北京".to_string()));
        assert!(results.contains(&"背景".to_string()));
    }

    #[test]
    fn test_pinyin_matches() {
        assert!(pinyin_matches("bei", "北京"));
        assert!(!pinyin_matches("shang", "北京"));
    }
}
