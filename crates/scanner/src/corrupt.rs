use std::path::{Path, PathBuf};

use anyhow::Result;
use lofty::read_from_path;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::is_supported_audio_file;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorruptFile {
    pub path: PathBuf,
    pub filename: String,
    pub size: u64,
    pub error_reason: String,
}

pub fn check_corrupt_file(file_path: &Path) -> Option<CorruptFile> {
    let filename = file_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let metadata = match std::fs::metadata(file_path) {
        Ok(m) => m,
        Err(e) => {
            return Some(CorruptFile {
                path: file_path.to_path_buf(),
                filename,
                size: 0,
                error_reason: format!("No se pudo leer metadatos del archivo: {e}"),
            });
        }
    };

    let size = metadata.len();
    if size == 0 {
        return Some(CorruptFile {
            path: file_path.to_path_buf(),
            filename,
            size: 0,
            error_reason: "El archivo está vacío (0 bytes)".to_string(),
        });
    }

    if let Err(e) = read_from_path(file_path) {
        return Some(CorruptFile {
            path: file_path.to_path_buf(),
            filename,
            size,
            error_reason: format!("Estructura de audio o cabecera corrupta: {e}"),
        });
    }

    None
}

pub fn scan_corrupt_files(path: &Path) -> Result<Vec<CorruptFile>> {
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

    let corrupt_files: Vec<CorruptFile> = audio_files
        .into_par_iter()
        .filter_map(|file_path| check_corrupt_file(&file_path))
        .collect();

    Ok(corrupt_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_corrupt_files_detection() {
        let temp_dir = std::env::temp_dir().join(format!("mt_corrupt_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        // 1. Zero bytes audio file
        let zero_file = temp_dir.join("empty.mp3");
        fs::write(&zero_file, b"").unwrap();

        // 2. Corrupted audio structure (garbage bytes with audio extension)
        let bad_audio_file = temp_dir.join("corrupted.flac");
        fs::write(&bad_audio_file, b"this is completely invalid audio bytes that will fail header parsing").unwrap();

        // 3. Non-audio file with corrupt bytes (should be ignored)
        let non_audio = temp_dir.join("ignored.txt");
        fs::write(&non_audio, b"hello text file").unwrap();

        let corrupt_list = scan_corrupt_files(&temp_dir).unwrap();
        assert_eq!(corrupt_list.len(), 2);

        let empty_entry = corrupt_list.iter().find(|c| c.filename == "empty.mp3").unwrap();
        assert_eq!(empty_entry.size, 0);
        assert_eq!(empty_entry.error_reason, "El archivo está vacío (0 bytes)");

        let bad_entry = corrupt_list.iter().find(|c| c.filename == "corrupted.flac").unwrap();
        assert!(bad_entry.size > 0);
        assert!(bad_entry.error_reason.starts_with("Estructura de audio o cabecera corrupta:"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_scan_corrupt_files_not_found() {
        let fake_path = Path::new("/path/that/does/not/exist/corrupt_123");
        let result = scan_corrupt_files(fake_path);
        assert!(result.is_err());
    }
}
