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

    pub fn with_tracks(mut self, tracks: Vec<Track>) -> Self {
        self.tracks = tracks;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_cover_url(mut self, cover_url: impl Into<String>) -> Self {
        self.cover_url = Some(cover_url.into());
        self
    }

    pub fn total_duration_ms(&self) -> u64 {
        self.tracks.iter().map(|t| t.duration_ms).sum()
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
}
