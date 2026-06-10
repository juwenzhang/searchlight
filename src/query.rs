use crate::tokenizer::{self, is_chinese_char};

/// Represents different types of query operations
#[derive(Debug, Clone, PartialEq)]
pub enum QueryOp {
    /// Match a single term exactly
    Term(String),
    /// Match a phrase (sequence of terms in order)
    Phrase(Vec<String>),
    /// Fuzzy match with edit distance tolerance
    Fuzzy(String, usize),
    /// Prefix match (autocomplete style)
    Prefix(String),
    /// Match Chinese characters (individual char matching)
    CharMatch(Vec<char>),
    /// AND of multiple query ops
    And(Vec<QueryOp>),
    /// OR of multiple query ops
    Or(Vec<QueryOp>),
    /// NOT (exclude) a query op
    Not(Box<QueryOp>),
}

/// Parsed query with associated metadata
#[derive(Debug, Clone)]
pub struct ParsedQuery {
    /// The root query operation
    pub root: QueryOp,
    /// Original query string
    pub original: String,
    /// Whether pinyin matching is requested
    pub use_pinyin: bool,
    /// Maximum edit distance for fuzzy matches
    pub max_edit_distance: usize,
}

/// Query parser for the search engine.
/// Supports:
/// - Simple term queries: `hello world`
/// - Phrase queries: `"exact phrase"`
/// - Boolean AND: `hello AND world` or `+hello +world`
/// - Boolean OR: `hello OR world`
/// - Boolean NOT: `-exclude` or `NOT exclude`
/// - Fuzzy queries: `term~` or `term~2`
/// - Prefix queries: `prefix*`
/// - Pinyin flag: `pinyin:ni hao` or `py:你好`
pub struct QueryParser {
    default_fuzzy_distance: usize,
}

impl QueryParser {
    pub fn new() -> Self {
        QueryParser {
            default_fuzzy_distance: 2,
        }
    }

    pub fn with_fuzzy_distance(distance: usize) -> Self {
        QueryParser {
            default_fuzzy_distance: distance,
        }
    }

    /// Parse a query string into a ParsedQuery
    pub fn parse(&self, input: &str) -> ParsedQuery {
        let original = input.to_string();
        let trimmed = input.trim();

        // Check for pinyin mode flag
        let (use_pinyin, query_text) =
            if trimmed.starts_with("pinyin:") || trimmed.starts_with("py:") {
                let q = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
                (true, q.to_string())
            } else {
                (false, trimmed.to_string())
            };

        let root = if query_text.is_empty() {
            QueryOp::And(vec![])
        } else {
            self.parse_expression(&query_text)
        };

        ParsedQuery {
            root,
            original,
            use_pinyin,
            max_edit_distance: self.default_fuzzy_distance,
        }
    }

    /// Parse a query expression, handling OR at the top level
    fn parse_expression(&self, input: &str) -> QueryOp {
        // Check for quoted phrase
        if let Some(phrase) = self.try_parse_phrase(input) {
            return phrase;
        }

        // Split by OR (lowest precedence)
        let or_parts = self.split_operator(input, "OR");
        if or_parts.len() > 1 {
            let ops: Vec<QueryOp> = or_parts
                .iter()
                .map(|p| self.parse_and(p.trim()))
                .filter(|op| !matches!(op, QueryOp::And(ref v) if v.is_empty()))
                .collect();
            return if ops.len() == 1 {
                ops.into_iter().next().unwrap()
            } else {
                QueryOp::Or(ops)
            };
        }

        self.parse_and(input)
    }

    /// Parse AND-connected terms
    fn parse_and(&self, input: &str) -> QueryOp {
        let and_parts = self.split_operator(input, "AND");

        let mut terms: Vec<QueryOp> = Vec::new();
        let mut exclude: Vec<QueryOp> = Vec::new();

        for part in &and_parts {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Handle NOT / exclusion with - prefix (both at start and inline)
            if let Some(rest) = trimmed.strip_prefix('-') {
                exclude.push(self.parse_term(rest.trim()));
                continue;
            }

            // Check for inline -exclusions like "hello -world"
            if let Some(pos) = trimmed.find(" -") {
                let include_part = &trimmed[..pos].trim();
                let exclude_part = &trimmed[pos + 2..].trim();
                if !include_part.is_empty() {
                    terms.push(self.parse_term(include_part));
                }
                if !exclude_part.is_empty() {
                    exclude.push(self.parse_term(exclude_part));
                }
                continue;
            }

            terms.push(self.parse_term(trimmed));
        }

        // Combine includes and excludes
        let mut all_ops = terms;
        for ex in exclude {
            all_ops.push(QueryOp::Not(Box::new(ex)));
        }

        if all_ops.is_empty() {
            QueryOp::And(vec![])
        } else if all_ops.len() == 1 {
            all_ops.into_iter().next().unwrap()
        } else {
            QueryOp::And(all_ops)
        }
    }

    /// Parse a single term with possible modifiers
    fn parse_term(&self, input: &str) -> QueryOp {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return QueryOp::And(vec![]);
        }

        // Fuzzy match: term~ or term~2
        if let Some(idx) = trimmed.rfind('~') {
            let term = &trimmed[..idx];
            let distance_str = &trimmed[idx + 1..];
            let distance = if distance_str.is_empty() {
                self.default_fuzzy_distance
            } else {
                distance_str
                    .parse::<usize>()
                    .unwrap_or(self.default_fuzzy_distance)
            };
            if !term.is_empty() {
                // Tokenize the fuzzy term in case it contains multiple words
                let tokens = tokenizer::tokenize(term);
                if tokens.len() == 1 {
                    return QueryOp::Fuzzy(tokens[0].text.clone(), distance.min(3));
                } else {
                    let terms: Vec<String> = tokens.into_iter().map(|t| t.text).collect();
                    return QueryOp::And(
                        terms
                            .into_iter()
                            .map(|t| QueryOp::Fuzzy(t, distance.min(3)))
                            .collect(),
                    );
                }
            }
        }

