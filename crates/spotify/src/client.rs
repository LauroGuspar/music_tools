use anyhow::Result;
use core::Playlist;
use librespot_core::Session;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use storage::CacheStorage;

use crate::models::SpotifyUserProfile;
use crate::playlists::{fetch_playlist, fetch_playlist_fast};

const UNRESERVED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn escape_component(value: &str) -> String {
    utf8_percent_encode(value, UNRESERVED).to_string()
}

#[derive(Debug, Default, Deserialize)]
struct ProfilePayload {
    display_name: Option<String>,
    name: Option<String>,
    image_url: Option<String>,
    followers_count: Option<u64>,
}

#[derive(Clone)]
pub struct SpotifyClient {
    session: Session,
}

impl SpotifyClient {
    pub fn new(session: Session) -> Self {
        Self { session }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Checks if the session is alive and connected.
    pub fn is_connected(&self) -> bool {
        !self.session.is_invalid()
    }

    /// Returns the currently authenticated username.
    pub fn username(&self) -> String {
        self.session.username()
    }

    /// Fetches the profile of the currently signed-in user.
    pub async fn user_profile(&self) -> Result<SpotifyUserProfile> {
        let username = self.session.username();
        let user_data = self.session.user_data();

        let product = user_data.attributes.get("type").cloned();
        let email = user_data.attributes.get("email").cloned();

        let mut display_name = username.clone();
        let mut avatar_url = None;
        let mut followers_count = None;

        let escaped = escape_component(&username);
        if let Ok(body) = self.session.spclient().get_user_profile(&escaped, Some(0), Some(0)).await {
            if let Ok(profile) = serde_json::from_slice::<ProfilePayload>(&body) {
                if let Some(name) = profile.display_name.or(profile.name) {
                    if !name.trim().is_empty() {
                        display_name = name;
                    }
                }
                avatar_url = profile.image_url.filter(|u| !u.trim().is_empty());
                followers_count = profile.followers_count;
            }
        }

        Ok(SpotifyUserProfile {
            id: username,
            display_name,
            email,
            avatar_url,
            followers_count,
            product,
        })
    }

    /// Fetches a playlist with high-speed SQLite caching.
    pub async fn fetch_playlist(
        &self,
        playlist_id: &str,
        storage: Option<&CacheStorage>,
    ) -> Result<Playlist> {
        fetch_playlist_fast(&self.session, playlist_id, storage).await
    }

    /// Fetches a playlist forcing a fresh download from Spotify, updating SQLite cache.
    pub async fn fetch_playlist_refresh(
        &self,
        playlist_id: &str,
        storage: Option<&CacheStorage>,
    ) -> Result<Playlist> {
        fetch_playlist(&self.session, playlist_id, storage, true).await
    }
}
