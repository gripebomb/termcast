# Implementation Plan: Severe Weather Alerts (NWS)

## Overview

Add NWS severe weather alert display to termcast. When active alerts exist for a US location, show a color-coded alert line below the weather display (normal mode) or append a warning indicator (ambient mode). Non-US locations are unaffected. Alerts are fetched concurrently with weather data so they add zero latency.

## Architecture Decisions

- **New module `alerts.rs`** — houses NWS response types, severity enum, parsing, and filtering logic. Follows the existing pattern where each domain has its own file (weather.rs, forecast.rs, geolocation.rs).
- **`get_alerts` method on existing `Client`** — follows the same HTTP pattern as `get_weather` and `get_forecast`. Reuses the existing `reqwest::Client` with its 10-second timeout and User-Agent header.
- **Alerts return `Option<Alert>` to the caller** — all NWS failures are silently swallowed. The `fetch_alerts` helper in main.rs wraps the `Result` from `Client::get_alerts` into `Option<Alert>`, keeping the API client's error handling consistent while ensuring alerts never block weather display.
- **Concurrent fetch with `tokio::join!`** — weather and alerts fetch in parallel. This is a spec requirement to avoid adding latency.
- **Fixed alert colors (not themeable)** — hardcoded RGB values as specified. Warning = bold red, Watch = yellow, Advisory = dim yellow. Falls back gracefully in 256-color terminals via crossterm's built-in approximation.
- **No new crate dependencies** — everything needed (reqwest, serde, chrono, crossterm) is already in Cargo.toml.

## Dependency Graph

```
alerts.rs (types + parsing + severity mapping)
    |
    +-- api.rs (get_alerts method)
    |       |
    |       +-- main.rs (fetch_alerts helper, concurrent fetch wiring)
    |               |
    |               +-- renderer.rs (render_alert_line, ambient warning indicator)
    |
    +-- lib.rs (pub mod alerts)
```

## Task List

### Phase 1: Foundation — Alert Data Types and Parsing

- [ ] **Task 1: Create `src/alerts.rs` with NWS types, severity mapping, and parsing logic**

**Description:** Create the new alerts module with serde structs for NWS API deserialization, the `AlertSeverity` enum with ordering, severity string mapping, and the `Alert` display type. Include time formatting logic to convert ISO 8601 `expires` to "H:MM AM/PM".

**Acceptance criteria:**
- [ ] `NwsAlertsResponse`, `AlertFeature`, `AlertProperties` structs derive `Deserialize` with correct field mapping for NWS API
- [ ] `AlertSeverity` enum has `Advisory < Watch < Warning` ordering via `Ord` derive
- [ ] `map_severity("Extreme")` returns `Warning`, `map_severity("Severe")` returns `Warning`, `map_severity("Moderate")` returns `Watch`, `map_severity("Minor")` returns `Advisory`, unknown strings return `Advisory`
- [ ] `format_expiry` correctly parses ISO 8601 timestamps and formats to "H:MM AM/PM"
- [ ] `format_expiry` returns `None` for malformed/missing expiry strings
- [ ] Unit tests pass for: severity mapping (all 4 NWS values + unknown), time formatting (valid ISO, malformed, empty), deserialization of sample NWS JSON

**Verification:**
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

**Dependencies:** None

**Files likely touched:**
- `src/alerts.rs` (new file)
- `src/lib.rs` (add `pub mod alerts`)

**Estimated scope:** S (1-2 files)

---

- [ ] **Task 2: Add `get_alerts` method to `Client` in `src/api.rs`**

**Description:** Add a `get_alerts(&self, lat: f64, lon: f64) -> Result<Vec<Alert>, AppError>` method to the existing `Client` struct. Follows the same HTTP pattern as `get_weather`: construct URL, send GET with User-Agent header, check status, parse JSON. Include alert filtering (remove expired alerts) and severity mapping.

**Acceptance criteria:**
- [ ] Method constructs correct URL: `https://api.weather.gov/alerts/active?point={lat},{lon}`
- [ ] Sends `User-Agent: termcast/0.1.0` header
- [ ] Returns `Ok(vec![])` for empty features (no alerts / non-US)
- [ ] Filters out already-expired alerts (comparing `expires` to `Utc::now()`)
- [ ] Maps NWS severity strings to `AlertSeverity` enum
- [ ] Formats expiry times to display strings
- [ ] Returns `AppError` on network/parse failures (caller handles suppression)
- [ ] Unit tests with wiremock for: empty response, single alert, multiple alerts, expired filtering

**Verification:**
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

**Dependencies:** Task 1

