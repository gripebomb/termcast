//! Configuration file loading and management.
//!
//! Loads optional TOML config from XDG config directory.
//! Falls back to defaults when no config file exists.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Application configuration loaded from TOML file.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub locations: HashMap<String, Location>,
}

/// Default settings for weather display.
#[derive(Debug, Deserialize)]
pub struct Defaults {
    #[serde(default = "Defaults::default_location")]
    pub default_location: String,
    #[serde(default = "Defaults::default_units")]
    pub units: String,
    #[serde(default = "Defaults::default_cache_ttl")]
    pub cache_ttl: u64,
    #[serde(default)]
    pub theme: String,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            default_location: Self::default_location(),
            units: Self::default_units(),
            cache_ttl: Self::default_cache_ttl(),
            theme: String::new(),
        }
    }
}

impl Defaults {
    fn default_location() -> String {
        "auto".to_string()
    }
    fn default_units() -> String {
        "auto".to_string()
    }
    fn default_cache_ttl() -> u64 {
        15
    }
}

/// A named location entry in the config.
#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    pub city: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// Returns the config file path, respecting XDG_CONFIG_HOME.
///
/// Falls back to ~/.config/termcast/config.toml if XDG_CONFIG_HOME is not set.
pub fn config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .expect("Cannot determine home directory")
                .join(".config")
        });

    base.join("termcast").join("config.toml")
}

/// Loads config from the given path, or the default XDG path if None.
///
/// Returns default `Config` if the file doesn't exist.
/// Logs a warning to stderr on parse errors and returns defaults.
pub fn load_config(path: Option<&Path>) -> Config {
    let config_path = match path {
        Some(p) => p.to_path_buf(),
        None => config_path(),
    };

    let contents = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Config::default(),
    };

    match toml::from_str(&contents) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "termcast: warning: failed to parse config {}: {}",
                config_path.display(),
                e
            );
            Config::default()
        }
    }
}

impl Config {
    /// Resolves the units setting to a boolean for Fahrenheit.
    ///
    /// Returns `None` for "auto" (use IP-based detection),
    /// `Some(true)` for "fahrenheit", `Some(false)` for "celsius".
    pub fn resolve_units(&self) -> Option<bool> {
        match self.defaults.units.to_lowercase().as_str() {
            "fahrenheit" | "f" => Some(true),
            "celsius" | "c" => Some(false),
            _ => None,
        }
    }

    /// Resolves a location query against saved locations.
    ///
    /// Returns `Some(ResolvedLocation)` if the query matches a saved location name.
    /// Returns `None` if no match (caller should fall back to geocoding).
    pub fn resolve_location_query(&self, query: &str) -> Option<ResolvedLocation> {
        self.locations.get(query).map(|loc| ResolvedLocation {
            city: loc.city.clone(),
            latitude: loc.latitude,
            longitude: loc.longitude,
        })
    }
}

