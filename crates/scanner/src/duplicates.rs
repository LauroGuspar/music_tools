use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use core::Track;
use matcher::{is_match_normalized, NormalizedTrack};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicateReason {
    ExactBytes,
    SemanticMatch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub id: String,
    pub reason: DuplicateReason,
    pub tracks: Vec<Track>,
    pub score: f64,
}

fn compute_sha256(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn detect_exact_duplicates(tracks: &[Track]) -> Vec<DuplicateGroup> {
    // 1. Group by size
    let mut size_buckets: HashMap<u64, Vec<&Track>> = HashMap::new();
    for track in tracks {
        let size = track
            .file_size
            .or_else(|| {
                track
                    .path
                    .as_ref()
                    .and_then(|p| std::fs::metadata(p).map(|m| m.len()).ok())
            });

        if let Some(sz) = size {
            size_buckets.entry(sz).or_default().push(track);
        }
    }

    let mut duplicate_groups = Vec::new();

    // 2. For buckets with >= 2 tracks, differentiate by SHA-256 if files exist
    for (size, bucket) in size_buckets {
        if bucket.len() < 2 {
            continue;
        }

        let mut hash_buckets: HashMap<String, Vec<&Track>> = HashMap::new();
        for track in bucket {
            let key = if let Some(path_str) = &track.path {
                let path = Path::new(path_str);
                if path.is_file() {
                    match compute_sha256(path) {
                        Ok(hash) => format!("sha256:{hash}"),
                        Err(_) => format!("size:{size}"),
                    }
                } else {
                    format!("size:{size}")
                }
            } else {
                format!("size:{size}")
            };

            hash_buckets.entry(key).or_default().push(track);
        }

        for (key, matching_tracks) in hash_buckets {
            if matching_tracks.len() >= 2 {
                let clean_key = key.replace(':', "-");
                duplicate_groups.push(DuplicateGroup {
                    id: format!("exact-{clean_key}"),
                    reason: DuplicateReason::ExactBytes,
                    tracks: matching_tracks.into_iter().cloned().collect(),
                    score: 1.0,
                });
            }
        }
    }

    duplicate_groups.sort_by(|a, b| a.id.cmp(&b.id));
    duplicate_groups
}

pub fn detect_semantic_duplicates(tracks: &[Track], threshold: f64) -> Vec<DuplicateGroup> {
    if tracks.len() < 2 {
        return Vec::new();
    }

    let normalized: Vec<NormalizedTrack> = tracks.iter().map(NormalizedTrack::new).collect();
    let n = tracks.len();

    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut [usize], i: usize) -> usize {
        if parent[i] == i {
            i
        } else {
            let p = parent[i];
            parent[i] = find(parent, p);
            parent[i]
        }
    }

    fn union(parent: &mut [usize], i: usize, j: usize) {
        let root_i = find(parent, i);
        let root_j = find(parent, j);
        if root_i != root_j {
            parent[root_i] = root_j;
        }
    }

    let mut pairwise_scores: Vec<(usize, usize, f64)> = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let res = is_match_normalized(&normalized[i], &normalized[j], threshold);
            if res.matched {
                union(&mut parent, i, j);
                pairwise_scores.push((i, j, res.score));
            }
        }
    }

    let mut components: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        components.entry(root).or_default().push(i);
    }

    let mut duplicate_groups = Vec::new();

    for (root, indices) in components {
        if indices.len() < 2 {
            continue;
        }

        let group_scores: Vec<f64> = pairwise_scores
            .iter()
            .filter(|(u, v, _)| indices.contains(u) && indices.contains(v))
            .map(|(_, _, s)| *s)
            .collect();

        let avg_score = if group_scores.is_empty() {
            threshold
        } else {
            group_scores.iter().sum::<f64>() / group_scores.len() as f64
        };

        let score = (avg_score * 10000.0).round() / 10000.0;
        let group_tracks: Vec<Track> = indices.iter().map(|&idx| tracks[idx].clone()).collect();
        let rep_title = &normalized[root].norm_title;
        let safe_title = rep_title.replace(|c: char| !c.is_alphanumeric(), "_");

        duplicate_groups.push(DuplicateGroup {
            id: format!("semantic-{root}-{safe_title}"),
            reason: DuplicateReason::SemanticMatch,
            tracks: group_tracks,
            score,
        });
    }

    duplicate_groups.sort_by(|a, b| a.id.cmp(&b.id));
    duplicate_groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_detect_exact_duplicates_in_memory() {
        let track1 = Track::new("Song A", vec!["Artist".into()], 180000).with_file_size(2048);
        let track2 = Track::new("Song A (Copy)", vec!["Artist".into()], 180000).with_file_size(2048);
        let track3 = Track::new("Song B", vec!["Artist".into()], 210000).with_file_size(4096);

        let tracks = vec![track1, track2, track3];
        let dups = detect_exact_duplicates(&tracks);

        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].reason, DuplicateReason::ExactBytes);
        assert_eq!(dups[0].score, 1.0);
        assert_eq!(dups[0].tracks.len(), 2);
    }

    #[test]
    fn test_detect_exact_duplicates_with_sha256() {
        let temp_dir = std::env::temp_dir().join(format!("mt_exact_dup_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let p1 = temp_dir.join("file1.mp3");
        let p2 = temp_dir.join("file2.mp3");
        let p3 = temp_dir.join("file3.mp3");

        let identical_content = b"identical audio bytes for test";
        let different_content = b"different bytes with same size";
        assert_eq!(identical_content.len(), different_content.len());

        fs::write(&p1, identical_content).unwrap();
        fs::write(&p2, identical_content).unwrap();
        fs::write(&p3, different_content).unwrap();

        let t1 = Track::new("Song 1", vec!["Artist".into()], 1000).with_path(p1.to_string_lossy().to_string());
        let t2 = Track::new("Song 2", vec!["Artist".into()], 1000).with_path(p2.to_string_lossy().to_string());
        let t3 = Track::new("Song 3", vec!["Artist".into()], 1000).with_path(p3.to_string_lossy().to_string());

        let dups = detect_exact_duplicates(&[t1, t2, t3]);

        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].tracks.len(), 2);
        let titles: Vec<&str> = dups[0].tracks.iter().map(|t| t.title.as_str()).collect();
        assert!(titles.contains(&"Song 1"));
        assert!(titles.contains(&"Song 2"));
        assert!(!titles.contains(&"Song 3"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_semantic_duplicates() {
        let t1 = Track::new("Yesterday", vec!["The Beatles".into()], 125000)
            .with_path("/music/Yesterday.mp3");
        let t2 = Track::new("Yesterday (Remastered 2009)", vec!["The Beatles".into()], 125000)
            .with_path("/music/Yesterday.flac");
        let t3 = Track::new("Creep", vec!["Radiohead".into()], 230000)
            .with_path("/music/Creep.mp3");
        let t4 = Track::new("Creep - Acoustic", vec!["Radiohead".into()], 230000)
            .with_path("/music/Creep_Acoustic.mp3");
        let t5 = Track::new("Smells Like Teen Spirit", vec!["Nirvana".into()], 300000)
            .with_path("/music/Smells.mp3");

        let tracks = vec![t1, t2, t3, t4, t5];
        let dups = detect_semantic_duplicates(&tracks, 0.80);

        // There should be 2 groups: Yesterday and Creep
        assert_eq!(dups.len(), 2);
        for group in &dups {
            assert_eq!(group.reason, DuplicateReason::SemanticMatch);
            assert_eq!(group.tracks.len(), 2);
            assert!(group.score >= 0.80);
        }
    }

    #[test]
    fn test_detect_semantic_duplicates_no_matches() {
        let t1 = Track::new("Song One", vec!["Artist A".into()], 120000);
        let t2 = Track::new("Different Title", vec!["Artist B".into()], 180000);
        let dups = detect_semantic_duplicates(&[t1, t2], 0.80);
        assert!(dups.is_empty());
    }
}
