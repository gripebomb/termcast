# Spec: TermCast Forecast

## Objective

Add multi-day and hourly forecast capabilities to TermCast via a `termcast forecast` subcommand. Users currently get only current conditions — this feature lets them see 1-7 day forecasts and hourly breakdowns for today without leaving the terminal.

**User stories:**
- As a terminal user, I run `termcast forecast` to see a 5-day table with highs, lows, icons, and precipitation probability
- As a terminal user, I run `termcast forecast --hourly` to see today's notable weather changes hour by hour
- As a terminal user, I run `termcast forecast --days 3 --location Oslo` to get a shorter forecast for a specific city
- As a tmux user, I run `termcast forecast --ambient` to get a one-line forecast for my status bar

## Tech Stack

- **Language:** Rust 2021 edition
- **HTTP client:** reqwest 0.12 (already in use)
- **CLI framework:** clap 4.5 with derive (already in use)
- **Terminal rendering:** crossterm 0.28 (already in use)
- **Weather data:** Open-Meteo `/v1/forecast` endpoint (already in use)
- **Serialization:** serde + serde_json (already in use)
- **Testing:** built-in `#[test]` + wiremock 0.6 for HTTP mocking (already in dev-deps)

## Commands

```bash
# Build
cargo build

# Run (dev)
cargo run -- forecast
cargo run -- forecast --hourly
cargo run -- forecast --days 3 --location "San Francisco"

# Test
cargo test

# Lint
cargo clippy

# Install locally
cargo install --path .
```

## Project Structure

```
src/
├── main.rs           # CLI entry point, argument parsing, subcommand dispatch
├── api.rs            # HTTP client (add get_forecast method)
├── weather.rs        # Current weather types (unchanged)
├── forecast.rs       # NEW — forecast types, hourly filter logic
├── geolocation.rs    # Geolocation data types (unchanged)
├── cache.rs          # Weather data caching (unchanged)
├── config.rs         # Configuration management (unchanged)
├── renderer.rs       # Terminal rendering (add forecast render functions)
├── errors.rs         # Structured error handling (unchanged)
└── lib.rs            # Library exports (add forecast module)
```

## Code Style

Follow existing patterns. Key conventions from the codebase:

```rust
// Serde structs with #[serde(rename)] for API field mapping
#[derive(Debug, Clone, Deserialize)]
pub struct ForecastResponse {
    #[serde(rename = "daily")]
    pub daily: DailyForecast,
}

// Descriptive error context
AppError::parse(&url, "forecast daily", e)

// Render functions use crossterm queue-based API
pub fn render_forecast(forecast: &ForecastDisplay) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(SetForegroundColor(Color::White))?;
    // ...
    stdout.flush()
}

// Tests use inline JSON fixtures
#[test]
fn test_parse_forecast_response() {
    let json = r#"{ ... }"#;
    let response: ForecastResponse = serde_json::from_str(json).unwrap();
}
```

## CLI Design

### Current CLI (unchanged behavior)

```
termcast                    # Current weather, auto-location
termcast -l Oslo            # Current weather for Oslo
termcast --ambient          # Compact current weather for prompts
termcast completions bash   # Shell completions
```

### New forecast subcommand

```
termcast forecast                     # 5-day forecast, auto-location
termcast forecast --days 3            # 3-day forecast
termcast forecast --days 7            # 7-day forecast (opt-in)
termcast forecast --hourly            # Today's hourly breakdown
termcast forecast --hourly --days 2   # Today + tomorrow hourly
termcast forecast -l Oslo             # 5-day forecast for Oslo
termcast forecast --at home           # 5-day forecast for saved location
termcast forecast --ambient           # One-line: tomorrow's high/low + next precip
termcast forecast --ambient --days 3  # One-line: 3-day summary
```

### Flag details

| Flag | Subcommand | Default | Description |
|------|-----------|---------|-------------|
| `--days` | forecast | 5 | Number of forecast days (1-7) |
| `--hourly` | forecast | false | Show hourly breakdown |
| `--location` / `-l` | forecast | auto | Location query |
| `--at` | forecast | none | Named or saved location |
| `--ambient` | forecast | false | Compact one-line output |
| `--config` | forecast | XDG default | Config file path |

## Data Types

### Open-Meteo API request

Current `get_weather()` URL:
```
https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true&daily=weather_code,temperature_2m_max,temperature_2m_min&temperature_unit={}&timezone=auto
```

Forecast `get_forecast()` URL:
```
https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max&hourly=temperature_2m,precipitation_probability,weather_code&temperature_unit={}&timezone=auto&forecast_days={}
```

Key additions:
- `precipitation_probability_max` in daily params
- `temperature_2m`, `precipitation_probability`, `weather_code` in hourly params
- `forecast_days` parameter to limit range

### New types (in `src/forecast.rs`)

