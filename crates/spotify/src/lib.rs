pub mod auth;
pub mod client;
pub mod models;
pub mod playlists;

pub use auth::{AuthConfig, DEFAULT_CLIENT_ID, DEFAULT_REDIRECT_URI};
pub use client::SpotifyClient;
pub use models::{AuthStatus, SpotifyUserProfile};
pub use playlists::{
    extract_playlist_id, extract_spotify_id, fetch_playlist, fetch_playlist_fast,
};
