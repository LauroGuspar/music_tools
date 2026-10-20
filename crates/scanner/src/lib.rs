pub mod corrupt;
pub mod duplicates;

pub use corrupt::{check_corrupt_file, scan_corrupt_files, CorruptFile};
pub use duplicates::{
    detect_exact_duplicates, detect_semantic_duplicates, DuplicateGroup, DuplicateReason,
};

use std::path::{Path, PathBuf};

use anyhow::Result;
use core::Track;
use lofty::prelude::*;
use lofty::read_from_path;
use rayon::prelude::*;
use walkdir::WalkDir;

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "m4a", "flac", "wav", "ogg", "aac", "wma", "opus",
];

pub fn is_supported_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_ascii_lowercase();
            SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}

pub fn split_artists(artist_str: &str) -> Vec<String> {
    artist_str
        .split(|c| c == ';' || c == ',' || c == '/')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn extract_track_from_path(path: &Path) -> Track {
    let mut title = None;
    let mut artists = Vec::new();
    let mut album = None;
    let mut year = None;
    let mut duration_ms = 0;

    if let Ok(tagged_file) = read_from_path(path) {
        duration_ms = tagged_file.properties().duration().as_millis() as u64;

        if let Some(tag) = tagged_file.primary_tag() {
            title = tag.title().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
            album = tag.album().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
            year = tag.year();
            if let Some(artist_str) = tag.artist() {
                artists = split_artists(&artist_str);
            }
        }

        for tag in tagged_file.tags() {
            if title.is_none() {
                title = tag.title().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
            }
            if album.is_none() {
                album = tag.album().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
            }
            if year.is_none() {
                year = tag.year();
            }
            if artists.is_empty() {
                if let Some(artist_str) = tag.artist() {
                    artists = split_artists(&artist_str);
                }
            }
        }
    }

    // Fallback inteligente si no hay etiquetas ID3: parsear el nombre del archivo ("Artista - Título" o "Título")
    if title.is_none() {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if let Some(dash_idx) = stem.find(" - ") {
                let artist_part = stem[..dash_idx].trim();
                let title_part = stem[dash_idx + 3..].trim();

                if !title_part.is_empty() {
                    title = Some(title_part.to_string());
                }
                if artists.is_empty() && !artist_part.is_empty() {
                    artists = split_artists(artist_part);
                }
            } else if !stem.trim().is_empty() {
                title = Some(stem.trim().to_string());
            }
        }
    }

    let final_title = title.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Title")
            .to_string()
    });

    let size = std::fs::metadata(path).map(|m| m.len()).ok();
    let absolute_path = std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();

    Track {
        title: final_title,
        artists,
        album,
        duration_ms,
        path: Some(absolute_path),
        spotify_id: None,
        isrc: None,
        release_date: year.map(|y| y.to_string()),
        file_size: size,
    }
}

