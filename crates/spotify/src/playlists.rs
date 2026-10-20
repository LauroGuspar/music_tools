use std::collections::HashMap;
use std::fmt::Write as _;

use anyhow::{Context as _, Result};
use core::{Playlist, Track};
use librespot_core::{Session, SpotifyId};
use librespot_protocol::extended_metadata::{BatchedEntityRequest, EntityRequest, ExtensionQuery};
use librespot_protocol::extension_kind::ExtensionKind;
use librespot_protocol::metadata::Track as TrackMessage;
use librespot_protocol::playlist4_external::{ListAttributes, SelectedListContent};
use protobuf::{EnumOrUnknown, Message as _};
use storage::CacheStorage;

const TRACK_PREFIX: &str = "spotify:track:";
const BATCH_SIZE: usize = 500;
const IMAGE_CDN: &str = "https://i.scdn.co/image/";
const BY_SIZE: [&str; 4] = ["xlarge", "large", "default", "small"];

/// Extracts a Spotify resource ID (22-character base62 string) from various formats:
/// - Plain 22-char alphanumeric ID
/// - Spotify URI (`spotify:playlist:...`, `spotify:track:...`, `spotify:album:...`)
/// - Open Spotify URL (`https://open.spotify.com/playlist/...`, `https://open.spotify.com/intl-es/playlist/...`)
pub fn extract_spotify_id(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(uri_rest) = trimmed.strip_prefix("spotify:") {
        for prefix in &["playlist:", "track:", "album:", "artist:"] {
            if let Some(id_part) = uri_rest.strip_prefix(prefix) {
                let id = id_part.split('?').next().unwrap_or(id_part);
                let id = id.split('#').next().unwrap_or(id);
                if is_valid_spotify_id(id) {
                    return Some(id.to_string());
                }
            }
        }
    }

    if trimmed.contains("open.spotify.com/") {
        let without_query = trimmed.split('?').next().unwrap_or(trimmed);
        let without_hash = without_query.split('#').next().unwrap_or(without_query);
        let segments: Vec<&str> = without_hash.split('/').filter(|s| !s.is_empty()).collect();

        for (idx, seg) in segments.iter().enumerate() {
            if (*seg == "playlist" || *seg == "track" || *seg == "album" || *seg == "artist")
                && idx + 1 < segments.len()
            {
                let candidate = segments[idx + 1];
                if is_valid_spotify_id(candidate) {
                    return Some(candidate.to_string());
                }
            }
        }
    }

    if is_valid_spotify_id(trimmed) {
        return Some(trimmed.to_string());
    }

    None
}

/// Extracts a playlist-specific Spotify ID from a URL, URI, or plain ID.
pub fn extract_playlist_id(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(uri_rest) = trimmed.strip_prefix("spotify:") {
        if let Some(id_part) = uri_rest.strip_prefix("playlist:") {
            let id = id_part.split('?').next().unwrap_or(id_part);
            let id = id.split('#').next().unwrap_or(id);
            if is_valid_spotify_id(id) {
                return Some(id.to_string());
            }
        }
        return None;
    }

    if trimmed.contains("open.spotify.com/") {
        let without_query = trimmed.split('?').next().unwrap_or(trimmed);
        let without_hash = without_query.split('#').next().unwrap_or(without_query);
        let segments: Vec<&str> = without_hash.split('/').filter(|s| !s.is_empty()).collect();

        for (idx, seg) in segments.iter().enumerate() {
            if *seg == "playlist" && idx + 1 < segments.len() {
                let candidate = segments[idx + 1];
                if is_valid_spotify_id(candidate) {
                    return Some(candidate.to_string());
                }
            }
        }
        return None;
    }

    if is_valid_spotify_id(trimmed) {
        return Some(trimmed.to_string());
    }

    None
}