**Files likely touched:**
- `src/api.rs` (add `use crate::alerts::*` and `get_alerts` method)

**Estimated scope:** S (1 file)

---

### Checkpoint: Foundation
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] Alert types, parsing, and API method work in isolation

---

### Phase 2: Rendering

- [ ] **Task 3: Add `render_alert_line` to `src/renderer.rs`**

**Description:** Add a `render_alert_line(alert: &Alert) -> io::Result<()>` function that renders a single centered alert line using crossterm's queue-based API. Color and bold styling based on severity. Follows the same 80-column centering pattern as `render_weather`.

**Acceptance criteria:**
- [ ] Warning severity renders in bold red (`Color::Rgb { r: 255, g: 59, b: 48 }`)
- [ ] Watch severity renders in yellow (`Color::Rgb { r: 255, g: 204, b: 0 }`)
- [ ] Advisory severity renders in dim yellow (`Color::Rgb { r: 204, g: 170, b: 0 }`)
- [ ] Output format: `warning_icon {event} until {time}` centered in 80 columns
- [ ] When `expires_time` is `None`, output format: `warning_icon {event}` (no "until")
- [ ] Uses crossterm `QueueableCommand` pattern (same as existing render functions)
- [ ] Resets attributes after rendering
- [ ] Rendering tests confirm no panics for each severity level

**Verification:**
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

**Dependencies:** Task 1

**Files likely touched:**
- `src/renderer.rs` (add `render_alert_line`, add tests)

**Estimated scope:** S (1 file)

---

### Checkpoint: Rendering
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] Alert line renders correctly for each severity (visual spot-check)

---

### Phase 3: Integration — Wire Everything Together

- [ ] **Task 4: Add `--no-alerts` CLI flag and wire alert fetching into `main.rs`**

**Description:** Add the `--no-alerts` flag to the `Args` struct. Create a `fetch_alerts` helper that wraps `Client::get_alerts` and returns `Option<Alert>` (suppressing all errors). Modify `fetch_and_display_weather` to fetch weather and alerts concurrently using `tokio::join!`, then render the alert line after weather. Modify `run_ambient_mode` to fetch alerts and append the warning indicator to output. Skip alert fetching when `--no-alerts` is set.

**Acceptance criteria:**
- [ ] `--no-alerts` flag parsed correctly (default: false, alerts shown)
- [ ] Normal mode: weather and alerts fetch concurrently via `tokio::join!`
- [ ] Normal mode: alert line renders below weather when alert exists
- [ ] Normal mode: no alert line when no alerts or fetch fails
- [ ] Normal mode: `--no-alerts` suppresses alert line entirely
- [ ] Ambient mode: warning indicator appended to output when alerts active
- [ ] Ambient mode: `--no-alerts` suppresses indicator even if alerts exist
- [ ] Forecast subcommand: unchanged (no alerts in forecast output)
- [ ] Existing weather display unchanged when no alerts exist
- [ ] CLI parsing tests for `--no-alerts` flag

**Verification:**
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] `cargo run -- -l "Tulsa, OK"` shows weather (with or without alert depending on NWS)
- [ ] `cargo run -- -l "Tulsa, OK" --no-alerts` shows weather without alert line
- [ ] `cargo run -- -l Oslo` shows weather without alert line (non-US)
- [ ] `cargo run -- --ambient` output format correct with/without alerts

**Dependencies:** Task 2, Task 3

**Files likely touched:**
- `src/main.rs` (add `--no-alerts` flag, `fetch_alerts` helper, modify `fetch_and_display_weather`, modify `run_ambient_mode`, modify `output_ambient_weather`)

**Estimated scope:** M (1 file, significant changes to flow)

---

### Checkpoint: Complete
- [ ] All acceptance criteria from spec met
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] Manual smoke test with US and non-US locations
- [ ] Manual smoke test with `--no-alerts`
- [ ] Manual smoke test with `--ambient`

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| NWS API changes response format | Med -- alerts stop displaying | All failures are silent; weather display unaffected. Parsing tests catch regressions. |
| NWS API slow/timeout | Low -- 10s timeout already set on client | Alerts fetched concurrently; timeout does not delay weather. `fetch_alerts` returns `None` on any error. |
| `expires` field format varies | Low -- time parsing fails | Fallback: show alert without time. Covered by `format_expiry` returning `None` for malformed input. |
| crossterm RGB not supported | Low -- colors look wrong in limited terminals | Crossterm handles fallback internally. Spec says "fall back gracefully in 256-color." |

## Open Questions

- (None -- all resolved in spec)
