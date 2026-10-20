use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub duration_ms: u64,
    pub path: Option<String>,
    pub spotify_id: Option<String>,
    pub isrc: Option<String>,
    pub release_date: Option<String>,
}

impl Track {
    pub fn new(title: impl Into<String>, artists: Vec<String>, duration_ms: u64) -> Self {
        Self {
            title: title.into(),
            artists,
            album: None,
            duration_ms,
            path: None,
            spotify_id: None,
            isrc: None,
            release_date: None,
        }
    }

    pub fn artist_display(&self) -> String {
        if self.artists.is_empty() {
            "Unknown Artist".to_string()
        } else {
            self.artists.join(", ")
        }
    }
}
