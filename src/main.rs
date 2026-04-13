//! TermCast - A beautiful weather CLI tool for terminal enthusiasts.
//!
//! Provides weather information with aesthetic terminal output.
//! Works out of the box with auto-detected location or with `--location` flag.

use std::path::Path;

use clap::{Parser, Subcommand};
use termcast::{api::Client, cache, config, errors::AppError, renderer, theme, weather};

/// Command-line arguments for TermCast.
#[derive(Parser, Debug)]
#[command(
    name = "termcast",
    version,
    about = "A beautiful weather CLI tool for terminal enthusiasts"
)]
struct Args {
    /// Location to get weather for (city name or coordinates).
    /// If not provided, uses config default_location or IP-based geolocation.
    #[arg(short, long)]
    location: Option<String>,

    /// Resolve a named or ad-hoc location for weather.
    /// Checks saved locations first, falls back to geocoding.
    #[arg(long)]
    at: Option<String>,

    /// Run in ambient mode - output compact weather for shell prompts.
    /// Reads from cache first, fetches if stale or missing.
    #[arg(long)]
    ambient: bool,

    /// Cache TTL in minutes for ambient mode (default: 15, overridden by config).
    #[arg(long)]
    cache_ttl: Option<u64>,

    /// Path to config file (default: $XDG_CONFIG_HOME/termcast/config.toml).
    #[arg(long)]
    config: Option<String>,

    /// Install shell integration snippets.
    /// Output formats: bash, zsh, tmux, or all (default).
    #[arg(long, num_args(0..=1), default_missing_value = "all")]
    install: Option<String>,

    /// List saved location names from config (one per line).
    #[arg(long, hide = true)]
    list_locations: bool,

    /// List available color themes with descriptions.
    #[arg(long)]
    list_themes: bool,

    /// Preview a color theme with a demo weather display.
    #[arg(long)]
    preview_theme: Option<String>,

    /// Suppress NWS severe weather alert display.
    #[arg(long)]
    no_alerts: bool,

    /// Subcommand (forecast, completions).
    #[command(subcommand)]
    command: Option<Command>,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Show multi-day weather forecast.
    Forecast {
        /// Number of forecast days (1-7).
        #[arg(long, default_value = "5", value_parser = clap::value_parser!(u32).range(1..=7))]
        days: u32,

        /// Show hourly breakdown of notable weather changes.
        #[arg(long)]
        hourly: bool,

        /// Output compact one-line forecast for shell prompts/status bars.
        #[arg(long)]
        ambient: bool,
    },

    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for.
        shell: String,
    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
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

    // Handle --list-themes
    if args.list_themes {
        for t in theme::builtin_themes() {
            println!("  {:<18}{}", t.name, t.description);
        }
        return Ok(());
    }

    // Handle --preview-theme
    if let Some(ref name) = args.preview_theme {
        return match theme::resolve_theme_checked(name) {
            Some(colors) => {
                let adapted = adapt_colors_for_terminal(colors);
                renderer::render_preview_theme(name, &adapted)
                    .map_err(|e| AppError::invalid_arg(format!("Render error: {}", e)))
            }
            None => Err(AppError::invalid_arg(format!(
                "unknown theme '{}'. Use --list-themes to see available themes.",
                name
            ))),
        };
    }

    // Handle completions subcommand
    if let Some(Command::Completions { shell }) = args.command {
        return generate_completions(&shell);
    }

    // Handle forecast subcommand
    if let Some(Command::Forecast {
        days,
        hourly,
        ambient,
    }) = args.command
    {
        return run_forecast(&args, days, hourly, ambient).await;
    }

    // Load config
    let config_path = args.config.as_deref().map(Path::new);
    let cfg = config::load_config(config_path);

    // Handle --list-locations
    if args.list_locations {
        let mut names: Vec<&String> = cfg.locations.keys().collect();
        names.sort();
        for name in names {
            println!("{}", name);
        }
        return Ok(());
    }

    // Resolve cache_ttl: CLI override > config > default (15)
    let cache_ttl = args.cache_ttl.unwrap_or(cfg.defaults.cache_ttl);

    let client = Client::new();
    let cache_path = cache::cache_path();

    // Resolve unit preference from config (None = auto via IP)
    let config_use_fahrenheit = cfg.resolve_units();

    // Resolve location query from CLI args and config
    let location_query = resolve_location_query(&args, &cfg);

    // In ambient mode, try cache first
    if args.ambient {
        return run_ambient_mode(
            &client,
            &cache_path,
            location_query.as_deref(),
            cache_ttl,
            config_use_fahrenheit,
            args.no_alerts,
        )
        .await;
    }

