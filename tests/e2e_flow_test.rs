//! Complete End-to-End Integration Flow Test for Music Tools v2.0
//!
//! Validates the full application pipeline across all domain crates:
//! 1. Fetching/loading playlist (simulated Spotify Protobuf/REST metadata).
//! 2. SQLite cache storage and retrieval (storage::CacheStorage).
//! 3. Scanning a simulated local audio folder (scanner::scan_directory).
//! 4. Comparing playlist against local files with Sørensen-Dice fuzzy matching (matcher::compare_tracks).
//! 5. Enqueueing missing tracks in the download manager (downloader::DownloadManager) with simulated progress transitions.
//! 6. Synchronizing the final state into views models (views::WorkspaceView) and testing keyboard shortcuts and Escape handling.

use std::fs;

use core::{DownloadStatus, Playlist, Track};
use downloader::{DownloadConfig, DownloadManager, DownloadRequest, DownloadStage};
use matcher::{compare_tracks, DEFAULT_THRESHOLD};
use scanner::scan_directory;
use storage::CacheStorage;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use ui::tabs::AppTab;
use views::WorkspaceView;

#[test]
fn test_full_e2e_music_tools_flow() {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        // Setup isolated temporary directories for test
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let base_path = temp_dir.path();
        let cache_db_path = base_path.join("cache.sqlite");
        let music_dir_path = base_path.join("Music");
        let download_dir_path = base_path.join("Downloads");

        fs::create_dir_all(&music_dir_path).expect("Failed to create music directory");
        fs::create_dir_all(&download_dir_path).expect("Failed to create download directory");

        // =========================================================================
        // STEP 1: Spotify Playlist Loading / Simulation
        // =========================================================================
        let playlist_tracks = vec![
            Track::new("Bohemian Rhapsody", vec!["Queen".into()], 354_000)
                .with_album("A Night at the Opera")
                .with_spotify_id("spotify:track:4u7EnebtmKWzUH433cf5Qv"),
            Track::new("Stairway To Heaven", vec!["Led Zeppelin".into()], 482_000)
                .with_album("Led Zeppelin IV")
                .with_spotify_id("spotify:track:5CQ30WqJwcep0pYcV4AMNc"),
            Track::new("Hotel California", vec!["Eagles".into()], 391_000)
                .with_album("Hotel California")
                .with_spotify_id("spotify:track:40riOy7x9W7FXImm9G2AcA"),
            Track::new("Comfortably Numb", vec!["Pink Floyd".into()], 382_000)
                .with_album("The Wall")
                .with_spotify_id("spotify:track:7rPekCSQTZbDRSZ228eB0z"),
            Track::new("Sweet Child O' Mine", vec!["Guns N' Roses".into()], 356_000)
                .with_album("Appetite for Destruction")
                .with_spotify_id("spotify:track:7o2CTH4ctstm8TNelqjb51"),
        ];

        let original_playlist = Playlist::new(
            "rock_classics",
            "Rock Classics Curated",
        )
        .with_tracks(playlist_tracks.clone());

        assert_eq!(original_playlist.tracks.len(), 5);
        assert_eq!(original_playlist.name, "Rock Classics Curated");

        // =========================================================================
        // STEP 2: SQLite Cache Storage & Retrieval
        // =========================================================================
        let storage = CacheStorage::new(&cache_db_path);
        let serialized_playlist = serde_json::to_string(&original_playlist)
            .expect("Failed to serialize playlist to JSON");

        // Store snapshot in SQLite database
        let cache_key = format!("playlist:spotify:{}", original_playlist.id);
        storage
            .set_snapshot(&cache_key, &serialized_playlist)
            .expect("Failed to persist snapshot in SQLite");

        // Also persist user settings
        storage
            .set_setting(storage::SETTING_FUZZY_THRESHOLD, "0.80")
            .expect("Failed to save fuzzy threshold setting");
        storage
            .set_setting(
                storage::SETTING_DOWNLOAD_DIR,
                download_dir_path.to_str().unwrap(),
            )
            .expect("Failed to save download dir setting");

        // Retrieve from SQLite and verify data integrity
        let cached_json = storage
            .get_snapshot(&cache_key)
            .expect("Failed to query snapshot from SQLite")
            .expect("Snapshot was not found in SQLite");

        let retrieved_playlist: Playlist = serde_json::from_str(&cached_json)
            .expect("Failed to deserialize cached playlist");

        assert_eq!(retrieved_playlist.id, original_playlist.id);
        assert_eq!(retrieved_playlist.name, original_playlist.name);
        assert_eq!(retrieved_playlist.tracks.len(), 5);
        assert_eq!(retrieved_playlist.tracks[0].title, "Bohemian Rhapsody");

        // =========================================================================
        // STEP 3: Local Simulated Music Folder Scanning
        // =========================================================================
        // We simulate a local collection on disk:
        // - "Queen - Bohemian Rhapsody.mp3" -> Exact match
        // - "Led Zeppelin - Stairway To Heaven (Remastered).mp3" -> Probable fuzzy match
        // - "AC DC - Back In Black.mp3" -> Extra local song, not in playlist
        // Note: Hotel California, Comfortably Numb, Sweet Child O' Mine are missing locally!
        let local_files = [
            "Queen - Bohemian Rhapsody.mp3",
            "Led Zeppelin - Stairway To Heaven (Remastered).mp3",
            "AC DC - Back In Black.mp3",
        ];

        for filename in &local_files {
            let file_path = music_dir_path.join(filename);
            fs::write(&file_path, b"mock audio content bytes").expect("Failed to write mock audio file");
        }

        let scanned_tracks = scan_directory(&music_dir_path)
            .expect("Failed to scan simulated music directory");

        assert_eq!(scanned_tracks.len(), 3);
        let scanned_titles: Vec<&str> = scanned_tracks.iter().map(|t| t.title.as_str()).collect();
        assert!(scanned_titles.contains(&"Bohemian Rhapsody"));
        assert!(scanned_titles.contains(&"Back In Black"));

        // =========================================================================
        // STEP 4: Fuzzy Comparison with Sørensen-Dice Accuracy
        // =========================================================================
        let comp_result = compare_tracks(
            &retrieved_playlist.tracks,
            &scanned_tracks,
            DEFAULT_THRESHOLD,
        );

        // 1 exact match (Bohemian Rhapsody)
        // 1 probable/exact match (Stairway To Heaven with Remastered normalized away)
        // 3 missing tracks (Hotel California, Comfortably Numb, Sweet Child O' Mine)
        // 1 extra track (Back In Black)
        let total_matches = comp_result.exact_matches.len() + comp_result.probable_matches.len();
        assert_eq!(total_matches, 2, "Expected 2 matched tracks");
        assert_eq!(comp_result.missing.len(), 3, "Expected 3 missing tracks");
        assert_eq!(comp_result.extra.len(), 1, "Expected 1 extra local track");

        let missing_titles: Vec<&str> = comp_result.missing.iter().map(|t| t.title.as_str()).collect();
        assert!(missing_titles.contains(&"Hotel California"));
        assert!(missing_titles.contains(&"Comfortably Numb"));
        assert!(missing_titles.contains(&"Sweet Child O' Mine"));

        // =========================================================================
        // STEP 5: Download Missing Songs with DownloadManager
        // =========================================================================
        let download_config = DownloadConfig::new(&download_dir_path)
            .with_max_concurrent(3)
            .with_dry_run(true);

        let (download_manager, mut progress_rx) = DownloadManager::new(download_config);

        // Enqueue missing tracks into the download manager
        for missing_track in &comp_result.missing {
            let req = DownloadRequest::new(
                format!("req-{}", missing_track.title.to_lowercase().replace(' ', "-")),
                &missing_track.title,
                missing_track.artists.clone(),
            );
            download_manager
                .enqueue(req)
                .await
                .expect("Failed to enqueue download request");
        }

        // Receive download progress updates and verify state transitions
        let mut completed_ids = Vec::new();
        let mut saw_searching_or_downloading = false;

        while let Some(progress) = progress_rx.recv().await {
            if progress.stage == DownloadStage::Searching || progress.stage == DownloadStage::Downloading {
                saw_searching_or_downloading = true;
            }
            if progress.stage == DownloadStage::Completed {
                assert_eq!(progress.status, DownloadStatus::Completed);
                assert_eq!(progress.percent, 100.0);
                if !completed_ids.contains(&progress.id) {
                    completed_ids.push(progress.id.clone());
                }
                if completed_ids.len() == comp_result.missing.len() {
                    break;
                }
            }
        }

        assert!(saw_searching_or_downloading, "Expected download progress stages to be observed");
        assert_eq!(completed_ids.len(), 3, "All 3 missing tracks must complete downloading");

        // Verify items in manager store
        for id in &completed_ids {
            let item = download_manager.get_item(id).await.expect("Item must exist in manager");
            assert_eq!(item.status, DownloadStatus::Completed);
            assert_eq!(item.progress_percent, 100.0);
        }

        // =========================================================================
        // STEP 6: Sync Final State in Views Models & Test Keyboard Navigation
        // =========================================================================
        let mut workspace = WorkspaceView::new();

        // Populate staging and compare views
        workspace.staging.tracks = retrieved_playlist.tracks.clone();
        assert_eq!(workspace.staging.track_count(), 5);

        workspace.compare.source_tracks = retrieved_playlist.tracks.clone();
        workspace.compare.target_tracks = scanned_tracks.clone();
        workspace.compare.compare();
        assert_eq!(workspace.compare.missing_count(), 3);

        // Send missing tracks to downloads queue in views
        let sent_to_dl = workspace.send_compare_missing_to_downloads();
        assert_eq!(sent_to_dl, 3);
        assert_eq!(workspace.downloads.total_count(), 3);

        // Mark all downloads as completed using update_progress
        for item in &workspace.downloads.items.clone() {
            workspace.downloads.update_progress(&item.id, 100.0, None, None);
        }
        assert_eq!(workspace.downloads.completed_count(), 3);
        assert_eq!(workspace.downloads.downloading_count(), 0);

        // Test global view switching shortcuts
        assert_eq!(workspace.active_tab, AppTab::Staging);

        workspace.select_compare();
        assert_eq!(workspace.active_tab, AppTab::Compare);

        workspace.select_downloads();
        assert_eq!(workspace.active_tab, AppTab::Downloads);

        workspace.select_utilities();
        assert_eq!(workspace.active_tab, AppTab::Utilities);

        workspace.select_settings();
        assert_eq!(workspace.active_tab, AppTab::Settings);

        workspace.select_staging();
        assert_eq!(workspace.active_tab, AppTab::Staging);

        // Test Escape key handling:
        // Case A: With table selection active in staging, Escape clears it
        workspace.staging.selection.select_range(0, 1);
        assert_eq!(workspace.staging.selection.count(), 2);
        let handled_esc = workspace.handle_escape();
        assert!(handled_esc);
        assert!(workspace.staging.selection.is_empty());

        // Case B: With deletion modal open in utilities, Escape cancels it
        workspace.select_utilities();
        workspace.utilities.prompt_delete(music_dir_path.join("Queen - Bohemian Rhapsody.mp3"));
        assert!(workspace.utilities.pending_delete_path.is_some());
        let handled_modal_esc = workspace.handle_escape();
        assert!(handled_modal_esc);
        assert!(workspace.utilities.pending_delete_path.is_none());

        // Case C: When idle with no selection or modal, Escape returns false
        assert!(!workspace.handle_escape());
    });
}
