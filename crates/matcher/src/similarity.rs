use std::collections::HashSet;

/// Calculates the Sørensen-Dice similarity score between two strings using bigrams.
///
/// Returns 1.0 if both strings are identical without whitespace.
/// Returns 0.0 if either string has fewer than 2 characters or if there are no shared bigrams.
/// Otherwise returns (2 * intersection) / (bigrams1.len() + bigrams2.len()).
pub fn calculate_similarity(str1: &str, str2: &str) -> f64 {
    let s1: String = str1.chars().filter(|c| !c.is_whitespace()).collect();
    let s2: String = str2.chars().filter(|c| !c.is_whitespace()).collect();

    if s1 == s2 {
        return 1.0;
    }

    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();

    if chars1.len() < 2 || chars2.len() < 2 {
        return 0.0;
    }

    let mut b1: HashSet<(char, char)> = HashSet::with_capacity(chars1.len().saturating_sub(1));
    for window in chars1.windows(2) {
        b1.insert((window[0], window[1]));
    }

    let mut b2: HashSet<(char, char)> = HashSet::with_capacity(chars2.len().saturating_sub(1));
    for window in chars2.windows(2) {
        b2.insert((window[0], window[1]));
    }

    let total = b1.len() + b2.len();
    if total == 0 {
        return 0.0;
    }

    let mut intersection = 0;
    for bigram in &b1 {
        if b2.contains(bigram) {
            intersection += 1;
        }
    }

    (2.0 * intersection as f64) / (total as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_exact() {
        assert_eq!(calculate_similarity("hello", "hello"), 1.0);
        assert_eq!(calculate_similarity("  hello ", "hello  "), 1.0);
    }

    #[test]
    fn test_similarity_completely_different() {
        assert_eq!(calculate_similarity("abc", "xyz"), 0.0);
    }

    #[test]
    fn test_similarity_minor_typo() {
        let score = calculate_similarity("smells like teen spirit", "smells like ten spirit");
        assert!(score > 0.85, "Expected score > 0.85, got {score}");
    }

    #[test]
    fn test_similarity_short_strings() {
        assert_eq!(calculate_similarity("a", "b"), 0.0);
        assert_eq!(calculate_similarity("a", "a"), 1.0);
        assert_eq!(calculate_similarity("", ""), 1.0);
    }
}