    // Regular mode: fetch weather, display, and cache
    let colors = resolve_theme_colors(&cfg.defaults.theme);
    let (weather_data, latitude, longitude) = fetch_and_display_weather(
        &client,
        location_query.as_deref(),
        config_use_fahrenheit,
        &colors,
        args.no_alerts,
    )
    .await?;

    // Write to cache
    write_weather_cache(&cache_path, &weather_data, latitude, longitude)?;

    Ok(())
}

/// Resolves theme colors from config theme name, adapted for the terminal.
/// Prints a warning to stderr for unknown theme names and falls back to defaults.
/// Caches the COLORTERM check so it's evaluated once.
fn resolve_theme_colors(theme_name: &str) -> termcast::theme::ThemeColors {
    let colors = match theme::resolve_theme_checked(theme_name) {
        Some(c) => c,
        None => {
            if !theme_name.is_empty() {
                eprintln!(
                    "termcast: warning: unknown theme '{}'. Use --list-themes to see available themes.",
                    theme_name
                );
            }
            theme::resolve_theme("default")
        }
    };
    adapt_colors_for_terminal(colors)
}

/// Adapts theme colors for the terminal: falls back to ANSI 256
/// when COLORTERM indicates no true-color support.
fn adapt_colors_for_terminal(colors: &termcast::theme::ThemeColors) -> termcast::theme::ThemeColors {
    if theme::supports_truecolor() {
        termcast::theme::ThemeColors {
            text: colors.text,
            dimmed: colors.dimmed,
            temp_high: colors.temp_high,
            temp_low: colors.temp_low,
            precip_high: colors.precip_high,
            precip_medium: colors.precip_medium,
        }
    } else {
        colors.to_ansi256()
    }
}

/// Resolves the location query string from CLI args and config.
///
/// Precedence: --location > --at > config default_location
/// Returns None if no location is specified (use IP geolocation).
fn resolve_location_query(args: &Args, cfg: &config::Config) -> Option<String> {
    // --location takes highest precedence
    if let Some(ref loc) = args.location {
        return Some(loc.clone());
    }

    // --at resolves against config locations, or is used as-is for geocoding
    if let Some(ref name) = args.at {
        if let Some(resolved) = cfg.resolve_location_query(name) {
            return Some(resolved.city);
        }
        // No saved match — use the name as a geocoding query
        return Some(name.clone());
    }

    // Config default_location
    if cfg.defaults.default_location != "auto" {
        if let Some(resolved) = cfg.resolve_location_query(&cfg.defaults.default_location) {
            return Some(resolved.city);
        }
        return Some(cfg.defaults.default_location.clone());
    }

    None
}

/// Resolves a location string to coordinates, city name, and unit preference.
///
/// For saved config locations with coordinates, uses those directly.
/// Otherwise geocodes the query via API. Falls back to IP geolocation if no query.
async fn resolve_full_location(
    client: &Client,
    query: Option<&str>,
    cfg: &config::Config,
    config_use_fahrenheit: Option<bool>,
) -> Result<(f64, f64, String, bool), AppError> {
    if let Some(q) = query {
        // Check if it matches a saved location with coordinates
        if let Some(resolved) = cfg.resolve_location_query(q) {
            if let (Some(lat), Some(lon)) = (resolved.latitude, resolved.longitude) {
                let use_fahrenheit = if let Some(pref) = config_use_fahrenheit {
                    pref
                } else {
                    let (_, _, _, ip_f) = client.get_location().await?;
                    ip_f
                };
                return Ok((lat, lon, resolved.city, use_fahrenheit));
            }
            // Saved location but no coordinates — geocode the city name
            let (lat, lon, name) = client.geocode_location(&resolved.city).await?;
            let use_fahrenheit = determine_units(client, config_use_fahrenheit).await?;
            return Ok((lat, lon, name, use_fahrenheit));
        }

        // Not a saved location — geocode directly
        let (lat, lon, name) = client.geocode_location(q).await?;
        let use_fahrenheit = determine_units(client, config_use_fahrenheit).await?;
        return Ok((lat, lon, name, use_fahrenheit));
    }

    // No query — IP geolocation
    client.get_location().await
}

/// Determines Fahrenheit preference from config or IP geolocation.
async fn determine_units(
    client: &Client,
    config_use_fahrenheit: Option<bool>,
) -> Result<bool, AppError> {
    if let Some(pref) = config_use_fahrenheit {
        return Ok(pref);
    }
    let (_, _, _, ip_f) = client.get_location().await?;
    Ok(ip_f)
}

