//! TermCast - A beautiful weather CLI tool for terminal enthusiasts.
//!
//! Provides weather information with aesthetic terminal output.
//! Works out of the box with auto-detected location or with `--location` flag.

mod api;
mod errors;
mod geolocation;
mod renderer;
mod weather;

use clap::Parser;
use errors::AppError;

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
    let client = api::Client::new();

    // Get location (either from CLI or auto-detect)
    let (latitude, longitude, location_name) = if let Some(ref loc) = args.location {
        // Geocode the provided location
        client.geocode_location(loc).await?
    } else {
        // Auto-detect via IP
        client.get_location().await?
    };

    // Get weather data
    let weather = client
        .get_weather(latitude, longitude, &location_name)
        .await?;

    // Get weather description
    let description = weather::weather_description(weather.weather_code);

    // Render to terminal
    renderer::render_weather(&weather, description)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
