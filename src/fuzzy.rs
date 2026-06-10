use std::cmp::min;

/// Compute Levenshtein (edit) distance between two strings.
/// Distance = minimum number of single-character edits
/// (insertions, deletions, substitutions) to change a into b.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Use two rows for space optimization
    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row: Vec<usize> = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr_row[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr_row[j] = min(
                curr_row[j - 1] + 1, // insertion
                min(
                    prev_row[j] + 1,        // deletion
                    prev_row[j - 1] + cost, // substitution
                ),
            );
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

/// Compute Damerau-Levenshtein distance (includes transpositions).
pub fn damerau_levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    let mut d = vec![vec![0usize; b_len + 2]; a_len + 2];

    let max_dist = a_len + b_len;
    d[0][0] = max_dist;

    for i in 0..=a_len {
        d[i + 1][0] = max_dist;
        d[i + 1][1] = i;
    }
    for j in 0..=b_len {
        d[0][j + 1] = max_dist;
        d[1][j + 1] = j;
    }

    // Build char->last position map for transposition detection
    use std::collections::HashMap;
    let mut da: HashMap<char, usize> = HashMap::new();

    for i in 1..=a_len {
        let mut db = 0usize;
        for j in 1..=b_len {
            let i1 = *da.get(&b_chars[j - 1]).unwrap_or(&0);
            let j1 = db;

            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                db = j;
                0
            } else {
                1
            };

            d[i + 1][j + 1] = min(
                d[i][j] + cost, // substitution
                min(
                    d[i + 1][j] + 1, // insertion
                    min(
                        d[i][j + 1] + 1,                             // deletion
                        d[i1][j1] + (i - i1 - 1) + 1 + (j - j1 - 1), // transposition
                    ),
                ),
            );
        }
        da.insert(a_chars[i - 1], i);
    }

    d[a_len + 1][b_len + 1]
}

/// Find fuzzy matches for a query term from a candidate list, within max distance.
pub fn fuzzy_match(term: &str, candidates: &[String], max_distance: usize) -> Vec<FuzzyMatch> {
    fuzzy_match_limited(term, candidates, max_distance, 64)
}

/// Find fuzzy matches with a hard result cap.
/// This prevents typo-search from collecting a huge result vector on large indexes.
pub fn fuzzy_match_limited(
    term: &str,
    candidates: &[String],
    max_distance: usize,
    limit: usize,
) -> Vec<FuzzyMatch> {
    if term.is_empty() || limit == 0 {
        return vec![];
    }

    let max_distance = max_distance.min(3);
    let term_chars = term.chars().count();
    let mut matches: Vec<FuzzyMatch> = Vec::new();

    for candidate in candidates {
        let candidate_chars = candidate.chars().count();
        if term_chars.abs_diff(candidate_chars) > max_distance {
            continue;
        }

        let distance = levenshtein_distance(term, candidate);
        if distance <= max_distance {
            let len_bonus = term_chars.min(candidate_chars) as f64
                / term_chars.max(candidate_chars).max(1) as f64;
            let distance_score = 1.0 - (distance as f64 / max_distance.max(1) as f64);
            let score = distance_score * 0.7 + len_bonus * 0.3;

            matches.push(FuzzyMatch {
                term: candidate.clone(),
                distance,
                score,
            });
        }
    }

    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(limit);
    matches
}

/// Find fuzzy matches with prefix boost (matches starting with same prefix score higher)
pub fn fuzzy_match_with_prefix(
    term: &str,
    candidates: &[String],
    max_distance: usize,
) -> Vec<FuzzyMatch> {
    let mut matches = fuzzy_match(term, candidates, max_distance);

    for m in &mut matches {
        if m.term.starts_with(term) {
            m.score *= 1.5; // Boost prefix matches
        }
    }

    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches
}

/// A fuzzy match result
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    /// The matched term
    pub term: String,
    /// Edit distance from query
    pub distance: usize,
    /// Relevance score (higher = better)
    pub score: f64,
}

/// Check if two strings are a fuzzy match within tolerance
pub fn is_fuzzy_match(query: &str, target: &str, max_distance: usize) -> bool {
    levenshtein_distance(query, target) <= max_distance
}

/// Compute Jaccard similarity between two token sets (for bag-of-words comparison)
pub fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    use std::collections::HashSet;
    let set_a: HashSet<&String> = a.iter().collect();
    let set_b: HashSet<&String> = b.iter().collect();

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Longest Common Subsequence ratio between two strings
pub fn lcs_similarity(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 || b_len == 0 {
        return 0.0;
    }

    let mut dp = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 1..=a_len {
        for j in 1..=b_len {
            if a_chars[i - 1] == b_chars[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let lcs_len = dp[a_len][b_len];
    lcs_len as f64 / a_len.max(b_len) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_single_edit() {
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
        assert_eq!(levenshtein_distance("hello", "hell"), 1);
        assert_eq!(levenshtein_distance("hello", "helloo"), 1);
    }

    #[test]
    fn test_levenshtein_chinese() {
        assert_eq!(levenshtein_distance("你好世界", "你好"), 2);
        assert_eq!(levenshtein_distance("你好", "你好"), 0);
    }

    #[test]
    fn test_fuzzy_match() {
        let candidates = vec![
            "hello".to_string(),
            "hallo".to_string(),
            "help".to_string(),
            "world".to_string(),
        ];
        let matches = fuzzy_match("hello", &candidates, 2);
        assert!(!matches.is_empty());
        // "hello" should be first (exact match)
        assert_eq!(matches[0].term, "hello");
    }

    #[test]
    fn test_jaccard() {
        let a = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let b = vec!["b".to_string(), "c".to_string(), "d".to_string()];
        let sim = jaccard_similarity(&a, &b);
        assert!((sim - 0.5).abs() < 0.01);
    }
}
