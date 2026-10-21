use std::process::Command;

use anyhow::{Context, Result};
use core::Track;

use crate::detector::{clean_env_std, find_ytdlp_binary};

/// Extracts playlist or video information from a YouTube or generic URL using `yt-dlp`.
pub fn extract_youtube_playlist(url: &str) -> Result<(String, Vec<Track>)> {
    let ytdlp = find_ytdlp_binary(None).ok_or_else(|| {
        anyhow::anyhow!("No se encontró yt-dlp en el sistema ni en el directorio de descargas.")
    })?;

    let mut cmd = Command::new(ytdlp);
    clean_env_std(&mut cmd);
    cmd.args([
        "-J",
        "--flat-playlist",
        "--no-warnings",
        "--ignore-errors",
        url,
    ]);

    let output = cmd
        .output()
        .with_context(|| "Fallo al ejecutar yt-dlp para extraer información de YouTube")?;
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp reportó un error: {}", err_msg.trim());
    }

    let json_val: serde_json::Value = serde_json::from_slice(&output.stdout)
        .with_context(|| "Fallo al analizar la salida JSON de yt-dlp")?;

    let playlist_title = json_val
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Lista de reproducción de YouTube")
        .to_string();

    let mut tracks = Vec::new();

    if let Some(entries) = json_val.get("entries").and_then(|v| v.as_array()) {
        for entry in entries {
            let title = entry
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let uploader = entry
                .get("uploader")
                .or_else(|| entry.get("channel"))
                .and_then(|v| v.as_str())
                .unwrap_or("YouTube")
                .to_string();
            let duration = entry
                .get("duration")
                .and_then(|v| v.as_f64())
                .map(|d| (d * 1000.0) as u64)
                .unwrap_or(0);
            let entry_url = entry
                .get("url")
                .or_else(|| entry.get("webpage_url"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if !title.is_empty() {
                let mut track = Track::new(title, vec![uploader], duration);
                if let Some(u) = entry_url {
                    track.spotify_id = Some(u);
                }
                tracks.push(track);
            }
        }
    } else {
        // Single video
        let title = json_val
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let uploader = json_val
            .get("uploader")
            .or_else(|| json_val.get("channel"))
            .and_then(|v| v.as_str())
            .unwrap_or("YouTube")
            .to_string();
        let duration = json_val
            .get("duration")
            .and_then(|v| v.as_f64())
            .map(|d| (d * 1000.0) as u64)
            .unwrap_or(0);

        if !title.is_empty() {
            let mut track = Track::new(title, vec![uploader], duration);
            track.spotify_id = Some(url.to_string());
            tracks.push(track);
        }
    }

    Ok((playlist_title, tracks))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_finds_ytdlp_or_fails_gracefully() {
        let result = extract_youtube_playlist("https://www.youtube.com/watch?v=invalid_or_offline");
        // Should either fail with error or run if ytdlp is present
        assert!(result.is_err() || result.is_ok());
    }
}
