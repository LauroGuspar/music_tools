pub mod download;
pub mod playlist;
pub mod track;

pub use download::{DownloadItem, DownloadStatus};
pub use playlist::Playlist;
pub use track::Track;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_creation_and_serialization() {
        let track = Track {
            title: "Song Name".into(),
            artists: vec!["Artist A".into(), "Artist B".into()],
            album: Some("Album Name".into()),
            duration_ms: 215000,
            path: Some("/music/song.mp3".into()),
            spotify_id: Some("spotify123".into()),
            isrc: Some("US1234567890".into()),
            release_date: Some("2024-01-01".into()),
        };

        assert_eq!(track.artist_display(), "Artist A, Artist B");

        let json = serde_json::to_string(&track).expect("serialization failed");
        let deserialized: Track = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(track, deserialized);
    }

    #[test]
    fn test_playlist_operations() {
        let mut playlist = Playlist::new("pl-1", "Favorites");
        assert_eq!(playlist.track_count(), 0);
        assert_eq!(playlist.total_duration_ms(), 0);

        playlist.tracks.push(Track::new("Track 1", vec!["Artist 1".into()], 120_000));
        playlist.tracks.push(Track::new("Track 2", vec!["Artist 2".into()], 180_000));

        assert_eq!(playlist.track_count(), 2);
        assert_eq!(playlist.total_duration_ms(), 300_000);
    }

    #[test]
    fn test_download_item_status() {
        let mut item = DownloadItem::new("dl-1", "Song Title", "Artist");
        assert_eq!(item.status, DownloadStatus::Queued);
        assert!(item.is_active());
        assert!(!item.is_finished());

        item.status = DownloadStatus::Downloading;
        item.progress_percent = 45.5;
        assert!(item.is_active());

        item.status = DownloadStatus::Completed;
        item.progress_percent = 100.0;
        assert!(!item.is_active());
        assert!(item.is_finished());
    }
}
