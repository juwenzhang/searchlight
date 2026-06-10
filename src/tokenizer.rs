/// Detect if a character is a Chinese character (CJK Unified Ideographs)
#[inline]
pub fn is_chinese_char(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}'
        | '\u{3400}'..='\u{4DBF}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{2F800}'..='\u{2FA1F}'
    )
}

#[inline]
pub fn contains_chinese(text: &str) -> bool {
    text.chars().any(is_chinese_char)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Chinese,
    English,
    Number,
    Symbol,
}

impl Token {
    pub fn new(text: String, start: usize, end: usize, kind: TokenKind) -> Self {
        Token {
            text,
            start,
            end,
            kind,
        }
    }
}

/// Low-memory tokenizer for mixed Chinese / English text.
///
/// Design notes:
/// - No global dictionary is loaded, avoiding the large memory spike from dictionary-based tokenizers.
/// - English / numbers are grouped by word.
/// - Chinese runs emit the whole run plus bounded 2~4 char n-grams, which gives useful recall
///   for terms such as `北京`, `天安门`, `编程语言` without quadratic memory growth.
pub fn tokenize(text: &str) -> Vec<Token> {
    const MAX_CHINESE_NGRAM: usize = 4;

    let mut tokens = Vec::new();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0;

    while i < chars.len() {
        let (start, ch) = chars[i];

        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        if is_chinese_char(ch) {
            let run_start = i;
            i += 1;
            while i < chars.len() && is_chinese_char(chars[i].1) {
                i += 1;
            }
            push_chinese_run(text, &chars, run_start, i, MAX_CHINESE_NGRAM, &mut tokens);
            continue;
        }

        if ch.is_ascii_alphabetic() {
            let mut end_i = i + 1;
            while end_i < chars.len() {
                let c = chars[end_i].1;
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    end_i += 1;
                } else {
                    break;
                }
            }
            let end = byte_end(text, &chars, end_i - 1);
            tokens.push(Token::new(
                text[start..end].to_lowercase(),
                start,
                end,
                TokenKind::English,
            ));
            i = end_i;
            continue;
        }

        if ch.is_ascii_digit() {
            let mut end_i = i + 1;
            while end_i < chars.len() && chars[end_i].1.is_ascii_digit() {
                end_i += 1;
            }
            let end = byte_end(text, &chars, end_i - 1);
            tokens.push(Token::new(
                text[start..end].to_string(),
                start,
                end,
                TokenKind::Number,
            ));
            i = end_i;
            continue;
        }

        let end = start + ch.len_utf8();
        tokens.push(Token::new(ch.to_string(), start, end, TokenKind::Symbol));
        i += 1;
    }

    tokens
}

fn push_chinese_run(
    text: &str,
    chars: &[(usize, char)],
    run_start: usize,
    run_end: usize,
    max_ngram: usize,
    tokens: &mut Vec<Token>,
) {
    let len = run_end - run_start;
    if len == 0 {
        return;
    }

    // Whole run token
    let start = chars[run_start].0;
    let end = byte_end(text, chars, run_end - 1);
    tokens.push(Token::new(
        text[start..end].to_string(),
        start,
        end,
        TokenKind::Chinese,
    ));

    // Bounded n-grams: O(n * max_ngram), not O(n^2)
    for n in 1..=max_ngram.min(len) {
        for offset in 0..=len - n {
            let s_idx = run_start + offset;
            let e_idx = s_idx + n - 1;
            let s = chars[s_idx].0;
            let e = byte_end(text, chars, e_idx);
            let token_text = text[s..e].to_string();
            if token_text.len() < text[start..end].len() || n == 1 {
                tokens.push(Token::new(token_text, s, e, TokenKind::Chinese));
            }
        }
    }

    tokens.sort_by(|a, b| (a.start, a.end, &a.text).cmp(&(b.start, b.end, &b.text)));
    tokens.dedup_by(|a, b| a.start == b.start && a.end == b.end && a.text == b.text);
}

fn byte_end(text: &str, chars: &[(usize, char)], char_idx: usize) -> usize {
    chars
        .get(char_idx)
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(text.len())
}

pub fn tokenize_ngrams(text: &str, max_ngram: usize) -> Vec<Token> {
    if max_ngram == 0 {
        return vec![];
    }

    let base_tokens = tokenize(text);
    let mut result = base_tokens.clone();
    let capped = max_ngram.min(8);

    for start in 0..base_tokens.len() {
        for n in 2..=capped {
            if start + n > base_tokens.len() {
                break;
            }
            let slice = &base_tokens[start..start + n];
            let combined = slice
                .iter()
                .map(|t| t.text.as_str())
                .collect::<Vec<_>>()
                .join("");
            result.push(Token::new(
                combined,
                slice[0].start,
                slice[n - 1].end,
                slice[0].kind,
            ));
        }
    }

    result.sort_by(|a, b| (a.start, a.end, &a.text).cmp(&(b.start, b.end, &b.text)));
    result.dedup_by(|a, b| a.start == b.start && a.end == b.end && a.text == b.text);
    result
}

pub fn tokenize_chars(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for (i, c) in text.char_indices() {
        if c.is_whitespace() {
            continue;
        }
        let kind = if is_chinese_char(c) {
            TokenKind::Chinese
        } else if c.is_alphabetic() {
            TokenKind::English
        } else if c.is_numeric() {
            TokenKind::Number
        } else {
            TokenKind::Symbol
        };

        let text_str = if kind == TokenKind::English {
            c.to_lowercase().to_string()
        } else {
            c.to_string()
        };

        tokens.push(Token::new(text_str, i, i + c.len_utf8(), kind));
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_chinese() {
        let text = "我爱北京天安门";
        let tokens = tokenize(text);
        let words: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(words.contains(&"北京"));
        assert!(words.contains(&"天安门"));
    }

    #[test]
    fn test_tokenize_english() {
        let text = "Hello world from Rust";
        let tokens = tokenize(text);
        let words: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(words, vec!["hello", "world", "from", "rust"]);
    }

    #[test]
    fn test_tokenize_mixed() {
        let text = "Rust是一门现代编程语言";
        let tokens = tokenize(text);
        let words: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(words.contains(&"rust"));
        assert!(words.contains(&"编程语言"));
    }

    #[test]
    fn test_contains_chinese() {
        assert!(contains_chinese("你好"));
        assert!(!contains_chinese("hello"));
        assert!(contains_chinese("hello你好"));
    }
}
