# Spec: TermCast Severe Weather Alerts (NWS)

## Objective

Display active NWS severe weather alerts in termcast's terminal output for US locations. When active alerts exist for the user's location, show a color-coded compact alert line below the weather display. In ambient mode, append a warning indicator. For non-US locations, nothing changes.

**User stories:**
- As a US-based terminal user, I run `termcast` and see `⚠ Tornado Warning until 6:00 PM` in bold red below my weather
- As a US-based terminal user, I run `termcast --ambient` and see `☀️ 14°C ⚠` when alerts are active
- As a non-US terminal user, I notice no change — alerts are silently skipped
- As any terminal user, I run `termcast --no-alerts` to suppress alert display

## Tech Stack

- **Language:** Rust 2021 edition
- **HTTP client:** reqwest 0.12 (already in use) — reuses existing Client
- **CLI framework:** clap 4.5 with derive (already in use)
- **Terminal rendering:** crossterm 0.28 (already in use)
- **Alert data:** NWS API `https://api.weather.gov/alerts/active?point={lat},{lon}` — free, no API key
- **Serialization:** serde + serde_json (already in use)
- **Time formatting:** chrono 0.4 (already in use)
- **Testing:** built-in `#[test]` + wiremock 0.6 for HTTP mocking (already in dev-deps)
- **No new crate dependencies**

## Commands

```bash
# Build
cargo build

# Run (dev) — alerts appear automatically for US locations
cargo run -- -l "Tulsa, OK"
cargo run -- -l Oslo           # No alerts (non-US)

# Suppress alerts
cargo run -- -l "Tulsa, OK" --no-alerts

# Test
cargo test

# Lint
cargo clippy
```

## Project Structure

```
src/
├── main.rs           # CLI entry point (add --no-alerts flag, wire alert fetch/display)
├── alerts.rs         # NEW — NWS API response types, alert parsing, severity mapping
├── api.rs            # HTTP client (add get_alerts method)
├── weather.rs        # Current weather types (unchanged)
├── forecast.rs       # Forecast types (unchanged)
├── geolocation.rs    # Geolocation data types (unchanged)
├── cache.rs          # Weather data caching (unchanged)
├── config.rs         # Configuration management (unchanged)
├── renderer.rs       # Terminal rendering (add render_alert_line)
├── errors.rs         # Structured error handling (unchanged)
├── theme.rs          # Color theme engine (unchanged — alert colors are fixed)
└── lib.rs            # Library exports (add alerts module)
```

## Code Style

Follow existing patterns. Key conventions:

```rust
// Serde structs with #[serde(rename)] for NWS API field mapping
#[derive(Debug, Clone, Deserialize)]
pub struct NwsAlertsResponse {
    pub features: Vec<AlertFeature>,
}

// Alert severity enum with ordering
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Advisory,  // lowest
    Watch,
    Warning,   // highest
}

// Graceful error handling — alerts never block weather display
async fn fetch_alerts(client: &Client, lat: f64, lon: f64) -> Option<Alert> {
    match client.get_alerts(lat, lon).await {
        Ok(alerts) => alerts.into_iter().max_by_key(|a| a.severity.clone()),
        Err(_) => None, // silently ignore
    }
}

// Render function uses crossterm queue-based API (same pattern as renderer.rs)
pub fn render_alert_line(alert: &Alert) -> io::Result<()> {
    let mut stdout = io::stdout();
    // color based on severity...
    stdout.flush()
}
```

## NWS API Details

### Endpoint

```
GET https://api.weather.gov/alerts/active?point={latitude},{longitude}
```

- Free, no API key required
- Returns empty `features` array for non-US locations (natural geo-filter)
- Requires `User-Agent` header (already sent by existing Client)
- Response format: GeoJSON with CAP v1.2 alert data

### Response structure (relevant fields only)

```json
{
  "features": [
    {
      "properties": {
        "event": "Tornado Warning",
        "severity": "Extreme",
        "urgency": "Immediate",
        "expires": "2026-04-13T23:00:00-05:00",
        "headline": "Tornado Warning issued April 13 at 5:30PM CDT"
      }
    }
  ]
}
```

### Severity mapping