```rust
/// Response from Open-Meteo forecast endpoint.
pub struct ForecastResponse {
    pub daily: DailyForecast,
    pub hourly: Option<HourlyForecast>,
}

pub struct DailyForecast {
    pub time: Vec<String>,           // "2024-01-15"
    pub weather_code: Vec<u32>,
    pub temperature_max: Vec<f64>,
    pub temperature_min: Vec<f64>,
    pub precipitation_probability: Vec<u32>,  // 0-100%
}

pub struct HourlyForecast {
    pub time: Vec<String>,           // "2024-01-15T00:00"
    pub temperature: Vec<f64>,
    pub precipitation_probability: Vec<u32>,
    pub weather_code: Vec<u32>,
}

/// Display-ready daily forecast row.
pub struct DailyRow {
    pub day_name: String,            // "Mon", "Today", "Tomorrow"
    pub weather_code: u32,
    pub temp_high: f64,
    pub temp_low: f64,
    pub precip_chance: u32,          // 0-100%
}

/// Display-ready hourly entry (after filtering).
pub struct HourlyEntry {
    pub time: String,                // "2pm"
    pub weather_code: u32,
    pub temperature: f64,
    pub precip_chance: u32,
}

/// Complete forecast data for rendering.
pub struct ForecastDisplay {
    pub location: String,
    pub use_fahrenheit: bool,
    pub daily: Vec<DailyRow>,
    pub hourly: Vec<HourlyEntry>,    // Empty if not requested
}
```

## Rendering

### Daily forecast table

80-column centered output, matching the existing aesthetic:

```
        Forecast for Oslo

  Mon   ☀️  17°/8°    ☂ 5%
  Tue   🌤 15°/7°    ☂ 10%
  Wed   🌧 12°/6°    ☂ 75%
  Thu   🌧 10°/4°    ☂ 80%
  Fri   🌤 14°/5°    ☂ 15%
```

- Day name: bold white (cyan for "Today", magenta for "Tomorrow")
- Icon: from existing `weather_icon()` mapping
- High: cyan, Low: magenta (consistent with current high/low styling)
- Precip chance: white with umbrella prefix, dim if < 30%, yellow if >= 50%, red if >= 80%

### Hourly breakdown

```
        Today's Forecast — Oslo

  9am   ☀️  11°    ☂ 5%
  12pm  🌤 14°    ☂ 10%
  3pm   🌧 13°    ☂ 70%    Rain starting
  6pm   🌧 11°    ☂ 60%
  9pm   🌤 10°    ☂ 15%    Rain clearing
```

- Only shows hours where something notable changes (precip starts/stops, temp shift > 5)
- Annotations in dim grey for change descriptions
- Always includes first and last hour of the day for context

### Ambient forecast

```
🌤 15°/7° ☂ 70% Wed
```

Format: `[tomorrow_icon] [high]/[low] ☂ [precip]% [day_name]`

## Hourly Filter Logic

The `--hourly` flag does not show all 24 hours. It filters to show only notable changes:

1. Always include the first hour (6am) and last hour (11pm) for context
2. Include hours where precipitation probability changes by > 20 percentage points from the previous included hour
3. Include hours where temperature changes by > 5 degrees from the previous included hour
4. Include the hour with the day's maximum precipitation probability
5. Merge entries that are less than 2 hours apart (keep the more notable one)

## Testing Strategy

- **Unit tests** in each module's `#[cfg(test)]` block (existing pattern)
- **Parsing tests**: JSON fixtures for `ForecastResponse` deserialization
- **Filter tests**: Hourly data to filtered entries, covering edge cases (no notable changes, all notable changes, boundary conditions)
- **Rendering tests**: Verify `render_forecast` and `render_forecast_hourly` do not panic (same pattern as existing renderer tests)
- **Integration tests**: Use `wiremock` to mock Open-Meteo responses and test the full fetch/parse/render pipeline
- No strict coverage target — test the logic that can fail (parsing, filtering) rather than rendering visuals

## Boundaries

**Always:**
- Run `cargo test` and `cargo clippy` before considering work done
- Follow existing naming conventions and error patterns
- Validate `--days` is in range 1-7
- Use existing `weather_icon()` and `weather_description()` functions — don't duplicate icon mappings
- Share location resolution code between current weather and forecast paths

**Ask first:**
- Adding new dependencies to `Cargo.toml`
- Changing the existing `WeatherResponse` or `DailyWeather` structs
- Modifying the current `termcast` (no subcommand) behavior

**Never:**
- Break backwards compatibility of existing CLI flags
- Add a second weather API source (OWM, etc.)
- Add wind, UV, humidity, or other non-MVP variables
- Modify files unrelated to the forecast feature
- Create daemon/background process functionality

## Success Criteria

- [ ] `termcast forecast` renders a styled 5-day table with day name, icon, high/low, precip chance
- [ ] `termcast forecast --hourly` renders a compact hourly view showing only notable changes
- [ ] `termcast forecast --days N` accepts 1-7 and validates the range
- [ ] `termcast forecast --ambient` outputs a single-line forecast for prompts
- [ ] `termcast forecast` with no location falls back to IP geolocation (same as current behavior)
- [ ] `termcast forecast -l Oslo` and `termcast forecast --at home` work for explicit locations
- [ ] Existing `termcast` behavior (no subcommand) is completely unchanged
- [ ] `cargo test` passes with new tests for parsing and hourly filter logic
- [ ] `cargo clippy` passes with no warnings
- [ ] Shell completions still work (`termcast completions bash`)

## Open Questions

- ~~Should `termcast forecast` with no location fall back to auto-detected IP location?~~ **Resolved: Yes, same as current behavior.**
- ~~Should the hourly view cover today only?~~ **Resolved: Today only for MVP. `--hourly --days 2` can show today + tomorrow.**
- ~~What's the right threshold for notable temperature changes?~~ **Resolved: 5 degrees for temp, 20% for precip probability.**
