use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context as _, Result};
use rusqlite::{Connection, params};

pub const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS snapshots (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        saved_at INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS cache_metadata (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at INTEGER NOT NULL
    );
";

pub const DEFAULT_FUZZY_THRESHOLD: f64 = 0.80;
pub const DEFAULT_CONCURRENT_DOWNLOADS: usize = 3;

pub const SETTING_FUZZY_THRESHOLD: &str = "fuzzy_threshold";
pub const SETTING_DOWNLOAD_DIR: &str = "download_dir";
pub const SETTING_MUSIC_DIR: &str = "music_dir";
pub const SETTING_CONCURRENT_DOWNLOADS: &str = "concurrent_downloads";

#[derive(Clone, Debug)]
pub struct CacheStorage {
    path: PathBuf,
    initialized: Arc<Mutex<bool>>,
}

impl CacheStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    pub fn standard() -> Result<Self> {
        let dir = dirs_fallback_cache_dir().join("music_tools");
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create cache directory: {:?}", dir))?;
        Ok(Self::new(dir.join("cache.sqlite")))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn connect(&self) -> Result<Connection> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent dir for {:?}", self.path))?;
        }

        let conn = Connection::open(&self.path)
            .with_context(|| format!("Failed to open database at {:?}", self.path))?;
        conn.busy_timeout(Duration::from_secs(5))
            .context("Failed to configure busy timeout")?;

        let mut init = self.initialized.lock().unwrap_or_else(|e| e.into_inner());
        if !*init {
            init_schema(&conn)?;
            *init = true;
        }

        Ok(conn)
    }

    pub fn set_snapshot(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.connect()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO snapshots (key, value, saved_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, saved_at = excluded.saved_at",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get_snapshot(&self, key: &str) -> Result<Option<String>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT value FROM snapshots WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.connect()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn get_fuzzy_threshold(&self) -> Result<f64> {
        match self.get_setting(SETTING_FUZZY_THRESHOLD)? {
            Some(val) => val
                .parse::<f64>()
                .with_context(|| format!("Invalid numeric value for {SETTING_FUZZY_THRESHOLD}: {val}")),
            None => Ok(DEFAULT_FUZZY_THRESHOLD),
        }
    }

    pub fn set_fuzzy_threshold(&self, threshold: f64) -> Result<()> {
        self.set_setting(SETTING_FUZZY_THRESHOLD, &threshold.to_string())
    }

    pub fn get_download_dir(&self) -> Result<Option<PathBuf>> {
        Ok(self.get_setting(SETTING_DOWNLOAD_DIR)?.map(PathBuf::from))
    }

    pub fn set_download_dir(&self, path: impl AsRef<Path>) -> Result<()> {
        self.set_setting(SETTING_DOWNLOAD_DIR, &path.as_ref().to_string_lossy())
    }

    pub fn get_music_dir(&self) -> Result<Option<PathBuf>> {
        Ok(self.get_setting(SETTING_MUSIC_DIR)?.map(PathBuf::from))
    }

    pub fn set_music_dir(&self, path: impl AsRef<Path>) -> Result<()> {
        self.set_setting(SETTING_MUSIC_DIR, &path.as_ref().to_string_lossy())
    }

    pub fn get_concurrent_downloads(&self) -> Result<usize> {
        match self.get_setting(SETTING_CONCURRENT_DOWNLOADS)? {
            Some(val) => val
                .parse::<usize>()
                .with_context(|| format!("Invalid numeric value for {SETTING_CONCURRENT_DOWNLOADS}: {val}")),
            None => Ok(DEFAULT_CONCURRENT_DOWNLOADS),
        }
    }

    pub fn set_concurrent_downloads(&self, count: usize) -> Result<()> {
        self.set_setting(SETTING_CONCURRENT_DOWNLOADS, &count.to_string())
    }
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)
        .context("Failed to initialize database schema")?;
    Ok(())
}

pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory().context("Failed to open SQLite in-memory database")?;
    conn.busy_timeout(Duration::from_secs(5))
        .context("Failed to configure busy timeout")?;
    init_schema(&conn)?;
    Ok(conn)
}

fn dirs_fallback_cache_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cache")
    } else {
        std::env::temp_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_and_schema() {
        let conn = open_in_memory().expect("failed to open in-memory database");
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        assert!(tables.contains(&"snapshots".to_string()));
        assert!(tables.contains(&"cache_metadata".to_string()));
        assert!(tables.contains(&"settings".to_string()));
    }

    #[test]
    fn test_snapshot_crud_in_memory() {
        let conn = open_in_memory().expect("failed to open in-memory database");
        conn.execute(
            "INSERT INTO snapshots (key, value, saved_at) VALUES (?1, ?2, ?3)",
            params!["playlist_test", "{\"name\":\"Rock Classics\"}", 1700000000],
        )
        .unwrap();

        let val: String = conn
            .query_row(
                "SELECT value FROM snapshots WHERE key = ?1",
                params!["playlist_test"],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(val, "{\"name\":\"Rock Classics\"}");
    }

    #[test]
    fn test_cache_storage_file() {
        let temp_dir = std::env::temp_dir().join(format!("mt_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("cache.sqlite");
        let storage = CacheStorage::new(&db_path);

        storage.set_snapshot("key1", "value1").unwrap();
        assert_eq!(
            storage.get_snapshot("key1").unwrap(),
            Some("value1".to_string())
        );
        assert_eq!(storage.get_snapshot("key_missing").unwrap(), None);

        // Update existing key
        storage.set_snapshot("key1", "value2").unwrap();
        assert_eq!(
            storage.get_snapshot("key1").unwrap(),
            Some("value2".to_string())
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_settings_storage_and_convenience_methods() {
        let temp_dir = std::env::temp_dir().join(format!("mt_settings_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("settings.sqlite");
        let storage = CacheStorage::new(&db_path);

        // 1. Check default values
        assert_eq!(storage.get_fuzzy_threshold().unwrap(), 0.80);
        assert_eq!(storage.get_concurrent_downloads().unwrap(), 3);
        assert_eq!(storage.get_download_dir().unwrap(), None);
        assert_eq!(storage.get_music_dir().unwrap(), None);

        // 2. Custom setting get / set
        storage.set_setting("theme", "dark").unwrap();
        assert_eq!(storage.get_setting("theme").unwrap(), Some("dark".to_string()));
        assert_eq!(storage.get_setting("non_existent").unwrap(), None);

        // Update setting
        storage.set_setting("theme", "light").unwrap();
        assert_eq!(storage.get_setting("theme").unwrap(), Some("light".to_string()));

        // 3. Convenience setters and getters
        storage.set_fuzzy_threshold(0.92).unwrap();
        assert_eq!(storage.get_fuzzy_threshold().unwrap(), 0.92);

        storage.set_concurrent_downloads(6).unwrap();
        assert_eq!(storage.get_concurrent_downloads().unwrap(), 6);

        let dl_path = PathBuf::from("/home/user/Music/Downloads");
        storage.set_download_dir(&dl_path).unwrap();
        assert_eq!(storage.get_download_dir().unwrap(), Some(dl_path));

        let music_path = PathBuf::from("/home/user/Music/Library");
        storage.set_music_dir(&music_path).unwrap();
        assert_eq!(storage.get_music_dir().unwrap(), Some(music_path));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
