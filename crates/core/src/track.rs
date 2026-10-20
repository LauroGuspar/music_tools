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
    #[serde(default, alias = "size", alias = "size_bytes")]
    pub file_size: Option<u64>,
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
            file_size: None,
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_file_size(mut self, file_size: u64) -> Self {
        self.file_size = Some(file_size);
        self
    }

    pub fn with_album(mut self, album: impl Into<String>) -> Self {
        self.album = Some(album.into());
        self
    }

    pub fn with_release_date(mut self, release_date: impl Into<String>) -> Self {
        self.release_date = Some(release_date.into());
        self
    }

    pub fn with_spotify_id(mut self, spotify_id: impl Into<String>) -> Self {
        self.spotify_id = Some(spotify_id.into());
        self
    }

    pub fn with_isrc(mut self, isrc: impl Into<String>) -> Self {
        self.isrc = Some(isrc.into());
        self
    }

    pub fn artist_display(&self) -> String {
        if self.artists.is_empty() {
            "Unknown Artist".to_string()
        } else {
            self.artists.join(", ")
        }
    }
}
