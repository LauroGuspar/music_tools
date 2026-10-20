use std::sync::LazyLock;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

const KEYWORDS_PATTERN: &str = r"\b(?:remaster(?:ed)?|live|deluxe|edit(?:ed)?|version|mix|remix|bonus|mono|stereo|instrumental|acoustic)\b";

static BRACKET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let pattern = format!(r"(?i)[(\[][^()\[\]]*?{KEYWORDS_PATTERN}[^()\[\]]*?[)\]]");
    Regex::new(&pattern).expect("valid bracket regex")
});

static SUFFIX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let pattern = format!(r"(?i)(?:[-–—/\\:])\s*(?:\d{{4}}\s+)?{KEYWORDS_PATTERN}.*");
    Regex::new(&pattern).expect("valid suffix regex")
});

/// Normalizes a song title by:
/// 1. Converting to lowercase and trimming.
/// 2. Normalizing diacritics / accents (NFD without combining marks).
/// 3. Removing bracketed/parenthesized keyword buzzwords.
/// 4. Removing dashed/hyphenated suffix buzzwords.
/// 5. Removing apostrophes and dots to keep letters contiguous.
/// 6. Replacing other punctuation with spaces.
/// 7. Collapsing multiple spaces.
pub fn normalize_title(title: &str) -> String {
    let clean = title.trim().to_lowercase();

    // Normalize diacritics / accents
    let clean: String = clean
        .nfd()
        .filter(|&c| !('\u{0300}'..='\u{036f}').contains(&c))
        .collect();

    // 1. Remove bracketed / parenthesized content containing buzzwords
    let clean = BRACKET_REGEX.replace_all(&clean, "");

    // 2. Remove dash/hyphen suffixes containing buzzwords
    let clean = SUFFIX_REGEX.replace_all(&clean, "");

    // Remove apostrophes and dots to keep letters contiguous
    let clean: String = clean
        .chars()
        .filter(|&c| c != '\'' && c != '’' && c != '.')
        .collect();

    // Remove other punctuation (everything except letters, numbers, and spaces)
    let clean: String = clean
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c.is_whitespace() { c } else { ' ' })
        .collect();

    // Normalize spaces
    clean.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalizes an artist name by converting to lowercase, removing diacritics,
/// replacing non-alphanumeric punctuation with spaces, and collapsing whitespace.
pub fn normalize_artist(artist: &str) -> String {
    let clean = artist.trim().to_lowercase();

    // Normalize diacritics / accents
    let clean: String = clean
        .nfd()
        .filter(|&c| !('\u{0300}'..='\u{036f}').contains(&c))
        .collect();

    // Replace non-alphanumerics with spaces
    let clean: String = clean
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c.is_whitespace() { c } else { ' ' })
        .collect();

    // Normalize spaces
    clean.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title_whitespace() {
        assert_eq!(normalize_title("  Hello World  "), "hello world");
        assert_eq!(normalize_title("hello   world  test"), "hello world test");
    }

    #[test]
    fn test_normalize_title_diacritics() {
        assert_eq!(normalize_title("Canción del Adiós"), "cancion del adios");
        assert_eq!(normalize_title("München"), "munchen");
    }

    #[test]
    fn test_normalize_title_punctuation() {
        assert_eq!(
            normalize_title("Hello, World! (Testing...)"),
            "hello world testing"
        );
        assert_eq!(normalize_title("A.B.C."), "abc");
    }

    #[test]
    fn test_normalize_title_suffix_keywords() {
        // Dash suffixes
        assert_eq!(normalize_title("Let It Be - Remastered 2009"), "let it be");
        assert_eq!(normalize_title("Creep - Live At Wembley"), "creep");
        assert_eq!(normalize_title("Purple Rain - Deluxe Edition"), "purple rain");
        assert_eq!(normalize_title("Acoustic - Bonus Track"), "acoustic");

        // Parenthesis and bracket suffixes
        assert_eq!(
            normalize_title("Under Pressure (2011 Remaster)"),
            "under pressure"
        );
        assert_eq!(normalize_title("Radiohead (Live)"), "radiohead");
        assert_eq!(normalize_title("Song [Acoustic Version]"), "song");
        assert_eq!(normalize_title("Music (Radio Edit)"), "music");
        assert_eq!(normalize_title("Track (Mono Mix)"), "track");
        assert_eq!(normalize_title("Beat [Club Remix]"), "beat");
        assert_eq!(normalize_title("Interlude (Instrumental)"), "interlude");
    }

    #[test]
    fn test_normalize_artist() {
        assert_eq!(normalize_artist("  Béyonce  "), "beyonce");
        assert_eq!(normalize_artist("M83."), "m83");
    }
}
