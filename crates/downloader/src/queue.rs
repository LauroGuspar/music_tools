use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use core::{DownloadItem, DownloadStatus};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot, RwLock, Semaphore};

use crate::detector::{clean_env_tokio, YtDlpDetector};
use crate::parser::{parse_ytdlp_line, sanitize_filename, ProgressState};
use crate::tagger::write_audio_tags;

#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub download_dir: PathBuf,
    pub max_concurrent_downloads: usize,
    pub custom_ytdlp_path: Option<PathBuf>,
    pub audio_format: String,
    pub audio_quality: String,
    pub dry_run: bool,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            download_dir: std::env::temp_dir().join("music_tools_downloads"),
            max_concurrent_downloads: storage::DEFAULT_CONCURRENT_DOWNLOADS,
            custom_ytdlp_path: None,
            audio_format: "mp3".to_string(),
            audio_quality: "0".to_string(),
            dry_run: false,
        }
    }
}

impl DownloadConfig {
    pub fn new(download_dir: impl Into<PathBuf>) -> Self {
        Self {
            download_dir: download_dir.into(),
            ..Default::default()
        }
    }

    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent_downloads = max.max(1);
        self
    }

    pub fn with_audio_format(mut self, format: impl Into<String>) -> Self {
        self.audio_format = format.into();
        self
    }

    pub fn with_audio_quality(mut self, quality: impl Into<String>) -> Self {
        self.audio_quality = quality.into();
        self
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn from_storage(cache: &storage::CacheStorage) -> Result<Self> {
        let max_concurrent = cache
            .get_setting(storage::SETTING_CONCURRENT_DOWNLOADS)?
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(storage::DEFAULT_CONCURRENT_DOWNLOADS);

        let download_dir = cache
            .get_setting(storage::SETTING_DOWNLOAD_DIR)?
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::var_os("XDG_MUSIC_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| std::env::temp_dir().join("music_tools_downloads"))
            });

        Ok(Self {
            download_dir,
            max_concurrent_downloads: max_concurrent,
            custom_ytdlp_path: None,
            audio_format: "mp3".to_string(),
            audio_quality: "0".to_string(),
            dry_run: false,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadRequest {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub year: Option<u32>,
    pub cover_url: Option<String>,
    pub custom_url: Option<String>,
    pub output_dir: Option<PathBuf>,
}

impl DownloadRequest {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        artists: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            artists,
            album: None,
            year: None,
            cover_url: None,
            custom_url: None,
            output_dir: None,
        }
    }

    pub fn with_album(mut self, album: impl Into<String>) -> Self {
        self.album = Some(album.into());
        self
    }

    pub fn with_year(mut self, year: u32) -> Self {
        self.year = Some(year);
        self
    }

    pub fn with_cover_url(mut self, url: impl Into<String>) -> Self {
        self.cover_url = Some(url.into());
        self
    }

    pub fn with_custom_url(mut self, url: impl Into<String>) -> Self {
        self.custom_url = Some(url.into());
        self
    }

    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = Some(dir.into());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStage {
    Queued,
    Searching,
    Downloading,
    Converting,
    Tagging,
    Completed,
    SkippedExisting,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadProgress {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub status: DownloadStatus,
    pub stage: DownloadStage,
    pub percent: f32,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub message: Option<String>,
    pub file_path: Option<PathBuf>,
}

impl DownloadProgress {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        artist: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            artist: artist.into(),
            status: DownloadStatus::Queued,
            stage: DownloadStage::Queued,
            percent: 0.0,
            speed: None,
            eta: None,
            message: None,
            file_path: None,
        }
    }

    pub fn to_item(&self) -> DownloadItem {
        DownloadItem {
            id: self.id.clone(),
            title: self.title.clone(),
            artist: self.artist.clone(),
            status: self.status,
            progress_percent: self.percent,
            eta: self.eta.clone(),
            speed: self.speed.clone(),
        }
    }
}

pub type DownloadQueue = DownloadManager;

#[derive(Clone)]
pub struct DownloadManager {
    config: Arc<RwLock<DownloadConfig>>,
    semaphore: Arc<Semaphore>,
    progress_tx: mpsc::Sender<DownloadProgress>,
    items: Arc<RwLock<HashMap<String, DownloadItem>>>,
    detector: YtDlpDetector,
    http_client: reqwest::Client,
    cancellations: Arc<RwLock<HashMap<String, oneshot::Sender<()>>>>,
}

impl DownloadManager {
    pub fn new(config: DownloadConfig) -> (Self, mpsc::Receiver<DownloadProgress>) {
        let (tx, rx) = mpsc::channel(128);
        (Self::with_channel(config, tx), rx)
    }

    pub fn with_channel(
        config: DownloadConfig,
        progress_tx: mpsc::Sender<DownloadProgress>,
    ) -> Self {
        let max_concurrent = config.max_concurrent_downloads;
        let detector = YtDlpDetector::new(config.custom_ytdlp_path.clone());
        let http_client = reqwest::Client::builder()
            .user_agent("music-tools-downloader")
            .build()
            .unwrap_or_default();

        Self {
            config: Arc::new(RwLock::new(config)),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            progress_tx,
            items: Arc::new(RwLock::new(HashMap::new())),
            detector,
            http_client,
            cancellations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn detector(&self) -> &YtDlpDetector {
        &self.detector
    }

    pub async fn config(&self) -> DownloadConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config<F>(&self, f: F)
    where
        F: FnOnce(&mut DownloadConfig),
    {
        let mut cfg = self.config.write().await;
        f(&mut cfg);
    }

    pub async fn get_item(&self, id: &str) -> Option<DownloadItem> {
        let items = self.items.read().await;
        items.get(id).cloned()
    }

    pub async fn list_items(&self) -> Vec<DownloadItem> {
        let items = self.items.read().await;
        items.values().cloned().collect()
    }

    pub async fn active_count(&self) -> usize {
        let items = self.items.read().await;
        items.values().filter(|it| it.is_active()).count()
    }

    pub async fn cancel(&self, id: &str) -> bool {
        let mut cancels = self.cancellations.write().await;
        if let Some(cancel_tx) = cancels.remove(id) {
            let _ = cancel_tx.send(());

            let mut items = self.items.write().await;
            if let Some(item) = items.get_mut(id) {
                item.status = DownloadStatus::Cancelled;
                let _ = self.progress_tx.send(DownloadProgress {
                    id: item.id.clone(),
                    title: item.title.clone(),
                    artist: item.artist.clone(),
                    status: DownloadStatus::Cancelled,
                    stage: DownloadStage::Cancelled,
                    percent: item.progress_percent,
                    speed: None,
                    eta: None,
                    message: Some("Cancelled by user".to_string()),
                    file_path: None,
                }).await;
            }
            return true;
        }
        false
    }

    pub async fn enqueue(&self, request: DownloadRequest) -> Result<()> {
        let item = DownloadItem::new(
            &request.id,
            &request.title,
            &request.artist_display(),
        );

        {
            let mut items = self.items.write().await;
            items.insert(request.id.clone(), item.clone());
        }

        let (cancel_tx, cancel_rx) = oneshot::channel();
        {
            let mut cancels = self.cancellations.write().await;
            cancels.insert(request.id.clone(), cancel_tx);
        }

        let _ = self.progress_tx.send(DownloadProgress {
            id: request.id.clone(),
            title: request.title.clone(),
            artist: request.artist_display(),
            status: DownloadStatus::Queued,
            stage: DownloadStage::Queued,
            percent: 0.0,
            speed: None,
            eta: None,
            message: None,
            file_path: None,
        }).await;

        let manager = self.clone();
        tokio::spawn(async move {
            manager.process_download(request, cancel_rx).await;
        });

        Ok(())
    }

    pub async fn enqueue_batch(&self, requests: Vec<DownloadRequest>) -> Result<()> {
        for req in requests {
            self.enqueue(req).await?;
        }
        Ok(())
    }

    async fn process_download(
        &self,
        request: DownloadRequest,
        mut cancel_rx: oneshot::Receiver<()>,
    ) {
        let config = self.config().await;
        let output_dir = request
            .output_dir
            .clone()
            .unwrap_or_else(|| config.download_dir.clone());

        if let Err(e) = std::fs::create_dir_all(&output_dir) {
            self.emit_failure(&request, format!("Failed to create download dir: {e}"))
                .await;
            return;
        }

        let clean_artists = sanitize_filename(&request.artist_display());
        let clean_title = sanitize_filename(&request.title);
        let final_filename = format!(
            "{} - {}.{}",
            clean_artists, clean_title, config.audio_format
        );
        let target_path = output_dir.join(&final_filename);

        // Verification of skip if file already exists on disk (skipped_existing)
        if target_path.exists() {
            info!("File already exists on disk, skipping: {:?}", target_path);
            {
                let mut items = self.items.write().await;
                if let Some(item) = items.get_mut(&request.id) {
                    item.status = DownloadStatus::Completed;
                    item.progress_percent = 100.0;
                }
            }

            let _ = self.progress_tx.send(DownloadProgress {
                id: request.id.clone(),
                title: request.title.clone(),
                artist: request.artist_display(),
                status: DownloadStatus::Completed,
                stage: DownloadStage::SkippedExisting,
                percent: 100.0,
                speed: None,
                eta: None,
                message: Some("skipped_existing".to_string()),
                file_path: Some(target_path),
            }).await;

            let mut cancels = self.cancellations.write().await;
            cancels.remove(&request.id);
            return;
        }

        // Check if cancelled before acquiring semaphore
        if cancel_rx.try_recv().is_ok() {
            return;
        }

        // Acquire concurrency permit
        let permit = match self.semaphore.acquire().await {
            Ok(p) => p,
            Err(e) => {
                self.emit_failure(&request, format!("Semaphore closed: {e}"))
                    .await;
                return;
            }
        };

        if cancel_rx.try_recv().is_ok() {
            drop(permit);
            return;
        }

        // Set status to Downloading / Searching
        {
            let mut items = self.items.write().await;
            if let Some(item) = items.get_mut(&request.id) {
                item.status = DownloadStatus::Downloading;
            }
        }

        let _ = self.progress_tx.send(DownloadProgress {
            id: request.id.clone(),
            title: request.title.clone(),
            artist: request.artist_display(),
            status: DownloadStatus::Downloading,
            stage: DownloadStage::Searching,
            percent: 0.0,
            speed: None,
            eta: None,
            message: Some("searching".to_string()),
            file_path: None,
        }).await;

        if config.dry_run {
            // Simulated execution for unit testing & dry runs
            let simulated_steps = [25.0, 50.0, 75.0, 100.0];
            for pct in simulated_steps {
                if cancel_rx.try_recv().is_ok() {
                    drop(permit);
                    return;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
                let _ = self.progress_tx.send(DownloadProgress {
                    id: request.id.clone(),
                    title: request.title.clone(),
                    artist: request.artist_display(),
                    status: DownloadStatus::Downloading,
                    stage: DownloadStage::Downloading,
                    percent: pct,
                    speed: Some("10.0MiB/s".to_string()),
                    eta: Some("00:01".to_string()),
                    message: None,
                    file_path: None,
                }).await;
            }

            let _ = self.progress_tx.send(DownloadProgress {
                id: request.id.clone(),
                title: request.title.clone(),
                artist: request.artist_display(),
                status: DownloadStatus::Downloading,
                stage: DownloadStage::Converting,
                percent: 100.0,
                speed: None,
                eta: None,
                message: Some("converting".to_string()),
                file_path: None,
            }).await;

            let _ = std::fs::write(&target_path, b"simulated audio content");

            drop(permit);

            {
                let mut items = self.items.write().await;
                if let Some(item) = items.get_mut(&request.id) {
                    item.status = DownloadStatus::Completed;
                    item.progress_percent = 100.0;
                }
            }

            let _ = self.progress_tx.send(DownloadProgress {
                id: request.id.clone(),
                title: request.title.clone(),
                artist: request.artist_display(),
                status: DownloadStatus::Completed,
                stage: DownloadStage::Completed,
                percent: 100.0,
                speed: None,
                eta: None,
                message: Some("completed".to_string()),
                file_path: Some(target_path),
            }).await;

            let mut cancels = self.cancellations.write().await;
            cancels.remove(&request.id);
            return;
        }

        // Live execution with yt-dlp
        let ytdlp_path = match self.detector.resolve_ytdlp() {
            Some(p) => p,
            None => {
                drop(permit);
                self.emit_failure(
                    &request,
                    "yt-dlp binary is not installed or available".to_string(),
                )
                .await;
                return;
            }
        };

        let query = if let Some(ref url) = request.custom_url {
            if !url.trim().is_empty() {
                url.trim().to_string()
            } else {
                format!("ytsearch1:{} - {}", request.artists.join(" "), request.title)
            }
        } else {
            format!("ytsearch1:{} - {}", request.artists.join(" "), request.title)
        };

        let output_template = format!(
            "{}/{} - {}.%(ext)s",
            output_dir.display(),
            clean_artists,
            clean_title
        );

        let mut cmd = tokio::process::Command::new(&ytdlp_path);
        clean_env_tokio(&mut cmd);
        cmd.arg("--extractor-args")
            .arg("youtube:player_client=android,web")
            .arg("--extract-audio")
            .arg("--audio-format")
            .arg(&config.audio_format)
            .arg("--audio-quality")
            .arg(&config.audio_quality)
            .arg("--newline")
            .arg(&query)
            .arg("-o")
            .arg(&output_template)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                drop(permit);
                self.emit_failure(&request, format!("Failed to spawn yt-dlp: {e}"))
                    .await;
                return;
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let stderr_task = tokio::spawn(async move {
            let mut err_msg = String::new();
            if let Some(err_pipe) = stderr {
                let reader = BufReader::new(err_pipe);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        err_msg = trimmed.to_string();
                    }
                }
            }
            err_msg
        });

        if let Some(stdout_pipe) = stdout {
            let reader = BufReader::new(stdout_pipe);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if cancel_rx.try_recv().is_ok() {
                    let _ = child.kill().await;
                    drop(permit);
                    return;
                }

                if let Some(update) = parse_ytdlp_line(&line) {
                    let (stage, percent) = match update.state {
                        ProgressState::Downloading => {
                            (DownloadStage::Downloading, update.percent)
                        }
                        ProgressState::Converting => (DownloadStage::Converting, 100.0),
                        ProgressState::Error(_) => (DownloadStage::Failed, 0.0),
                    };

                    {
                        let mut items = self.items.write().await;
                        if let Some(item) = items.get_mut(&request.id) {
                            item.progress_percent = percent;
                            item.speed = update.speed.clone();
                            item.eta = update.eta.clone();
                        }
                    }

                    let _ = self.progress_tx.send(DownloadProgress {
                        id: request.id.clone(),
                        title: request.title.clone(),
                        artist: request.artist_display(),
                        status: DownloadStatus::Downloading,
                        stage,
                        percent,
                        speed: update.speed,
                        eta: update.eta,
                        message: None,
                        file_path: None,
                    }).await;
                }
            }
        }

        let exit_status = match child.wait().await {
            Ok(s) => s,
            Err(e) => {
                drop(permit);
                self.emit_failure(&request, format!("Error waiting on yt-dlp: {e}"))
                    .await;
                return;
            }
        };

        let last_err = stderr_task.await.unwrap_or_default();

        if !exit_status.success() {
            drop(permit);
            let detail = if last_err.is_empty() {
                "yt-dlp process returned non-zero exit code".to_string()
            } else {
                format!("yt-dlp error: {}", last_err)
            };
            self.emit_failure(&request, detail).await;
            return;
        }

        // Post-processing tags and cover
        if target_path.exists() {
            let _ = self.progress_tx.send(DownloadProgress {
                id: request.id.clone(),
                title: request.title.clone(),
                artist: request.artist_display(),
                status: DownloadStatus::Downloading,
                stage: DownloadStage::Tagging,
                percent: 100.0,
                speed: None,
                eta: None,
                message: Some("tagging".to_string()),
                file_path: Some(target_path.clone()),
            }).await;

            let mut cover_data = None;
            if let Some(ref cover_url) = request.cover_url {
                if !cover_url.trim().is_empty() {
                    if let Ok(resp) = self.http_client.get(cover_url).send().await {
                        let content_type = resp
                            .headers()
                            .get("content-type")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("image/jpeg")
                            .to_string();
                        if let Ok(bytes) = resp.bytes().await {
                            cover_data = Some((bytes.to_vec(), content_type));
                        }
                    }
                }
            }

            let cover_ref = cover_data
                .as_ref()
                .map(|(b, m)| (b.as_slice(), m.as_str()));

            if let Err(e) = write_audio_tags(
                &target_path,
                &request.title,
                &request.artists,
                request.album.as_deref(),
                request.year,
                cover_ref,
            ) {
                warn!("Warning: Failed to write audio tags: {e}");
            }
        }

        drop(permit);

        {
            let mut items = self.items.write().await;
            if let Some(item) = items.get_mut(&request.id) {
                item.status = DownloadStatus::Completed;
                item.progress_percent = 100.0;
            }
        }

        let _ = self.progress_tx.send(DownloadProgress {
            id: request.id.clone(),
            title: request.title.clone(),
            artist: request.artist_display(),
            status: DownloadStatus::Completed,
            stage: DownloadStage::Completed,
            percent: 100.0,
            speed: None,
            eta: None,
            message: Some("completed".to_string()),
            file_path: Some(target_path),
        }).await;

        let mut cancels = self.cancellations.write().await;
        cancels.remove(&request.id);
    }

    async fn emit_failure(&self, request: &DownloadRequest, error_message: String) {
        error!("Download failed for {}: {}", request.id, error_message);
        {
            let mut items = self.items.write().await;
            if let Some(item) = items.get_mut(&request.id) {
                item.status = DownloadStatus::Failed;
            }
        }

        let _ = self.progress_tx.send(DownloadProgress {
            id: request.id.clone(),
            title: request.title.clone(),
            artist: request.artist_display(),
            status: DownloadStatus::Failed,
            stage: DownloadStage::Failed,
            percent: 0.0,
            speed: None,
            eta: None,
            message: Some(error_message),
            file_path: None,
        }).await;

        let mut cancels = self.cancellations.write().await;
        cancels.remove(&request.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tokio::runtime::Runtime;

    #[test]
    fn test_download_request_creation_and_artist_display() {
        let req = DownloadRequest::new("req-1", "Echoes", vec!["Pink Floyd".into()])
            .with_album("Meddle")
            .with_year(1971);

        assert_eq!(req.id, "req-1");
        assert_eq!(req.title, "Echoes");
        assert_eq!(req.artist_display(), "Pink Floyd");
        assert_eq!(req.album.as_deref(), Some("Meddle"));
        assert_eq!(req.year, Some(1971));

        let multiple_artists = DownloadRequest::new(
            "req-2",
            "Under Pressure",
            vec!["Queen".into(), "David Bowie".into()],
        );
        assert_eq!(multiple_artists.artist_display(), "Queen, David Bowie");
    }

    #[test]
    fn test_download_config_builder() {
        let temp_dir = std::env::temp_dir().join("test_dl_dir");
        let cfg = DownloadConfig::new(temp_dir.clone())
            .with_max_concurrent(5)
            .with_audio_format("flac")
            .with_dry_run(true);

        assert_eq!(cfg.download_dir, temp_dir);
        assert_eq!(cfg.max_concurrent_downloads, 5);
        assert_eq!(cfg.audio_format, "flac");
        assert!(cfg.dry_run);
    }

    #[test]
    fn test_download_manager_skipped_existing() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = std::env::temp_dir().join(format!("mt_skip_test_{}", std::process::id()));
            let _ = fs::create_dir_all(&temp_dir);

            let cfg = DownloadConfig::new(&temp_dir).with_dry_run(true);
            let (manager, mut rx) = DownloadManager::new(cfg);

            // Create the pre-existing file on disk
            let pre_existing = temp_dir.join("Led Zeppelin - Stairway to Heaven.mp3");
            let mut f = File::create(&pre_existing).unwrap();
            f.write_all(b"already downloaded audio").unwrap();

            let req = DownloadRequest::new(
                "req-skip",
                "Stairway to Heaven",
                vec!["Led Zeppelin".into()],
            );

            manager.enqueue(req).await.unwrap();

            // 1. First event: Queued
            let ev1 = rx.recv().await.expect("event 1");
            assert_eq!(ev1.stage, DownloadStage::Queued);
            assert_eq!(ev1.status, DownloadStatus::Queued);

            // 2. Second event: SkippedExisting
            let ev2 = rx.recv().await.expect("event 2");
            assert_eq!(ev2.stage, DownloadStage::SkippedExisting);
            assert_eq!(ev2.status, DownloadStatus::Completed);
            assert_eq!(ev2.percent, 100.0);
            assert_eq!(ev2.message.as_deref(), Some("skipped_existing"));
            assert_eq!(ev2.file_path, Some(pre_existing));

            let item = manager.get_item("req-skip").await.expect("item exists");
            assert_eq!(item.status, DownloadStatus::Completed);
            assert_eq!(item.progress_percent, 100.0);

            let _ = fs::remove_dir_all(&temp_dir);
        });
    }

    #[test]
    fn test_download_manager_dry_run_pipeline() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = std::env::temp_dir().join(format!("mt_dry_test_{}", std::process::id()));
            let _ = fs::create_dir_all(&temp_dir);

            let cfg = DownloadConfig::new(&temp_dir).with_dry_run(true);
            let (manager, mut rx) = DownloadManager::new(cfg);

            let req = DownloadRequest::new("req-pipe", "Time", vec!["Pink Floyd".into()]);
            manager.enqueue(req).await.unwrap();

            let mut stages_seen = Vec::new();
            while let Some(progress) = rx.recv().await {
                stages_seen.push(progress.stage);
                if progress.stage == DownloadStage::Completed {
                    break;
                }
            }

            assert!(stages_seen.contains(&DownloadStage::Queued));
            assert!(stages_seen.contains(&DownloadStage::Searching));
            assert!(stages_seen.contains(&DownloadStage::Downloading));
            assert!(stages_seen.contains(&DownloadStage::Converting));
            assert!(stages_seen.contains(&DownloadStage::Completed));

            let item = manager.get_item("req-pipe").await.expect("item exists");
            assert_eq!(item.status, DownloadStatus::Completed);
            assert_eq!(item.progress_percent, 100.0);

            let expected_file = temp_dir.join("Pink Floyd - Time.mp3");
            assert!(expected_file.exists());

            let _ = fs::remove_dir_all(&temp_dir);
        });
    }

    #[test]
    fn test_download_manager_concurrency_limit() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = std::env::temp_dir().join(format!("mt_conc_test_{}", std::process::id()));
            let _ = fs::create_dir_all(&temp_dir);

            let cfg = DownloadConfig::new(&temp_dir)
                .with_max_concurrent(2)
                .with_dry_run(true);
            let (manager, mut rx) = DownloadManager::new(cfg);

            for i in 1..=4 {
                let req = DownloadRequest::new(
                    format!("req-{}", i),
                    format!("Song {}", i),
                    vec!["Artist".into()],
                );
                manager.enqueue(req).await.unwrap();
            }

            // Collect all completed
            let mut completed_count = 0;
            while let Some(prog) = rx.recv().await {
                if prog.stage == DownloadStage::Completed {
                    completed_count += 1;
                    if completed_count == 4 {
                        break;
                    }
                }
            }

            assert_eq!(completed_count, 4);
            let items = manager.list_items().await;
            assert_eq!(items.len(), 4);
            for it in items {
                assert_eq!(it.status, DownloadStatus::Completed);
            }

            let _ = fs::remove_dir_all(&temp_dir);
        });
    }

    #[test]
    fn test_download_manager_cancellation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = std::env::temp_dir().join(format!("mt_cancel_test_{}", std::process::id()));
            let _ = fs::create_dir_all(&temp_dir);

            let cfg = DownloadConfig::new(&temp_dir).with_dry_run(true);
            let (manager, mut rx) = DownloadManager::new(cfg);

            let req = DownloadRequest::new("req-cancel", "Money", vec!["Pink Floyd".into()]);
            manager.enqueue(req).await.unwrap();

            // Immediately cancel
            let cancelled = manager.cancel("req-cancel").await;
            assert!(cancelled);

            // Receive events until cancelled
            let mut saw_cancelled = false;
            while let Some(prog) = rx.recv().await {
                if prog.stage == DownloadStage::Cancelled {
                    saw_cancelled = true;
                    break;
                }
            }

            assert!(saw_cancelled);
            let item = manager.get_item("req-cancel").await.expect("item exists");
            assert_eq!(item.status, DownloadStatus::Cancelled);

            let _ = fs::remove_dir_all(&temp_dir);
        });
    }
}
