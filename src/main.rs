//! TermCast - A beautiful weather CLI tool for terminal enthusiasts.
//!
//! Provides weather information with aesthetic terminal output.
//! Works out of the box with auto-detected location or with `--location` flag.

use clap::Parser;
use termcast::{api::Client, errors::AppError, renderer, weather};

/// Command-line arguments for TermCast.
#[derive(Parser, Debug)]
#[command(
    name = "termcast",
    version,
    about = "A beautiful weather CLI tool for terminal enthusiasts"
)]
struct Args {
    /// Location to get weather for (city name or coordinates).
    /// If not provided, uses IP-based geolocation.
    #[arg(short, long)]
    location: Option<String>,

    /// Run in ambient mode - output compact weather for shell prompts.
    /// Reads from cache first, fetches if stale or missing.
    #[arg(long)]
    ambient: bool,

    /// Cache TTL in minutes for ambient mode (default: 15).
    #[arg(long, default_value_t = 15)]
    cache_ttl: u64,

    /// Install shell integration snippets.
    /// Output formats: bash, zsh, tmux, or all (default).
    #[arg(long, num_args(0..=1), default_missing_value = "all")]
    install: Option<String>,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        // Render styled error
        let _ = renderer::render_error(&e.to_string());
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    let args = Args::parse();
    let client = Client::new();

    // Get location (either from CLI or auto-detect)
    let (latitude, longitude, location_name) = if let Some(ref loc) = args.location {
        // Geocode the provided location
        client.geocode_location(loc).await?
    } else {
        // Auto-detect via IP
        client.get_location().await?
    };

    // Get weather data
    let weather_data = client
        .get_weather(latitude, longitude, &location_name)
        .await?;

    // Get weather description
    let description = weather::weather_description(weather_data.weather_code);

    // Render to terminal
    renderer::render_weather(&weather_data, description)
        .map_err(|e| AppError::invalid_arg(format!("Render error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use crate::Args;

    #[test]
    fn test_args_parsing_no_location() {
        let args = Args::parse_from(&["termcast"]);
        assert!(args.location.is_none());
    }

    #[test]
    fn test_args_parsing_with_location() {
        let args = Args::parse_from(&["termcast", "-l", "Oslo"]);
        assert_eq!(args.location, Some("Oslo".to_string()));
    }

    #[test]
    fn test_args_parsing_long_form() {
        let args = Args::parse_from(&["termcast", "--location", "San Francisco, CA"]);
        assert_eq!(args.location, Some("San Francisco, CA".to_string()));
    }

    #[test]
    fn test_args_ambient_mode() {
        let args = Args::parse_from(&["termcast", "--ambient"]);
        assert!(args.ambient);
        assert!(!args.location.is_some());
    }

    #[test]
    fn test_args_ambient_with_location() {
        let args = Args::parse_from(&["termcast", "--ambient", "--location", "Oslo"]);
        assert!(args.ambient);
        assert_eq!(args.location, Some("Oslo".to_string()));
    }

    #[test]
    fn test_args_cache_ttl_default() {
        let args = Args::parse_from(&["termcast"]);
        assert_eq!(args.cache_ttl, 15);
    }

    #[test]
    fn test_args_cache_ttl_custom() {
        let args = Args::parse_from(&["termcast", "--cache-ttl", "30"]);
        assert_eq!(args.cache_ttl, 30);
    }

    #[test]
    fn test_args_install_bash() {
        let args = Args::parse_from(&["termcast", "--install", "bash"]);
        assert_eq!(args.install, Some("bash".to_string()));
    }

    #[test]
    fn test_args_install_all() {
        let args = Args::parse_from(&["termcast", "--install"]);
        assert_eq!(args.install, Some("all".to_string())); // defaults to "all"
    }
}
