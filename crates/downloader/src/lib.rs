pub mod detector;
pub mod parser;
pub mod queue;
pub mod tagger;

pub use detector::{
    check_ffmpeg_available, check_ytdlp_available, find_ffmpeg_binary, find_ytdlp_binary,
    get_ytdlp_download_url, get_ytdlp_filename, YtDlpDetector,
};
pub use parser::{parse_ytdlp_line, sanitize_filename, ProgressState, ProgressUpdate};
pub use queue::{
    DownloadConfig, DownloadManager, DownloadProgress, DownloadQueue, DownloadRequest,
    DownloadStage,
};
pub use tagger::write_audio_tags;
