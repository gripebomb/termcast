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
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::InvalidArg(format!("IO error: {}", e))
    }
}