        // Prefix match: term*
        if let Some(prefix) = trimmed.strip_suffix('*') {
            if !prefix.is_empty() {
                let tokens = tokenizer::tokenize(prefix);
                if tokens.len() == 1 {
                    return QueryOp::Prefix(tokens[0].text.clone());
                } else {
                    let terms: Vec<String> = tokens.into_iter().map(|t| t.text).collect();
                    return QueryOp::And(terms.into_iter().map(|t| QueryOp::Prefix(t)).collect());
                }
            }
        }

        // Multi-word term (space-separated, outside phrase)
        let tokens = tokenizer::tokenize(trimmed);
        let has_chinese = trimmed.chars().any(is_chinese_char);

        if has_chinese && tokens.len() >= 2 {
            // For Chinese, also try character-level matching
            let chars: Vec<char> = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
            if chars.len() >= 2 {
                return QueryOp::Or(vec![
                    QueryOp::And(tokens.into_iter().map(|t| QueryOp::Term(t.text)).collect()),
                    QueryOp::CharMatch(chars),
                ]);
            }
        }

        if tokens.len() == 1 {
            QueryOp::Term(tokens[0].text.clone())
        } else {
            QueryOp::And(tokens.into_iter().map(|t| QueryOp::Term(t.text)).collect())
        }
    }

    /// Try to parse a quoted phrase
    fn try_parse_phrase(&self, input: &str) -> Option<QueryOp> {
        let trimmed = input.trim();
        if trimmed.starts_with('"') && trimmed.contains('"') {
            let end = trimmed[1..].find('"').map(|i| i + 1)?;
            let phrase = &trimmed[1..end];
            let tokens = tokenizer::tokenize(phrase);
            let terms: Vec<String> = tokens.into_iter().map(|t| t.text).collect();
            if !terms.is_empty() {
                return Some(QueryOp::Phrase(terms));
            }
        }
        None
    }

    /// Split input by an operator, respecting quoted sections
    fn split_operator(&self, input: &str, operator: &str) -> Vec<String> {
        let op_upper = operator.to_uppercase();
        let mut parts = Vec::new();
        let mut in_quote = false;
        let mut current = String::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '"' {
                in_quote = !in_quote;
                current.push(chars[i]);
                i += 1;
                continue;
            }

            if !in_quote && i + operator.len() <= chars.len() {
                let slice: String = chars[i..i + operator.len()].iter().collect();
                let slice_upper = slice.to_uppercase();

                // Check word boundaries
                let prev_is_boundary = i == 0 || chars[i - 1].is_whitespace();
                let next_is_boundary =
                    i + operator.len() >= chars.len() || chars[i + operator.len()].is_whitespace();

                if slice_upper == op_upper && prev_is_boundary && next_is_boundary {
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                    }
                    current = String::new();
                    i += operator.len();
                    continue;
                }
            }

            current.push(chars[i]);
            i += 1;
        }

        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        if parts.is_empty() {
            vec![input.to_string()]
        } else {
            parts
        }
    }
}

impl Default for QueryParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_term() {
        let parser = QueryParser::new();
        let q = parser.parse("hello");
        assert_eq!(q.root, QueryOp::Term("hello".to_string()));
    }

    #[test]
    fn test_and_query() {
        let parser = QueryParser::new();
        let q = parser.parse("hello AND world");
        assert!(matches!(q.root, QueryOp::And(_)));
        if let QueryOp::And(ops) = &q.root {
            assert_eq!(ops.len(), 2);
        }
    }

    #[test]
    fn test_or_query() {
        let parser = QueryParser::new();
        let q = parser.parse("hello OR world");
        assert!(matches!(q.root, QueryOp::Or(_)));
    }

    #[test]
    fn test_not_query() {
        let parser = QueryParser::new();
        let q = parser.parse("hello -world");
        assert!(matches!(q.root, QueryOp::And(_)));
        if let QueryOp::And(ops) = &q.root {
            assert!(ops.iter().any(|op| matches!(op, QueryOp::Not(_))));
        }
    }

    #[test]
    fn test_phrase_query() {
        let parser = QueryParser::new();
        let q = parser.parse("\"hello world\"");
        assert!(matches!(q.root, QueryOp::Phrase(_)));
    }

    #[test]
    fn test_fuzzy_query() {
        let parser = QueryParser::new();
        let q = parser.parse("hello~");
        assert!(matches!(q.root, QueryOp::Fuzzy(_, _)));
    }

    #[test]
    fn test_fuzzy_with_distance() {
        let parser = QueryParser::new();
        let q = parser.parse("hello~2");
        if let QueryOp::Fuzzy(_, d) = &q.root {
            assert_eq!(*d, 2);
        } else {
            panic!("Expected Fuzzy query");
        }
    }

    #[test]
    fn test_prefix_query() {
        let parser = QueryParser::new();
        let q = parser.parse("prog*");
        assert!(matches!(q.root, QueryOp::Prefix(_)));
    }

    #[test]
    fn test_pinyin_mode() {
        let parser = QueryParser::new();
        let q = parser.parse("pinyin:ni hao");
        assert!(q.use_pinyin);
    }
}
