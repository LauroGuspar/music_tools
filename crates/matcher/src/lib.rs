pub mod comparator;
pub mod normalizer;
pub mod similarity;

pub use comparator::{
    compare_tracks, compare_tracks_with_progress, is_match, is_match_normalized, ComparisonResult,
    DEFAULT_THRESHOLD, MatchResult, MatchType, NormalizedTrack,
};
pub use normalizer::{normalize_artist, normalize_title};
pub use similarity::calculate_similarity;