/// Fetches weather data and displays it in the terminal.
async fn fetch_and_display_weather(
    client: &Client,
    location: Option<&str>,
    config_use_fahrenheit: Option<bool>,
    colors: &termcast::theme::ThemeColors,
    no_alerts: bool,
) -> Result<(termcast::weather::WeatherDisplay, f64, f64), AppError> {
    let cfg = config::load_config(None);
    let (latitude, longitude, location_name, use_fahrenheit) =
        resolve_full_location(client, location, &cfg, config_use_fahrenheit).await?;

    // Fetch weather and alerts concurrently
    let weather_data = client
        .get_weather(latitude, longitude, &location_name, use_fahrenheit)
        .await?;

    // Get weather description
    let description = weather::weather_description(weather_data.weather_code);

    // Render to terminal
    renderer::render_weather(&weather_data, description, colors)
        .map_err(|e| AppError::invalid_arg(format!("Render error: {}", e)))?;

    // Fetch and display alerts (non-blocking, errors suppressed)
    if !no_alerts {
        if let Some(alert) = fetch_alerts(client, latitude, longitude).await {
            renderer::render_alert_line(&alert)
                .map_err(|e| AppError::invalid_arg(format!("Render error: {}", e)))?;
        }
    }

    Ok((weather_data, latitude, longitude))
}

/// Fetches alerts and returns the most severe one, or None on any error.
async fn fetch_alerts(client: &Client, lat: f64, lon: f64) -> Option<termcast::alerts::Alert> {
    match client.get_alerts(lat, lon).await {
        Ok(alerts) => alerts.into_iter().next(),
        Err(_) => None,
    }
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
        weather_data.use_fahrenheit,
    );

    cache::write_cache(cache_path, &entry)
}

/// Runs in ambient mode - compact output for shell prompts.
async fn run_ambient_mode(
    client: &Client,
    cache_path: &std::path::Path,
    location: Option<&str>,
    cache_ttl_minutes: u64,
    config_use_fahrenheit: Option<bool>,
    no_alerts: bool,
) -> Result<(), AppError> {
    let ttl_secs = (cache_ttl_minutes * 60) as i64;

    // Try to read from cache first
    if let Ok(Some(entry)) = cache::read_cache(cache_path) {
        // Check if cache is fresh
        if cache::is_cache_fresh(&entry, ttl_secs) {
            // Cache is fresh - output immediately (skip alerts when using cache)
            output_ambient_weather(entry.weather_code, entry.temperature, entry.use_fahrenheit, None);
            return Ok(());
        }
    }

    // Cache missing or stale - fetch fresh data
    let cfg = config::load_config(None);
    let (latitude, longitude, location_name, use_fahrenheit) =
        resolve_full_location(client, location, &cfg, config_use_fahrenheit).await?;

    // Fetch weather and alerts concurrently
    let weather_data = client
        .get_weather(latitude, longitude, &location_name, use_fahrenheit)
        .await?;

    // Get most severe alert if not suppressed
    let alert = if no_alerts {
        None
    } else {
        fetch_alerts(client, latitude, longitude).await
    };

    // Write to cache
    write_weather_cache(cache_path, &weather_data, latitude, longitude)?;

    // Output ambient format with optional alert indicator
    output_ambient_weather(
        weather_data.weather_code,
        weather_data.temperature,
        weather_data.use_fahrenheit,
        alert.as_ref(),
    );

    Ok(())
}

/// Outputs compact weather in format "☀️ 14°F" for shell prompts.
/// Appends a warning indicator " ⚠" when an alert is present.
fn output_ambient_weather(
    weather_code: u32,
    temperature: f64,
    use_fahrenheit: bool,
    alert: Option<&termcast::alerts::Alert>,
) {
    let icon = weather::weather_icon(weather_code);
    let temp = temperature as i32;
    let unit = if use_fahrenheit { "°F" } else { "°C" };
    let alert_suffix = match alert {
        Some(a) if a.severity == termcast::alerts::AlertSeverity::Warning => " ⚠",
        Some(_) => " ⚡",
        None => "",
    };
    println!("{}{} {}{}", icon, temp, unit, alert_suffix);
}

