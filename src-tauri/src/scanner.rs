use lofty::prelude::*;
use lofty::read_from_path;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalTrack {
    pub path: String,
    pub title: Option<String>,
    pub artists: Vec<String>,
    pub filename: String,
    pub size: u64,
}

pub fn scan_dir_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let _ = scan_dir_recursive(&path, files); // continue scanning, ignore permission errors inside
            } else if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ["mp3", "m4a", "flac", "wav", "ogg", "aac", "wma"].contains(&ext_str.as_str()) {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}

pub fn extract_track_info(path: &Path) -> (Option<String>, Vec<String>) {
    let mut title = None;
    let mut artists = Vec::new();

    if let Ok(tagged_file) = read_from_path(path) {
        if let Some(tag) = tagged_file.primary_tag() {
            title = tag.title().map(|s| s.to_string());
            if let Some(artist_str) = tag.artist() {
                artists = artist_str
                    .split(|c| c == ';' || c == ',' || c == '/')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        } else {
            for tag in tagged_file.tags() {
                if title.is_none() {
                    title = tag.title().map(|s| s.to_string());
                }
                if artists.is_empty() {
                    if let Some(artist_str) = tag.artist() {
                        artists = artist_str
                            .split(|c| c == ';' || c == ',' || c == '/')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
            }
        }
    }

    if title.is_none() {
        if let Some(filename_str) = path.file_stem().and_then(|s| s.to_str()) {
            if let Some(dash_idx) = filename_str.find(" - ") {
                let artist_part = filename_str[..dash_idx].trim();
                let title_part = filename_str[dash_idx + 3..].trim();

                title = Some(title_part.to_string());
                if artists.is_empty() && !artist_part.is_empty() {
                    artists = vec![artist_part.to_string()];
                }
            } else {
                title = Some(filename_str.to_string());
            }
        }
    }

    (title, artists)
}

#[tauri::command]
pub fn scan_local_folder(path: String) -> Result<Vec<LocalTrack>, String> {
    let root_path = Path::new(&path);
    if !root_path.exists() {
        return Err(format!("El directorio especificado no existe: {path}"));
    }

    let mut audio_files = Vec::new();
    scan_dir_recursive(root_path, &mut audio_files)
        .map_err(|error| format!("Error al escanear directorio: {error}"))?;

    let tracks = audio_files
        .into_iter()
        .map(|file_path| {
            let filename = file_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let absolute_path = file_path.to_string_lossy().to_string();
            let (title, artists) = extract_track_info(&file_path);
            let size = fs::metadata(&file_path)
                .map(|meta| meta.len())
                .unwrap_or(0);

            LocalTrack {
                path: absolute_path,
                title,
                artists,
                filename,
                size,
            }
        })
        .collect();

    Ok(tracks)
}
