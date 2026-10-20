use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub status: DownloadStatus,
    pub progress_percent: f32,
    pub eta: Option<String>,
    pub speed: Option<String>,
}

impl DownloadItem {
    pub fn new(id: impl Into<String>, title: impl Into<String>, artist: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            artist: artist.into(),
            status: DownloadStatus::Queued,
            progress_percent: 0.0,
            eta: None,
            speed: None,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, DownloadStatus::Queued | DownloadStatus::Downloading)
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            DownloadStatus::Completed | DownloadStatus::Failed | DownloadStatus::Cancelled
        )
    }
}
