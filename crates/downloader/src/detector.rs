use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

pub fn get_ytdlp_filename() -> &'static str {
    if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "yt-dlp_macos"
    } else {
        "yt-dlp"
    }
}

pub fn get_ytdlp_download_url() -> &'static str {
    if cfg!(target_os = "windows") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    }
}

pub fn clean_env_std(cmd: &mut Command) {
    cmd.env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONSTARTUP")
        .env_remove("PYTHONOPTIMIZE")
        .env_remove("PYTHONEXECUTABLE")
        .env_remove("PYTHONNOUSERSITE");
}

pub fn clean_env_tokio(cmd: &mut tokio::process::Command) {
    cmd.env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONSTARTUP")
        .env_remove("PYTHONOPTIMIZE")
        .env_remove("PYTHONEXECUTABLE")
        .env_remove("PYTHONNOUSERSITE");
}

/// Checks if `yt-dlp` is available either in a custom directory or in system PATH.
pub fn check_ytdlp_available(custom_dir: Option<&Path>) -> bool {
    find_ytdlp_binary(custom_dir).is_some()
}

/// Checks if `ffmpeg` is available in system PATH.
pub fn check_ffmpeg_available() -> bool {
    find_ffmpeg_binary().is_some()
}

/// Finds the path to `yt-dlp`, checking custom directory first then system PATH.
pub fn find_ytdlp_binary(custom_dir: Option<&Path>) -> Option<PathBuf> {
    if let Some(dir) = custom_dir {
        let candidate = dir.join(get_ytdlp_filename());
        if candidate.is_file() {
            let mut cmd = Command::new(&candidate);
            clean_env_std(&mut cmd);
            if cmd.arg("--version").output().is_ok() {
                return Some(candidate);
            }
        }
    }

    let sys_name = get_ytdlp_filename();
    let mut cmd = Command::new(sys_name);
    clean_env_std(&mut cmd);
    if let Ok(output) = cmd.arg("--version").output() {
        if output.status.success() {
            return Some(PathBuf::from(sys_name));
        }
    }

    None
}

/// Finds the path to `ffmpeg` in system PATH.
pub fn find_ffmpeg_binary() -> Option<PathBuf> {
    let mut cmd = Command::new("ffmpeg");
    clean_env_std(&mut cmd);
    if let Ok(output) = cmd.arg("-version").output() {
        if output.status.success() {
            return Some(PathBuf::from("ffmpeg"));
        }
    }
    None
}

#[derive(Debug, Clone, Default)]
pub struct YtDlpDetector {
    custom_dir: Option<PathBuf>,
}

impl YtDlpDetector {
    pub fn new(custom_dir: Option<PathBuf>) -> Self {
        Self { custom_dir }
    }

    pub fn is_ytdlp_available(&self) -> bool {
        check_ytdlp_available(self.custom_dir.as_deref())
    }

    pub fn is_ffmpeg_available(&self) -> bool {
        check_ffmpeg_available()
    }

    pub fn resolve_ytdlp(&self) -> Option<PathBuf> {
        find_ytdlp_binary(self.custom_dir.as_deref())
    }

    pub fn resolve_ffmpeg(&self) -> Option<PathBuf> {
        find_ffmpeg_binary()
    }

    /// Downloads the official `yt-dlp` executable into the target directory, making it executable.
    pub async fn ensure_ytdlp_downloaded(&self, target_dir: &Path) -> Result<PathBuf> {
        if !target_dir.exists() {
            std::fs::create_dir_all(target_dir)
                .with_context(|| format!("Failed to create directory: {:?}", target_dir))?;
        }

        let binary_path = target_dir.join(get_ytdlp_filename());
        if binary_path.exists() {
            return Ok(binary_path);
        }

        let url = get_ytdlp_download_url();
        let client = reqwest::Client::builder()
            .user_agent("music-tools-downloader")
            .build()
            .context("Failed to build HTTP client for yt-dlp download")?;

        let response = client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch yt-dlp from {}", url))?;

        let bytes = response
            .bytes()
            .await
            .context("Failed to download yt-dlp binary content")?;

        std::fs::write(&binary_path, &bytes)
            .with_context(|| format!("Failed to write yt-dlp binary to {:?}", binary_path))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&binary_path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&binary_path, perms);
            }
        }

        Ok(binary_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ytdlp_filename_and_url() {
        let filename = get_ytdlp_filename();
        let url = get_ytdlp_download_url();

        assert!(!filename.is_empty());
        assert!(url.starts_with("https://github.com/yt-dlp/yt-dlp/releases/latest/download/"));
        assert!(url.ends_with(filename));
    }

    #[test]
    fn test_check_ffmpeg_available() {
        // Just verify it runs without crashing or panicking
        let _ = check_ffmpeg_available();
    }

    #[test]
    fn test_detector_with_custom_dir() {
        let temp_dir = std::env::temp_dir().join(format!("mt_detector_test_{}", std::process::id()));
        let detector = YtDlpDetector::new(Some(temp_dir.clone()));

        // In an empty temp dir, custom yt-dlp won't be found
        assert_eq!(
            detector.resolve_ytdlp(),
            find_ytdlp_binary(None)
        );
    }
}