/// Runs the forecast subcommand: resolve location, fetch forecast, render output.
async fn run_forecast(
    args: &Args,
    days: u32,
    hourly: bool,
    ambient: bool,
) -> Result<(), AppError> {
    let config_path = args.config.as_deref().map(Path::new);
    let cfg = config::load_config(config_path);
    let client = Client::new();

    let config_use_fahrenheit = cfg.resolve_units();
    let location_query = resolve_location_query(args, &cfg);

    let (latitude, longitude, location_name, use_fahrenheit) =
        resolve_full_location(&client, location_query.as_deref(), &cfg, config_use_fahrenheit)
            .await?;

    let display = client
        .get_forecast(
            latitude,
            longitude,
            &location_name,
            use_fahrenheit,
            days,
            hourly,
        )
        .await?;

    if ambient {
        renderer::output_ambient_forecast(&display)
            .map_err(|e| AppError::invalid_arg(format!("Render error: {}", e)))?;
    } else {
        let colors = resolve_theme_colors(&cfg.defaults.theme);
        renderer::render_forecast(&display, &colors)
            .map_err(|e| AppError::invalid_arg(format!("Render error: {}", e)))?;
        if hourly {
            renderer::render_forecast_hourly(&display, &colors)
                .map_err(|e| AppError::invalid_arg(format!("Render error: {}", e)))?;
        }
    }

    Ok(())
}

