use core::Track;
use serde::{Deserialize, Serialize};

use crate::normalizer::{normalize_artist, normalize_title};
use crate::similarity::calculate_similarity;

pub const DEFAULT_THRESHOLD: f64 = 0.80;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchType {
    Exact,
    Probable,
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchResult {
    pub matched: bool,
    pub score: f64,
    #[serde(rename = "type", alias = "match_type")]
    pub match_type: MatchType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedTrack<'a> {
    pub track: &'a Track,
    pub norm_title: String,
    pub norm_artists: Vec<String>,
}

impl<'a> NormalizedTrack<'a> {
    pub fn new(track: &'a Track) -> Self {
        Self {
            norm_title: normalize_title(&track.title),
            norm_artists: track.artists.iter().map(|a| normalize_artist(a)).collect(),
            track,
        }
    }
}

pub fn is_match_normalized(
    track_a: &NormalizedTrack,
    track_b: &NormalizedTrack,
    threshold: f64,
) -> MatchResult {
    let has_artist_intersection = track_a
        .norm_artists
        .iter()
        .any(|a| track_b.norm_artists.contains(a));
    let has_any_artist_missing = track_a.norm_artists.is_empty() || track_b.norm_artists.is_empty();

    let is_exact_title = track_a.norm_title == track_b.norm_title;

    if is_exact_title && has_artist_intersection {
        return MatchResult {
            matched: true,
            score: 1.0,
            match_type: MatchType::Exact,
        };
    }

    let title_similarity = calculate_similarity(&track_a.norm_title, &track_b.norm_title);

    if is_exact_title {
        if has_any_artist_missing {
            return MatchResult {
                matched: true,
                score: 1.0,
                match_type: MatchType::Probable,
            };
        }
        return MatchResult {
            matched: false,
            score: 0.0,
            match_type: MatchType::None,
        };
    }

    if title_similarity >= threshold && (has_artist_intersection || has_any_artist_missing) {
        return MatchResult {
            matched: true,
            score: title_similarity,
            match_type: MatchType::Probable,
        };
    }

    MatchResult {
        matched: false,
        score: title_similarity,
        match_type: MatchType::None,
    }
}

/// Evaluates if two tracks match according to title normalization and fuzzy similarity.
pub fn is_match(track_a: &Track, track_b: &Track, threshold: f64) -> MatchResult {
    let norm_a = NormalizedTrack::new(track_a);
    let norm_b = NormalizedTrack::new(track_b);
    is_match_normalized(&norm_a, &norm_b, threshold)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub exact_matches: Vec<(Track, Track)>,
    pub probable_matches: Vec<(Track, Track, f64)>,
    pub missing: Vec<Track>,
    pub extra: Vec<Track>,
}

/// Compares a list of source tracks with a list of target tracks.
pub fn compare_tracks(
    source: &[Track],
    target: &[Track],
    threshold: f64,
) -> ComparisonResult {
    compare_tracks_with_progress(source, target, threshold, |_, _| {})
}

/// Compares a list of source tracks with target tracks, invoking a callback with progress (processed, total).
pub fn compare_tracks_with_progress<F>(
    source: &[Track],
    target: &[Track],
    threshold: f64,
    mut progress: F,
) -> ComparisonResult
where
    F: FnMut(usize, usize),
{
    let mut exact_matches = Vec::new();
    let mut probable_matches = Vec::new();
    let mut missing = Vec::new();

    let norm_source: Vec<NormalizedTrack> = source.iter().map(NormalizedTrack::new).collect();
    let mut remaining_targets: Vec<NormalizedTrack> =
        target.iter().map(NormalizedTrack::new).collect();

    let total = norm_source.len();

    for (idx, source_track) in norm_source.into_iter().enumerate() {
        let mut matched_index = None;
        for (i, target_track) in remaining_targets.iter().enumerate() {
            let res = is_match_normalized(&source_track, target_track, threshold);
            if res.matched && res.match_type == MatchType::Exact {
                matched_index = Some(i);
                exact_matches.push((source_track.track.clone(), target_track.track.clone()));
                break;
            }
        }

        if let Some(i) = matched_index {
            remaining_targets.remove(i);
            progress(idx + 1, total);
            continue;
        }

        let mut best_probable_index = None;
        let mut best_score = 0.0;
        for (i, target_track) in remaining_targets.iter().enumerate() {
            let res = is_match_normalized(&source_track, target_track, threshold);
            if res.matched && res.match_type == MatchType::Probable && res.score > best_score {
                best_score = res.score;
                best_probable_index = Some(i);
            }
        }

        if let Some(i) = best_probable_index {
            let target_track = remaining_targets.remove(i);
            probable_matches.push((
                source_track.track.clone(),
                target_track.track.clone(),
                best_score,
            ));
            progress(idx + 1, total);
            continue;
        }

        missing.push(source_track.track.clone());
        progress(idx + 1, total);
    }

    let extra: Vec<Track> = remaining_targets
        .into_iter()
        .map(|t| t.track.clone())
        .collect();

    ComparisonResult {
        exact_matches,
        probable_matches,
        missing,
        extra,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_match_exact() {
        let track_a = Track::new("Let It Be - Remastered 2009", vec!["The Beatles".into()], 240000);
        let track_b = Track::new("Let It Be", vec!["The Beatles".into()], 240000);

        let res = is_match(&track_a, &track_b, DEFAULT_THRESHOLD);
        assert!(res.matched);
        assert_eq!(res.match_type, MatchType::Exact);
        assert_eq!(res.score, 1.0);
    }

    #[test]
    fn test_is_match_artist_intersection() {
        let track_a = Track::new("Under Pressure", vec!["Queen".into(), "David Bowie".into()], 240000);
        let track_b = Track::new("Under Pressure (Remastered)", vec!["Queen".into()], 240000);

        let res = is_match(&track_a, &track_b, DEFAULT_THRESHOLD);
        assert!(res.matched);
        assert_eq!(res.match_type, MatchType::Exact);
        assert_eq!(res.score, 1.0);
    }

    #[test]
    fn test_is_match_probable_typo() {
        let track_a = Track::new("Smells Like Teen Spirit", vec!["Nirvana".into()], 300000);
        let track_b = Track::new("Smells Like Ten Spirit", vec!["Nirvana".into()], 300000);

        let res = is_match(&track_a, &track_b, DEFAULT_THRESHOLD);
        assert!(res.matched);
        assert_eq!(res.match_type, MatchType::Probable);
        assert!(res.score > 0.85);
    }

    #[test]
    fn test_is_match_different_artists_no_match() {
        let track_a = Track::new("Creep", vec!["Radiohead".into()], 240000);
        let track_b = Track::new("Creep", vec!["TLC".into()], 240000);

        let res = is_match(&track_a, &track_b, DEFAULT_THRESHOLD);
        assert!(!res.matched);
        assert_eq!(res.match_type, MatchType::None);
    }

    #[test]
    fn test_is_match_missing_artist_probable() {
        let track_a = Track::new("Comfortably Numb", vec!["Pink Floyd".into()], 360000);
        let track_b = Track::new("Comfortably Numb", vec![], 360000);

        let res = is_match(&track_a, &track_b, DEFAULT_THRESHOLD);
        assert!(res.matched);
        assert_eq!(res.match_type, MatchType::Probable);
        assert_eq!(res.score, 1.0);
    }

    #[test]
    fn test_compare_tracks_categorization() {
        let source = vec![
            Track::new("Yesterday", vec!["The Beatles".into()], 125000),
            Track::new("Creep - Live", vec!["Radiohead".into()], 230000),
            Track::new("Smells Like Teen Spirit", vec!["Nirvana".into()], 301000),
            Track::new("Missing Track", vec!["Artist A".into()], 180000),
        ];

        let target = vec![
            Track::new("Yesterday", vec!["The Beatles".into()], 125000),
            Track::new("Creep", vec!["Radiohead".into()], 230000),
            Track::new("Smells Like Ten Spirit", vec!["Nirvana".into()], 301000),
            Track::new("Extra Track", vec!["Artist B".into()], 200000),
        ];

        let results = compare_tracks(&source, &target, DEFAULT_THRESHOLD);

        assert_eq!(results.exact_matches.len(), 2);
        assert_eq!(results.exact_matches[0].0.title, "Yesterday");
        assert_eq!(results.exact_matches[1].0.title, "Creep - Live");

        assert_eq!(results.probable_matches.len(), 1);
        assert_eq!(results.probable_matches[0].0.title, "Smells Like Teen Spirit");
        assert_eq!(results.probable_matches[0].1.title, "Smells Like Ten Spirit");

        assert_eq!(results.missing.len(), 1);
        assert_eq!(results.missing[0].title, "Missing Track");

        assert_eq!(results.extra.len(), 1);
        assert_eq!(results.extra[0].title, "Extra Track");
    }

    #[test]
    fn test_compare_tracks_with_progress_callback() {
        let source = vec![
            Track::new("Yesterday", vec!["The Beatles".into()], 125000),
            Track::new("Missing Track", vec!["Artist A".into()], 180000),
        ];
        let target = vec![
            Track::new("Yesterday", vec!["The Beatles".into()], 125000),
        ];

        let mut progress_calls = Vec::new();
        let results = compare_tracks_with_progress(&source, &target, DEFAULT_THRESHOLD, |proc, tot| {
            progress_calls.push((proc, tot));
        });

        assert_eq!(results.exact_matches.len(), 1);
        assert_eq!(results.missing.len(), 1);
        assert_eq!(progress_calls.len(), 2);
        assert_eq!(progress_calls[0], (1, 2));
        assert_eq!(progress_calls[1], (2, 2));
    }
}