| NWS `severity` field | termcast `AlertSeverity` | Display color |
|---|---|---|
| "Extreme", "Severe" | Warning | Bold red (`Color::Rgb { r: 255, g: 59, b: 48 }`) |
| "Moderate" | Watch | Yellow (`Color::Rgb { r: 255, g: 204, b: 0 }`) |
| "Minor", "Unknown" | Advisory | Dim yellow (`Color::Rgb { r: 204, g: 170, b: 0 }`) |

### Time formatting

Extract time from `expires` field (ISO 8601 with timezone). Format as `h:MM AM/PM` in the alert's local timezone. Use chrono's `DateTime::parse_from_rfc3339` or `NaiveDateTime` parsing.

## Data Types

### New types (in `src/alerts.rs`)

```rust
/// NWS API GeoJSON response for active alerts.
pub struct NwsAlertsResponse {
    pub features: Vec<AlertFeature>,
}

/// A single alert feature from NWS API.
pub struct AlertFeature {
    pub properties: AlertProperties,
}

/// Alert properties extracted from NWS CAP data.
pub struct AlertProperties {
    pub event: String,          // "Tornado Warning"
    pub severity: String,       // "Extreme", "Severe", "Moderate", "Minor"
    pub expires: String,        // ISO 8601 datetime
}

/// Display-ready alert for rendering.
pub struct Alert {
    pub event: String,          // "Tornado Warning"
    pub severity: AlertSeverity,
    pub expires_time: String,   // "6:00 PM" (formatted for display)
}
```

## Rendering

### Normal mode — alert line below weather

When an active alert exists, render a single centered line below the existing 4-line weather display:

```
     ☀️ 72°F Tulsa, OK
   Feels 68°F
   High 78° · Low 55°
   Clear
   ⚠ Tornado Warning until 6:00 PM
```

- Warning severity: bold red text (`SetAttribute(Bold)` + `SetForegroundColor(red)`)
- Watch severity: yellow text (no bold)
- Advisory severity: dim yellow text
- Centered in 80 columns (same as weather display)
- Icon: `⚠` prefix for all severities
- Time format: "until H:MM AM/PM" using the alert's local timezone
- If the alert has no `expires` field or parsing fails, show `⚠ Tornado Warning` without time

### Ambient mode — warning indicator

When alerts are active, append `⚠` to the ambient output:

```
☀️ 72°F ⚠
```

No severity distinction in ambient mode — just the presence of any active alert.

### No alerts — invisible

When no active alerts exist (empty features, non-US location, or NWS API failure), no additional output is rendered. The weather display is identical to current behavior.

## CLI Design

### New flag

```
termcast --no-alerts          # Suppress alert display
termcast -l "Tulsa" --no-alerts
```

| Flag | Default | Description |
|------|---------|-------------|
| `--no-alerts` | false (alerts shown) | Suppress NWS alert display |

Alerts are **on by default** for all locations. The NWS API returns empty results for non-US, so the flag is rarely needed. It exists for users who want to opt out (e.g., slow networks, offline usage).

### Forecast subcommand — no alert changes

The `termcast forecast` subcommand does **not** display alerts in MVP. Alert display is limited to current weather and ambient mode only.

## Integration Flow

### Normal mode (current weather)

```
1. Resolve location → (lat, lon, name, use_fahrenheit)
2. Fetch weather data from Open-Meteo (existing)
3. Concurrently: fetch alerts from NWS via ?point={lat},{lon}
4. Render weather (existing 4 lines)
5. If alert exists: render alert line below
6. Cache weather data (existing)
```

Use `tokio::join!` to fetch weather and alerts concurrently:

```rust
let (weather_result, alert) = tokio::join!(
    client.get_weather(lat, lon, &name, use_fahrenheit),
    fetch_alerts(&client, lat, lon)
);
```

This means NWS latency does not add to total response time — both requests run in parallel.

### Ambient mode

```
1. Try cache (existing)
2. If stale: fetch weather + alerts concurrently
3. Output: icon + temp + unit + (⚠ if alert active)
```

### Error handling

Alert failures are **always silent**:
- NWS API timeout → no alert displayed, no error message
- NWS API returns 4xx/5xx → no alert displayed
- JSON parsing failure → no alert displayed
- Network unreachable → no alert displayed

Alerts must never prevent or delay weather display. The `fetch_alerts` function returns `Option<Alert>`, where `None` means "no alerts or fetch failed."

## API Client Design

### New method on `Client` (in `src/api.rs`)

