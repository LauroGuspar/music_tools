use serde::{Deserialize, Serialize};

use crate::track::Track;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tracks: Vec<Track>,
    pub cover_url: Option<String>,
}

impl Playlist {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            tracks: Vec::new(),
            cover_url: None,
        }
    }

    pub fn total_duration_ms(&self) -> u64 {
        self.tracks.iter().map(|t| t.duration_ms).sum()
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
}