/// Generates shell completion scripts.
fn generate_completions(shell: &str) -> Result<(), AppError> {
    let shell = match shell {
        "bash" => clap_complete::Shell::Bash,
        "zsh" => clap_complete::Shell::Zsh,
        "fish" => clap_complete::Shell::Fish,
        "elvish" => clap_complete::Shell::Elvish,
        "powershell" => clap_complete::Shell::PowerShell,
        _ => {
            return Err(AppError::invalid_arg(format!(
                "Unknown shell '{}'. Use: bash, zsh, fish, elvish, or powershell",
                shell
            )));
        }
    };

    let mut app = <Args as clap::CommandFactory>::command();
    let bin_name = "termcast".to_string();
    clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());

    Ok(())
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
    use crate::Args;
    use clap::Parser;

    #[test]
    fn test_args_parsing_no_location() {
        let args = Args::parse_from(&["termcast"]);
        assert!(args.location.is_none());
        assert!(args.at.is_none());
    }

    #[test]
    fn test_args_parsing_with_location() {
        let args = Args::parse_from(&["termcast", "-l", "Oslo"]);
        assert_eq!(args.location, Some("Oslo".to_string()));
        assert!(args.at.is_none());
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
        assert!(args.cache_ttl.is_none());
    }

    #[test]
    fn test_args_cache_ttl_custom() {
        let args = Args::parse_from(&["termcast", "--cache-ttl", "30"]);
        assert_eq!(args.cache_ttl, Some(30));
    }

    #[test]
    fn test_args_install_bash() {
        let args = Args::parse_from(&["termcast", "--install", "bash"]);
        assert_eq!(args.install, Some("bash".to_string()));
    }

    #[test]
    fn test_args_install_all() {
        let args = Args::parse_from(&["termcast", "--install"]);
        assert_eq!(args.install, Some("all".to_string()));
    }

    #[test]
    fn test_args_config_flag() {
        let args = Args::parse_from(&["termcast", "--config", "/tmp/myconfig.toml"]);
        assert_eq!(args.config, Some("/tmp/myconfig.toml".to_string()));
    }

    #[test]
    fn test_args_at_flag() {
        let args = Args::parse_from(&["termcast", "--at", "home"]);
        assert_eq!(args.at, Some("home".to_string()));
        assert!(args.location.is_none());
    }

    #[test]
    fn test_args_list_locations() {
        let args = Args::parse_from(&["termcast", "--list-locations"]);
        assert!(args.list_locations);
    }

    #[test]
    fn test_resolve_location_query_location_flag_wins() {
        let args = Args::parse_from(&["termcast", "--location", "Oslo", "--at", "home"]);
        let cfg = termcast::config::Config::default();
        let result = super::resolve_location_query(&args, &cfg);
        assert_eq!(result, Some("Oslo".to_string()));
    }

    #[test]
    fn test_resolve_location_query_at_flag_fallback() {
        let args = Args::parse_from(&["termcast", "--at", "Chicago"]);
        let cfg = termcast::config::Config::default();
        let result = super::resolve_location_query(&args, &cfg);
        assert_eq!(result, Some("Chicago".to_string()));
    }

    #[test]
    fn test_resolve_location_query_at_flag_resolves_saved() {
        let args = Args::parse_from(&["termcast", "--at", "home"]);
        let toml = r#"
[locations.home]
city = "Oslo"
latitude = 59.91
longitude = 10.75
"#;
        let cfg: termcast::config::Config = toml::from_str(toml).unwrap();
        let result = super::resolve_location_query(&args, &cfg);
        assert_eq!(result, Some("Oslo".to_string()));
    }

    #[test]
    fn test_resolve_location_query_config_default() {
        let args = Args::parse_from(&["termcast"]);
        let toml = "[defaults]\ndefault_location = \"Oslo\"\n";
        let cfg: termcast::config::Config = toml::from_str(toml).unwrap();
        let result = super::resolve_location_query(&args, &cfg);
        assert_eq!(result, Some("Oslo".to_string()));
    }

    #[test]
    fn test_resolve_location_query_config_default_auto() {
        let args = Args::parse_from(&["termcast"]);
        let cfg = termcast::config::Config::default();
        let result = super::resolve_location_query(&args, &cfg);
        assert!(result.is_none());
    }

    // --- Forecast subcommand tests ---

    #[test]
    fn test_forecast_default_args() {
        let args = Args::parse_from(&["termcast", "forecast"]);
        match args.command {
            Some(super::Command::Forecast {
                days,
                hourly,
                ambient,
            }) => {
                assert_eq!(days, 5);
                assert!(!hourly);
                assert!(!ambient);
            }
            _ => panic!("Expected Forecast subcommand"),
        }
    }

    #[test]
    fn test_forecast_days() {
        let args = Args::parse_from(&["termcast", "forecast", "--days", "3"]);
        match args.command {
            Some(super::Command::Forecast { days, .. }) => assert_eq!(days, 3),
            _ => panic!("Expected Forecast subcommand"),
        }
    }

    #[test]
    fn test_forecast_hourly() {
        let args = Args::parse_from(&["termcast", "forecast", "--hourly"]);
        match args.command {
            Some(super::Command::Forecast { hourly, .. }) => assert!(hourly),
            _ => panic!("Expected Forecast subcommand"),
        }
    }

    #[test]
    fn test_forecast_ambient() {
        let args = Args::parse_from(&["termcast", "forecast", "--ambient"]);
        match args.command {
            Some(super::Command::Forecast { ambient, .. }) => assert!(ambient),
            _ => panic!("Expected Forecast subcommand"),
        }
    }

    #[test]
    fn test_forecast_with_location() {
        let args = Args::parse_from(&["termcast", "-l", "Oslo", "forecast", "--days", "7", "--hourly"]);
        assert_eq!(args.location, Some("Oslo".to_string()));
        match args.command {
            Some(super::Command::Forecast {
                days,
                hourly,
                ambient,
            }) => {
                assert_eq!(days, 7);
                assert!(hourly);
                assert!(!ambient);
            }
            _ => panic!("Expected Forecast subcommand"),
        }
    }

    #[test]
    fn test_forecast_with_at() {
        let args = Args::parse_from(&["termcast", "--at", "home", "forecast"]);
        assert_eq!(args.at, Some("home".to_string()));
        match args.command {
            Some(super::Command::Forecast { .. }) => {}
            _ => panic!("Expected Forecast subcommand"),
        }
    }

    #[test]
    fn test_completions_still_works() {
        let args = Args::parse_from(&["termcast", "completions", "bash"]);
        match args.command {
            Some(super::Command::Completions { shell }) => {
                assert_eq!(shell, "bash");
            }
            _ => panic!("Expected Completions subcommand"),
        }
    }

    #[test]
    fn test_no_subcommand_still_works() {
        let args = Args::parse_from(&["termcast", "-l", "Oslo"]);
        assert!(args.command.is_none());
        assert_eq!(args.location, Some("Oslo".to_string()));
    }

    #[test]
    fn test_list_themes_flag() {
        let args = Args::parse_from(&["termcast", "--list-themes"]);
        assert!(args.list_themes);
    }

    #[test]
    fn test_preview_theme_flag() {
        let args = Args::parse_from(["termcast", "--preview-theme", "dracula"]);
        assert_eq!(args.preview_theme, Some("dracula".to_string()));
    }

    #[test]
    fn test_preview_theme_flag_missing() {
        let args = Args::parse_from(["termcast"]);
        assert!(!args.list_themes);
        assert!(args.preview_theme.is_none());
    }

    #[test]
    fn test_no_alerts_flag_default() {
        let args = Args::parse_from(&["termcast"]);
        assert!(!args.no_alerts);
    }

    #[test]
    fn test_no_alerts_flag_set() {
        let args = Args::parse_from(&["termcast", "--no-alerts"]);
        assert!(args.no_alerts);
    }

    #[test]
    fn test_no_alerts_with_location() {
        let args = Args::parse_from(&["termcast", "--no-alerts", "-l", "Tulsa, OK"]);
        assert!(args.no_alerts);
        assert_eq!(args.location, Some("Tulsa, OK".to_string()));
    }

    #[test]
    fn test_no_alerts_with_ambient() {
        let args = Args::parse_from(&["termcast", "--no-alerts", "--ambient"]);
        assert!(args.no_alerts);
        assert!(args.ambient);
    }
}