```rust
/// Fetches active NWS alerts for a point location.
///
/// Returns a list of active alerts, or empty if none exist.
/// Failures are handled upstream — this method propagates errors
/// so the caller can decide to suppress them.
pub async fn get_alerts(&self, lat: f64, lon: f64) -> Result<Vec<Alert>, AppError>
```

The method:
1. Constructs URL: `https://api.weather.gov/alerts/active?point={lat},{lon}`
2. Sends GET with `User-Agent: termcast/0.1.0` header
3. Parses JSON response into `NwsAlertsResponse`
4. Converts each feature into an `Alert`, filtering by expiry (skip already-expired alerts)
5. Returns the list

### Alert filtering

After fetching all alerts:
1. Filter out expired alerts (compare `expires` to `Utc::now()`)
2. Convert NWS severity strings to `AlertSeverity` enum
3. Return the highest-severity alert for display (MVP shows one alert only)

## Testing Strategy

- **Unit tests** in each module's `#[cfg(test)]` block (existing pattern)
- **Parsing tests**: JSON fixtures for `NwsAlertsResponse` deserialization, covering:
  - Empty features array (no alerts)
  - Single alert with each severity level
  - Missing or malformed `expires` field
  - Multiple alerts with different severities
- **Severity mapping tests**: Verify NWS string to `AlertSeverity` conversion
- **Alert filtering tests**: Verify expired alerts are removed, highest severity is selected
- **Time formatting tests**: Verify ISO 8601 to "H:MM AM/PM" conversion
- **Rendering tests**: Verify `render_alert_line` does not panic for each severity
- **Integration tests**: Use `wiremock` to mock NWS API responses
- No strict coverage target — test parsing and filtering logic thoroughly

### Test fixtures

```rust
// Minimal NWS alert response for testing
let nws_response = r#"{
    "features": [{
        "properties": {
            "event": "Tornado Warning",
            "severity": "Extreme",
            "urgency": "Immediate",
            "expires": "2026-04-13T23:00:00-05:00",
            "headline": "Tornado Warning issued..."
        }
    }]
}"#;

// Empty response (non-US or no active alerts)
let empty_response = r#"{"features": []}"#;
```

## Boundaries

**Always:**
- Run `cargo test` and `cargo clippy` before considering work done
- Follow existing naming conventions and error patterns
- Handle all NWS API failures gracefully (silent, never block weather)
- Use `tokio::join!` for concurrent fetch (alerts must not add latency)
- Keep alert colors fixed (not themeable) — use hardcoded RGB values

**Ask first:**
- Adding new dependencies to `Cargo.toml`
- Changing the existing `WeatherDisplay`, `Client`, or `AppError` types
- Modifying the existing weather rendering layout (4-line format)
- Adding alert display to the forecast subcommand

**Never:**
- Break backwards compatibility of existing CLI flags
- Add a second weather data API (only alerts come from NWS)
- Show alerts for non-US locations (NWS handles this via empty response)
- Create daemon/background process functionality for alert monitoring
- Add full alert descriptions to the terminal output (event name + time only)
- Show multiple simultaneous alerts (highest severity only for MVP)
- Make alert colors themeable through the existing theme system

## Success Criteria

- [ ] `termcast -l "Tulsa, OK"` shows a color-coded alert line when NWS has active alerts for that location
- [ ] `termcast -l Oslo` shows no alert line (non-US, NWS returns empty)
- [ ] `termcast --no-alerts` suppresses all alert display
- [ ] `termcast --ambient` appends `⚠` when alerts are active, plain output when not
- [ ] NWS API failure (timeout, 5xx, parse error) results in no alert line and no error message
- [ ] Existing `termcast` behavior (4-line weather display) is unchanged when no alerts exist
- [ ] `termcast forecast` behavior is unchanged (no alerts in forecast output)
- [ ] `cargo test` passes with new tests for alert parsing, severity mapping, and filtering
- [ ] `cargo clippy` passes with no warnings
- [ ] Weather + alerts fetch concurrently — NWS latency does not increase total response time
- [ ] Alert colors render correctly in true-color terminals (RGB) and fall back gracefully in 256-color

## Open Questions

- Should we cache alert data alongside weather data in ambient mode? (Currently: re-fetch on every invocation. Caching would reduce NWS load but risks stale alerts.)
- Should `--forecast` show alert indicators in a future iteration? (MVP: no. Deferred.)
- What happens if the user's clock is wrong and an alert appears expired when it's not? (Accept this edge case — use client time for expiry check.)
