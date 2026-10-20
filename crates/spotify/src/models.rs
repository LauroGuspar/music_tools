use serde::{Deserialize, Serialize};

pub use core::{Playlist, Track};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpotifyUserProfile {
    pub id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub followers_count: Option<u64>,
    pub product: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthStatus {
    NotAuthenticated,
    WaitingForAuth { url: String },
    Authenticated { username: String },
    Failed { error: String },
}
