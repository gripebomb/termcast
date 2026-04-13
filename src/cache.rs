//! Weather data caching for ambient mode.
//!
//! Caches weather data to disk for fast shell prompt integration.
//! Uses XDG-compliant cache directory with fallback to ~/.cache.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::AppError;

/// Cache entry containing weather data and metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    /// Unix timestamp when this entry was created.
    pub timestamp: i64,
    /// Current temperature in the user's preferred unit.
    pub temperature: f64,
    /// WMO weather code.
    pub weather_code: u32,
    /// Location name.
    pub location: String,
    /// Latitude coordinate.
    pub latitude: f64,
    /// Longitude coordinate.
    pub longitude: f64,
    /// Whether the cached temperature is in Fahrenheit.
    pub use_fahrenheit: bool,
}

impl CacheEntry {
    /// Creates a new cache entry with the current timestamp.
    pub fn new(
        temperature: f64,
        weather_code: u32,
        location: String,
        latitude: f64,
        longitude: f64,
        use_fahrenheit: bool,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        Self {
            timestamp,
            temperature,
            weather_code,
            location,
            latitude,
            longitude,
            use_fahrenheit,
        }
    }
}

/// Returns the cache file path, respecting XDG_CACHE_HOME.
///
/// Falls back to ~/.cache/termcast/current if XDG_CACHE_HOME is not set.
pub fn cache_path() -> PathBuf {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .expect("Cannot determine home directory")
                .join(".cache")
        });

    base.join("termcast").join("current")
}

/// Reads a cache entry from the given path.
///
/// Returns `Ok(None)` if the file doesn't exist or is invalid.
/// Returns `Ok(Some(CacheEntry))` if the file is valid.
/// Returns `Err` only for unexpected I/O errors.
pub fn read_cache(path: &Path) -> Result<Option<CacheEntry>, AppError> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(AppError::cache_read(path, e)),
    };

    let entry: CacheEntry = serde_json::from_str(&contents)
        .map_err(|e| AppError::cache_parse(path, e))?;

    Ok(Some(entry))
}

/// Writes a cache entry to the given path.
///
/// Creates parent directories if they don't exist.
pub fn write_cache(path: &Path, entry: &CacheEntry) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::cache_write(path, e))?;
    }

    let json = serde_json::to_string_pretty(entry)
        .map_err(|e| AppError::cache_serialize(path, e))?;

    std::fs::write(path, json)
        .map_err(|e| AppError::cache_write(path, e))
}

/// Checks if a cache entry is fresh based on the given TTL.
///
/// Returns `true` if the entry is younger than `ttl_secs`.
pub fn is_cache_fresh(entry: &CacheEntry, ttl_secs: i64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;

    now - entry.timestamp < ttl_secs
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_path_respects_xdg() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("XDG_CACHE_HOME", temp_dir.path());

        let path = cache_path();
        assert!(path.starts_with(temp_dir.path()));
        assert_eq!(path.file_name().unwrap(), "current");

        std::env::remove_var("XDG_CACHE_HOME");
    }

    #[test]
    fn test_cache_path_fallback_to_home() {
        std::env::remove_var("XDG_CACHE_HOME");
        let path = cache_path();
        assert!(path.to_string_lossy().contains(".cache"));
        assert_eq!(path.file_name().unwrap(), "current");
    }

    #[test]
    fn test_cache_entry_new_sets_timestamp() {
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let entry = CacheEntry::new(14.0, 0, "Oslo".to_string(), 59.91, 10.75, false);

        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert!(entry.timestamp >= before);
        assert!(entry.timestamp <= after);
        assert_eq!(entry.temperature, 14.0);
        assert_eq!(entry.weather_code, 0);
        assert_eq!(entry.location, "Oslo");
        assert_eq!(entry.latitude, 59.91);
        assert_eq!(entry.longitude, 10.75);
    }

    #[test]
    fn test_read_cache_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent");

        let result = read_cache(&path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_read_cache_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid.json");
        std::fs::write(&path, "not valid json").unwrap();

        let result = read_cache(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_cache_valid_entry() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("current");

        let entry = CacheEntry::new(14.0, 0, "Oslo".to_string(), 59.91, 10.75, false);
        write_cache(&path, &entry).unwrap();

        let result = read_cache(&path);
        assert!(result.is_ok());
        let loaded = result.unwrap().unwrap();
        assert_eq!(loaded.temperature, 14.0);
        assert_eq!(loaded.weather_code, 0);
        assert_eq!(loaded.location, "Oslo");
    }

    #[test]
    fn test_write_cache_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("subdir").join("current");

        let entry = CacheEntry::new(20.0, 1, "Berlin".to_string(), 52.52, 13.40, false);
        let result = write_cache(&path, &entry);

        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_is_cache_fresh_within_ttl() {
        let entry = CacheEntry::new(14.0, 0, "Oslo".to_string(), 59.91, 10.75, false);

        // Entry is fresh within 15 minutes (900 seconds)
        assert!(is_cache_fresh(&entry, 900));
        // Entry is fresh within 1 hour
        assert!(is_cache_fresh(&entry, 3600));
    }

    #[test]
    fn test_is_cache_fresh_outside_ttl() {
        // Create an entry with an old timestamp manually
        let mut entry = CacheEntry::new(14.0, 0, "Oslo".to_string(), 59.91, 10.75, false);
        entry.timestamp -= 3600; // 1 hour ago

        // Entry is stale beyond 15 minutes
        assert!(!is_cache_fresh(&entry, 900));
        // But fresh within 2 hours (entry is 1 hour old)
        assert!(is_cache_fresh(&entry, 7200));
    }

    #[test]
    fn test_roundtrip_serialization() {
        let entry = CacheEntry::new(14.5, 3, "San Francisco, CA".to_string(), 37.77, -122.41, false);
        let json = serde_json::to_string(&entry).unwrap();
        let loaded: CacheEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.temperature, entry.temperature);
        assert_eq!(loaded.weather_code, entry.weather_code);
        assert_eq!(loaded.location, entry.location);
        assert_eq!(loaded.latitude, entry.latitude);
        assert_eq!(loaded.longitude, entry.longitude);
        assert_eq!(loaded.timestamp, entry.timestamp);
    }
}