fn is_valid_spotify_id(s: &str) -> bool {
    s.len() == 22 && s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Generates a CDN image URL from a binary Spotify image ID.
pub fn image_url(file_id: &[u8]) -> Option<String> {
    if file_id.is_empty() {
        return None;
    }
    let mut hex = String::with_capacity(file_id.len() * 2);
    for byte in file_id {
        let _ = write!(hex, "{byte:02x}");
    }
    Some(format!("{IMAGE_CDN}{hex}"))
}

/// Resolves the best available cover image URL from playlist attributes.
pub fn playlist_cover(attributes: &ListAttributes) -> Option<String> {
    for target in BY_SIZE {
        if let Some(size) = attributes
            .picture_size
            .iter()
            .find(|size| size.target_name() == target)
            .filter(|size| size.url().starts_with("http"))
        {
            return Some(size.url().to_owned());
        }
    }

    attributes
        .picture_size
        .first()
        .map(|size| size.url())
        .filter(|url| url.starts_with("http"))
        .map(str::to_owned)
        .or_else(|| image_url(attributes.picture()))
}

/// Maps a Protobuf TrackMessage to the core::Track domain model.
pub fn track_from_message(uri: &str, track: &TrackMessage) -> Track {
    let artists: Vec<String> = track
        .artist
        .iter()
        .filter_map(|a| {
            let name = a.name().trim();
            if !name.is_empty() {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect();

    let album = track.album.as_ref().and_then(|alb| {
        let name = alb.name().trim();
        if !name.is_empty() {
            Some(name.to_string())
        } else {
            None
        }
    });

    let duration_ms = track.duration.unwrap_or(0).max(0) as u64;

    let spotify_id = uri
        .strip_prefix(TRACK_PREFIX)
        .map(str::to_owned)
        .or_else(|| SpotifyId::from_raw(track.gid()).ok().map(|id| id.to_base62()));

    let isrc = track
        .external_id
        .iter()
        .find(|ext| ext.type_().eq_ignore_ascii_case("isrc"))
        .map(|ext| ext.id().trim().to_string())
        .filter(|id| !id.is_empty());

    let release_date = track.album.as_ref().and_then(|album| {
        album.date.as_ref().and_then(|d| {
            let year = d.year.unwrap_or(0);
            let month = d.month.unwrap_or(0);
            let day = d.day.unwrap_or(0);
            if year > 0 && month > 0 && day > 0 {
                Some(format!("{:04}-{:02}-{:02}", year, month, day))
            } else if year > 0 && month > 0 {
                Some(format!("{:04}-{:02}", year, month))
            } else if year > 0 {
                Some(format!("{:04}", year))
            } else {
                None
            }
        })
    });

    let title = if track.name().trim().is_empty() {
        "Unknown Track".to_string()
    } else {
        track.name().trim().to_string()
    };

    Track {
        title,
        artists: if artists.is_empty() {
            vec!["Unknown Artist".to_string()]
        } else {
            artists
        },
        album,
        duration_ms,
        path: None,
        spotify_id,
        isrc,
        release_date,
        file_size: None,
    }
}

/// High-speed playlist fetcher.
/// Checks the SQLite CacheStorage first for an instant cache hit (<5ms).
/// Otherwise, queries Spotify using the binary Protobuf protocol with 500-track batches
/// and saves the resulting snapshot into SQLite.
pub async fn fetch_playlist_fast(
    session: &Session,
    playlist_id: &str,
    storage: Option<&CacheStorage>,
) -> Result<Playlist> {
    fetch_playlist(session, playlist_id, storage, false).await
}

/// Fetches a playlist with an optional force-refresh flag to bypass cache.
pub async fn fetch_playlist(
    session: &Session,
    playlist_id: &str,
    storage: Option<&CacheStorage>,
    force_refresh: bool,
) -> Result<Playlist> {
    let clean_id = extract_playlist_id(playlist_id)
        .or_else(|| extract_spotify_id(playlist_id))
        .unwrap_or_else(|| playlist_id.trim().to_string());

    let cache_key = format!("spotify:playlist:{clean_id}");

    if !force_refresh {
        if let Some(storage) = storage {
            if let Ok(Some(cached_json)) = storage.get_snapshot(&cache_key) {
                if let Ok(playlist) = serde_json::from_str::<Playlist>(&cached_json) {
                    log::debug!("Cache hit: loaded playlist {} from SQLite", clean_id);
                    return Ok(playlist);
                } else {
                    log::warn!(
                        "Cache corrupted for playlist {}, fetching from Spotify",
                        clean_id
                    );
                }
            }
        }
    }

    let spotify_id = SpotifyId::from_base62(&clean_id)
        .with_context(|| format!("Invalid Spotify playlist ID: '{clean_id}'"))?;

    let body = session
        .spclient()
        .get_playlist(&spotify_id)
        .await
        .with_context(|| format!("Failed to fetch playlist '{clean_id}' from Spotify"))?;

    let content = SelectedListContent::parse_from_bytes(&body)
        .context("Failed to parse playlist SelectedListContent protobuf")?;

    let name = content.attributes.name().trim().to_string();
    let name = if name.is_empty() {
        "Untitled Playlist".to_string()
    } else {
        name
    };

    let description = {
        let desc = content.attributes.description().trim().to_string();
        if desc.is_empty() {
            None
        } else {
            Some(desc)
        }
    };

    let cover_url = playlist_cover(&content.attributes);

    let uris: Vec<String> = content
        .contents
        .items
        .iter()
        .map(|item| item.uri().to_owned())
        .filter(|uri| uri.starts_with(TRACK_PREFIX))
        .collect();

    let mut tracks_by_uri = HashMap::new();

    for chunk in uris.chunks(BATCH_SIZE) {
        let request = BatchedEntityRequest {
            entity_request: chunk
                .iter()
                .map(|uri| EntityRequest {
                    entity_uri: uri.clone(),
                    query: vec![ExtensionQuery {
                        extension_kind: EnumOrUnknown::new(ExtensionKind::TRACK_V4),
                        ..Default::default()
                    }],
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };

        let response = session
            .spclient()
            .get_extended_metadata(request)
            .await
            .with_context(|| {
                format!("Failed to fetch extended track metadata batch for playlist '{clean_id}'")
            })?;

        for ext_meta in response.extended_metadata {
            for entity in ext_meta.extension_data {
                if let Ok(msg) = TrackMessage::parse_from_bytes(&entity.extension_data.value) {
                    let track = track_from_message(&entity.entity_uri, &msg);
                    tracks_by_uri.insert(entity.entity_uri, track);
                }
            }
        }
    }

    let tracks: Vec<Track> = uris
        .iter()
        .filter_map(|uri| tracks_by_uri.get(uri).cloned())
        .collect();

    let playlist = Playlist {
        id: clean_id.clone(),
        name,
        description,
        tracks,
        cover_url,
    };

    if let Some(storage) = storage {
        if let Ok(json) = serde_json::to_string(&playlist) {
            if let Err(e) = storage.set_snapshot(&cache_key, &json) {
                log::warn!("Failed to store playlist snapshot in cache: {e}");
            }
        }
    }

    Ok(playlist)
}

#[cfg(test)]
mod tests {
    use super::*;
    use librespot_protocol::metadata::{
        Album as AlbumMessage, Artist as ArtistMessage, Date as DateMessage,
        ExternalId as ExternalIdMessage,
    };

    #[test]
    fn test_extract_spotify_id() {
        let raw = "37i9dQZF1DXcBWIGoYBM5M";
        assert_eq!(extract_spotify_id(raw).as_deref(), Some(raw));

        let uri = "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M";
        assert_eq!(extract_spotify_id(uri).as_deref(), Some(raw));

        let url = "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=e2f3d4c5";
        assert_eq!(extract_spotify_id(url).as_deref(), Some(raw));

        let intl_url = "https://open.spotify.com/intl-es/playlist/37i9dQZF1DXcBWIGoYBM5M?si=123";
        assert_eq!(extract_spotify_id(intl_url).as_deref(), Some(raw));

        let track_uri = "spotify:track:4cOdK2wGLETKBW3PvgPWqT";
        assert_eq!(
            extract_spotify_id(track_uri).as_deref(),
            Some("4cOdK2wGLETKBW3PvgPWqT")
        );

        let track_url = "https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT?si=abc";
        assert_eq!(
            extract_spotify_id(track_url).as_deref(),
            Some("4cOdK2wGLETKBW3PvgPWqT")
        );

        assert_eq!(extract_spotify_id(""), None);
        assert_eq!(extract_spotify_id("invalid-id-that-is-not-22-chars"), None);
    }

    #[test]
    fn test_extract_playlist_id_filters_tracks() {
        let raw = "37i9dQZF1DXcBWIGoYBM5M";
        assert_eq!(extract_playlist_id(raw).as_deref(), Some(raw));

        let playlist_url = "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=xyz";
        assert_eq!(extract_playlist_id(playlist_url).as_deref(), Some(raw));

        let playlist_uri = "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M";
        assert_eq!(extract_playlist_id(playlist_uri).as_deref(), Some(raw));

        let track_url = "https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT?si=abc";
        assert_eq!(extract_playlist_id(track_url), None);

        let track_uri = "spotify:track:4cOdK2wGLETKBW3PvgPWqT";
        assert_eq!(extract_playlist_id(track_uri), None);
    }

    #[test]
    fn test_track_from_message_mapping() {
        let mut msg = TrackMessage::new();
        msg.name = Some("Bohemian Rhapsody".to_string());
        msg.duration = Some(354000);

        let mut artist = ArtistMessage::new();
        artist.name = Some("Queen".to_string());
        msg.artist.push(artist);

        let mut album = AlbumMessage::new();
        album.name = Some("A Night at the Opera".to_string());
        let mut date = DateMessage::new();
        date.year = Some(1975);
        date.month = Some(11);
        date.day = Some(21);
        album.date = protobuf::MessageField::some(date);
        msg.album = protobuf::MessageField::some(album);

        let mut ext_id = ExternalIdMessage::new();
        ext_id.set_type("isrc".to_string());
        ext_id.set_id("GBUM71029604".to_string());
        msg.external_id.push(ext_id);

        let track = track_from_message("spotify:track:4cOdK2wGLETKBW3PvgPWqT", &msg);

        assert_eq!(track.title, "Bohemian Rhapsody");
        assert_eq!(track.artists, vec!["Queen".to_string()]);
        assert_eq!(track.album, Some("A Night at the Opera".to_string()));
        assert_eq!(track.duration_ms, 354000);
        assert_eq!(track.spotify_id, Some("4cOdK2wGLETKBW3PvgPWqT".to_string()));
        assert_eq!(track.isrc, Some("GBUM71029604".to_string()));
        assert_eq!(track.release_date, Some("1975-11-21".to_string()));
    }

    #[test]
    fn test_sqlite_snapshot_cache_speed_and_fidelity() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let db_path = temp_dir.path().join("cache.sqlite");
        let storage = CacheStorage::new(&db_path);

        let playlist_id = "37i9dQZF1DXcBWIGoYBM5M";
        let cache_key = format!("spotify:playlist:{playlist_id}");

        let mut playlist = Playlist::new(playlist_id, "Today's Top Hits");
        playlist.description = Some("The hottest 50 songs right now.".to_string());
        playlist.cover_url = Some("https://i.scdn.co/image/abc123cover".to_string());

        for i in 1..=50 {
            let mut track = Track::new(
                format!("Hit Song {i}"),
                vec![format!("Artist {i}"), "Featured Artist".into()],
                200_000 + i * 1_000,
            );
            track.album = Some(format!("Album {i}"));
            track.spotify_id = Some(format!("track_{i:022}"));
            track.isrc = Some(format!("USRC{i:08}"));
            track.release_date = Some("2024-05-10".to_string());
            playlist.tracks.push(track);
        }

        assert_eq!(playlist.track_count(), 50);

        let json = serde_json::to_string(&playlist).expect("serialization failed");
        storage
            .set_snapshot(&cache_key, &json)
            .expect("set_snapshot failed");

        let start = std::time::Instant::now();
        let cached_json = storage
            .get_snapshot(&cache_key)
            .expect("get_snapshot failed")
            .expect("snapshot not found");
        let loaded: Playlist =
            serde_json::from_str(&cached_json).expect("deserialization failed");
        let duration = start.elapsed();

        assert_eq!(playlist, loaded);
        assert!(
            duration < std::time::Duration::from_millis(50),
            "Cache load took {:?}, expected fast load (<50ms in debug test)",
            duration
        );
    }
}