pub fn scan_directory(path: &Path) -> Result<Vec<Track>> {
    if !path.exists() {
        anyhow::bail!("El directorio especificado no existe: {}", path.display());
    }

    let audio_files: Vec<PathBuf> = WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_supported_audio_file(e.path()))
        .map(|e| e.into_path())
        .collect();

    let tracks: Vec<Track> = audio_files
        .into_par_iter()
        .map(|file_path| extract_track_from_path(&file_path))
        .collect();

    Ok(tracks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_is_supported_audio_file() {
        assert!(is_supported_audio_file(Path::new("song.mp3")));
        assert!(is_supported_audio_file(Path::new("song.M4A")));
        assert!(is_supported_audio_file(Path::new("song.FLAC")));
        assert!(is_supported_audio_file(Path::new("song.wav")));
        assert!(is_supported_audio_file(Path::new("song.ogg")));
        assert!(is_supported_audio_file(Path::new("song.aac")));
        assert!(is_supported_audio_file(Path::new("song.wma")));
        assert!(is_supported_audio_file(Path::new("song.opus")));

        assert!(!is_supported_audio_file(Path::new("notes.txt")));
        assert!(!is_supported_audio_file(Path::new("cover.jpg")));
        assert!(!is_supported_audio_file(Path::new("noextension")));
    }

    #[test]
    fn test_split_artists() {
        let single = split_artists("Queen");
        assert_eq!(single, vec!["Queen"]);

        let semicolon = split_artists("Queen; David Bowie");
        assert_eq!(semicolon, vec!["Queen", "David Bowie"]);

        let comma = split_artists("Artist A, Artist B, Artist C");
        assert_eq!(comma, vec!["Artist A", "Artist B", "Artist C"]);

        let slash = split_artists("Artist 1 / Artist 2");
        assert_eq!(slash, vec!["Artist 1", "Artist 2"]);

        let mixed = split_artists("Artist 1; Artist 2 / Artist 3, Artist 4");
        assert_eq!(mixed, vec!["Artist 1", "Artist 2", "Artist 3", "Artist 4"]);

        let with_empty = split_artists(";; Artist A ,, / ");
        assert_eq!(with_empty, vec!["Artist A"]);
    }

    #[test]
    fn test_extract_track_filename_fallback() {
        let temp_dir = std::env::temp_dir().join(format!("mt_scanner_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        // 1. "Artist - Title.mp3"
        let path1 = temp_dir.join("Pink Floyd - Comfortably Numb.mp3");
        let mut file1 = File::create(&path1).unwrap();
        file1.write_all(b"dummy audio content").unwrap();

        let track1 = extract_track_from_path(&path1);
        assert_eq!(track1.title, "Comfortably Numb");
        assert_eq!(track1.artists, vec!["Pink Floyd"]);
        assert!(track1.file_size.unwrap() > 0);
        assert!(track1.path.is_some());

        // 2. Multiple artists separated by comma in filename: "Artist 1, Artist 2 - Title.flac"
        let path2 = temp_dir.join("Queen, David Bowie - Under Pressure.flac");
        let mut file2 = File::create(&path2).unwrap();
        file2.write_all(b"more dummy audio content").unwrap();

        let track2 = extract_track_from_path(&path2);
        assert_eq!(track2.title, "Under Pressure");
        assert_eq!(track2.artists, vec!["Queen", "David Bowie"]);

        // 3. Just "Title.wav" without dash
        let path3 = temp_dir.join("Bohemian Rhapsody.wav");
        let mut file3 = File::create(&path3).unwrap();
        file3.write_all(b"wav content").unwrap();

        let track3 = extract_track_from_path(&path3);
        assert_eq!(track3.title, "Bohemian Rhapsody");
        assert!(track3.artists.is_empty());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_scan_directory_recursive() {
        let base_dir = std::env::temp_dir().join(format!("mt_scan_dir_test_{}", std::process::id()));
        let sub_dir = base_dir.join("subfolder");
        fs::create_dir_all(&sub_dir).unwrap();

        let f1 = base_dir.join("The Beatles - Yesterday.mp3");
        let f2 = sub_dir.join("Radiohead - Creep.opus");
        let non_audio = sub_dir.join("lyrics.txt");

        fs::write(&f1, b"mp3 bytes").unwrap();
        fs::write(&f2, b"opus bytes").unwrap();
        fs::write(&non_audio, b"hello lyrics").unwrap();

        let tracks = scan_directory(&base_dir).unwrap();
        assert_eq!(tracks.len(), 2);

        let titles: Vec<&str> = tracks.iter().map(|t| t.title.as_str()).collect();
        assert!(titles.contains(&"Yesterday"));
        assert!(titles.contains(&"Creep"));

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_scan_directory_not_found() {
        let fake_path = Path::new("/path/that/definitely/does/not/exist/music_test_123");
        let result = scan_directory(fake_path);
        assert!(result.is_err());
    }
}
