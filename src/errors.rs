//! Structured error types for the application.
//!
//! All errors are descriptive and include context (URLs, field names, etc.)
//! for debugging purposes.

use thiserror::Error;

/// Main application error type.
///
/// Covers all failure modes from network errors to parsing failures.
#[derive(Debug, Error)]
pub enum AppError {
    /// Failed to determine location via IP geolocation.
    #[error("Failed to determine location: {0}")]
    GeolocationError(String),

    /// Network-related errors when making HTTP requests.
    #[error("Network error fetching {url}: {source}")]
    NetworkError {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    /// Failed to parse JSON response from an API.
    #[error("Failed to parse {field} from {url}: {source}")]
    ParseError {
        url: String,
        field: String,
        #[source]
        source: serde_json::Error,
    },

    /// API returned an error response.
    #[error("API error from {url}: {message}")]
    WeatherError { url: String, message: String },

    /// Invalid command-line arguments.
    #[error("Invalid argument: {0}")]
    InvalidArg(String),

    /// Cache read error.
    #[error("Failed to read cache from {path}: {source}")]
    CacheRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Cache write error.
    #[error("Failed to write cache to {path}: {source}")]
    CacheWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Cache parse error (invalid JSON).
    #[error("Failed to parse cache from {path}: {source}")]
    CacheParse {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    /// Cache serialization error.
    #[error("Failed to serialize cache for {path}: {source}")]
    CacheSerialize {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}

impl AppError {
    /// Creates a network error with URL context.
    pub fn network(url: &str, source: reqwest::Error) -> Self {
        Self::NetworkError {
            url: url.to_string(),
            source,
        }
    }

    /// Creates a parse error with URL and field context.
    pub fn parse(url: &str, field: &str, source: serde_json::Error) -> Self {
        Self::ParseError {
            url: url.to_string(),
            field: field.to_string(),
            source,
        }
    }

    /// Creates a geolocation error with context.
    pub fn geolocation(msg: impl Into<String>) -> Self {
        Self::GeolocationError(msg.into())
    }

    /// Creates a weather API error.
    pub fn weather(url: &str, message: &str) -> Self {
        Self::WeatherError {
            url: url.to_string(),
            message: message.to_string(),
        }
    }

    /// Creates an invalid argument error.
    #[allow(dead_code)]
    pub fn invalid_arg(msg: impl Into<String>) -> Self {
        Self::InvalidArg(msg.into())
    }

    /// Creates a cache read error.
    #[allow(dead_code)]
    pub fn cache_read(path: &std::path::Path, source: std::io::Error) -> Self {
        Self::CacheRead {
            path: path.to_string_lossy().to_string(),
            source,
        }
    }

    /// Creates a cache write error.
    #[allow(dead_code)]
    pub fn cache_write(path: &std::path::Path, source: std::io::Error) -> Self {
        Self::CacheWrite {
            path: path.to_string_lossy().to_string(),
            source,
        }
    }

    /// Creates a cache parse error.
    #[allow(dead_code)]
    pub fn cache_parse(path: &std::path::Path, source: serde_json::Error) -> Self {
        Self::CacheParse {
            path: path.to_string_lossy().to_string(),
            source,
        }
    }

    /// Creates a cache serialization error.
    #[allow(dead_code)]
    pub fn cache_serialize(path: &std::path::Path, source: serde_json::Error) -> Self {
        Self::CacheSerialize {
            path: path.to_string_lossy().to_string(),
            source,
        }
    }
}
