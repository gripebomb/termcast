//! TermCast - A beautiful weather CLI tool for terminal enthusiasts.
//!
//! Provides weather information with aesthetic terminal output.
//! Works out of the box with auto-detected location or with `--location` flag.

use clap::Parser;
use termcast::{api::Client, cache, errors::AppError, renderer, weather};

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
        // Render styled error to stderr
        eprintln!("termcast: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    let args = Args::parse();

    // Handle --install flag
    if let Some(shell) = args.install {
        return install_shell_integration(&shell);
    }

    let client = Client::new();
    let cache_path = cache::cache_path();

    // In ambient mode, try cache first
    if args.ambient {
        return run_ambient_mode(&client, &cache_path, args.location.as_deref(), args.cache_ttl).await;
    }

    // Regular mode: fetch weather, display, and cache
    let (weather_data, latitude, longitude) = fetch_and_display_weather(&client, args.location.as_deref()).await?;

    // Write to cache
    write_weather_cache(&cache_path, &weather_data, latitude, longitude)?;

    Ok(())
}

/// Fetches weather data and displays it in the terminal.
async fn fetch_and_display_weather(
    client: &Client,
    location: Option<&str>,
) -> Result<(termcast::weather::WeatherDisplay, f64, f64), AppError> {
    // Get location (either from CLI or auto-detect)
    let (latitude, longitude, location_name) = if let Some(loc) = location {
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

    Ok((weather_data, latitude, longitude))
}

/// Writes weather data to the cache.
fn write_weather_cache(
    cache_path: &std::path::Path,
    weather_data: &termcast::weather::WeatherDisplay,
    latitude: f64,
    longitude: f64,
) -> Result<(), AppError> {
    let entry = cache::CacheEntry::new(
        weather_data.temperature,
        weather_data.weather_code,
        weather_data.location.clone(),
        latitude,
        longitude,
    );

    cache::write_cache(cache_path, &entry)
}

/// Runs in ambient mode - compact output for shell prompts.
async fn run_ambient_mode(
    client: &Client,
    cache_path: &std::path::Path,
    location: Option<&str>,
    cache_ttl_minutes: u64,
) -> Result<(), AppError> {
    let ttl_secs = (cache_ttl_minutes * 60) as i64;

    // Try to read from cache first
    if let Ok(Some(entry)) = cache::read_cache(cache_path) {
        // Check if cache is fresh
        if cache::is_cache_fresh(&entry, ttl_secs) {
            // Cache is fresh - output immediately
            output_ambient_weather(entry.weather_code, entry.temperature);
            return Ok(());
        }
    }

    // Cache missing or stale - fetch fresh data
    let (latitude, longitude, location_name) = if let Some(loc) = location {
        client.geocode_location(loc).await?
    } else {
        client.get_location().await?
    };

    let weather_data = client
        .get_weather(latitude, longitude, &location_name)
        .await?;

    // Write to cache
    write_weather_cache(cache_path, &weather_data, latitude, longitude)?;

    // Output ambient format
    output_ambient_weather(weather_data.weather_code, weather_data.temperature);

    Ok(())
}

/// Outputs compact weather in format "☀️ 14°" for shell prompts.
fn output_ambient_weather(weather_code: u32, temperature: f64) {
    let icon = weather::weather_icon(weather_code);
    let temp = temperature as i32;
    println!("{} {}°", icon, temp);
}

/// Outputs shell integration snippets.
fn install_shell_integration(shell: &str) -> Result<(), AppError> {
    match shell {
        "bash" | "zsh" | "tmux" | "all" => {}
        _ => {
            return Err(AppError::invalid_arg(format!(
                "Unknown shell '{}'. Use: bash, zsh, tmux, or all",
                shell
            )));
        }
    }

    if shell == "bash" || shell == "all" {
        println!("# Bash integration - add to ~/.bashrc or ~/.bash_profile:");
        println!();
        println!("termcast_prompt() {{");
        println!("    termcast --ambient");
        println!("}}");
        println!();
        println!("# Add to your PS1:");
        println!("# export PS1='\\u@\\h \\w $(termcast_prompt)$ '");
        println!();
    }

    if shell == "zsh" || shell == "all" {
        if shell == "all" {
            println!();
            println!("---");
            println!();
        }
        println!("# Zsh integration - add to ~/.zshrc:");
        println!();
        println!("termcast_prompt() {{");
        println!("    termcast --ambient");
        println!("}}");
        println!();
        println!("# Add to your PROMPT:");
        println!(r"# PROMPT='%n@%m %~ $(termcast_prompt)% '");
        println!();
    }

    if shell == "tmux" || shell == "all" {
        if shell == "all" {
            println!();
            println!("---");
            println!();
        }
        println!("# tmux integration - add to ~/.tmux.conf:");
        println!();
        println!("set -g status-right '#(termcast --ambient)'");
        println!();
    }

    if shell == "all" {
        println!("---");
        println!("# Run 'termcast --install bash' (or zsh/tmux) to see just that snippet.");
    }

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