/// A resolved location with optional coordinates.
#[derive(Debug, Clone)]
pub struct ResolvedLocation {
    pub city: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_valid_toml() {
        let toml = r#"
[defaults]
default_location = "home"
units = "celsius"
cache_ttl = 30

[locations.home]
city = "Oslo"
latitude = 59.91
longitude = 10.75

[locations.mom]
city = "Chicago"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.defaults.default_location, "home");
        assert_eq!(config.defaults.units, "celsius");
        assert_eq!(config.defaults.cache_ttl, 30);
        assert_eq!(config.locations.len(), 2);
        assert_eq!(config.locations["home"].city, "Oslo");
        assert_eq!(config.locations["home"].latitude, Some(59.91));
        assert_eq!(config.locations["home"].longitude, Some(10.75));
        assert_eq!(config.locations["mom"].city, "Chicago");
        assert!(config.locations["mom"].latitude.is_none());
    }

    #[test]
    fn test_parse_empty_toml() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.defaults.default_location, "auto");
        assert_eq!(config.defaults.units, "auto");
        assert_eq!(config.defaults.cache_ttl, 15);
        assert!(config.locations.is_empty());
    }

    #[test]
    fn test_parse_partial_fields_get_defaults() {
        let toml = r#"
[defaults]
units = "fahrenheit"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.defaults.default_location, "auto");
        assert_eq!(config.defaults.units, "fahrenheit");
        assert_eq!(config.defaults.cache_ttl, 15);
    }

    #[test]
    fn test_parse_invalid_toml_returns_error() {
        let result = toml::from_str::<Config>("not valid [[[");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_missing_file_returns_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.toml");
        let config = load_config(Some(&path));
        assert_eq!(config.defaults.default_location, "auto");
        assert_eq!(config.defaults.units, "auto");
        assert_eq!(config.defaults.cache_ttl, 15);
    }

    #[test]
    fn test_load_config_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("config.toml");
        std::fs::write(&path, "[defaults]\nunits = \"celsius\"\n").unwrap();
        let config = load_config(Some(&path));
        assert_eq!(config.defaults.units, "celsius");
    }

    #[test]
    fn test_load_config_none_path_uses_xdg() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("termcast");
        std::fs::create_dir_all(&config_dir).unwrap();
        let file_path = config_dir.join("config.toml");
        std::fs::write(&file_path, "[defaults]\nunits = \"fahrenheit\"\n").unwrap();

        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        let config = load_config(None);
        assert_eq!(config.defaults.units, "fahrenheit");
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_resolve_units_auto() {
        let config = Config::default();
        assert_eq!(config.resolve_units(), None);
    }

    #[test]
    fn test_resolve_units_celsius() {
        let toml = "[defaults]\nunits = \"celsius\"\n";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.resolve_units(), Some(false));
    }

    #[test]
    fn test_resolve_units_fahrenheit() {
        let toml = "[defaults]\nunits = \"fahrenheit\"\n";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.resolve_units(), Some(true));
    }

    #[test]
    fn test_resolve_units_shorthand() {
        let toml = "[defaults]\nunits = \"F\"\n";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.resolve_units(), Some(true));

        let toml = "[defaults]\nunits = \"C\"\n";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.resolve_units(), Some(false));
    }

    #[test]
    fn test_resolve_location_query_found() {
        let toml = r#"
[locations.home]
city = "Oslo"
latitude = 59.91
longitude = 10.75
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let resolved = config.resolve_location_query("home").unwrap();
        assert_eq!(resolved.city, "Oslo");
        assert_eq!(resolved.latitude, Some(59.91));
        assert_eq!(resolved.longitude, Some(10.75));
    }

    #[test]
    fn test_resolve_location_query_not_found() {
        let config = Config::default();
        assert!(config.resolve_location_query("home").is_none());
    }

    #[test]
    fn test_resolve_location_query_partial_coordinates() {
        let toml = r#"
[locations.office]
city = "San Francisco"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let resolved = config.resolve_location_query("office").unwrap();
        assert_eq!(resolved.city, "San Francisco");
        assert!(resolved.latitude.is_none());
        assert!(resolved.longitude.is_none());
    }

    #[test]
    fn test_config_path_respects_xdg() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        let path = config_path();
        assert!(path.starts_with(temp_dir.path()));
        assert_eq!(path.file_name().unwrap(), "config.toml");

        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_config_path_fallback_to_home() {
        std::env::remove_var("XDG_CONFIG_HOME");
        let path = config_path();
        assert!(path.to_string_lossy().contains(".config"));
        assert!(path.to_string_lossy().contains("termcast"));
        assert_eq!(path.file_name().unwrap(), "config.toml");
    }

    // --- Theme field tests ---

    #[test]
    fn test_theme_field_defaults_to_empty() {
        let config = Config::default();
        assert_eq!(config.defaults.theme, "");
    }

    #[test]
    fn test_theme_field_parsed_from_toml() {
        let toml = "[defaults]\ntheme = \"dracula\"\n";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.defaults.theme, "dracula");
    }

    #[test]
    fn test_theme_field_missing_gets_empty_string() {
        let toml = "[defaults]\nunits = \"celsius\"\n";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.defaults.theme, "");
    }
}
