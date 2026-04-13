# Implementation Plan: TermCast

## Overview

Build a weather CLI tool in Rust with beautiful terminal output. The project is greenfield — no source files exist yet. Will implement the core functionality following the dependency graph: errors → data types → API client → CLI → renderer.

## Architecture Decisions

1. **Async runtime via `tokio`** — Required for `reqwest` async features, lightweight for CLI
2. **`thiserror` for errors** — Provides structured error types with `?` operator support
3. **Crossterm for rendering** — Cross-platform terminal capabilities with ANSI support
4. **Minimal dependencies** — Only what's specified in SPEC.md to keep binary small

## Task List

### Phase 1: Project Foundation

- [ ] **Task 1: Initialize Rust project structure**
  - Create `Cargo.toml` with all dependencies (crossterm, reqwest, serde, serde_json, tokio, thiserror, clap)
  - Set Rust edition to 2021
  - Create `src/` directory structure matching SPEC.md

- [ ] **Task 2: Implement error types**
  - Create `src/errors.rs` with `AppError` enum
  - Include variants: `NetworkError`, `ParseError`, `GeolocationError`, `WeatherError`
  - Implement `std::error::Error` and `Display` traits via thiserror
  - Add context to errors (URL, field names, etc.)

**Acceptance criteria:**
- [ ] `AppError` variants cover all failure modes from SPEC.md
- [ ] Errors are descriptive for debugging (include URLs, response fields)
- [ ] `cargo build` compiles without warnings

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo clippy` shows no warnings

**Dependencies:** None

**Files touched:**
- `Cargo.toml`
- `src/errors.rs`

**Estimated scope:** XS

---

- [ ] **Task 3: Implement data types for API responses**
  - Create `src/geolocation.rs` — `GeoResponse` struct for ipapi.co
    - Fields: `latitude`, `longitude`, `city`, `country`
  - Create `src/weather.rs` — `WeatherResponse` struct for Open-Meteo
    - Fields: `temperature_2m`, `apparent_temperature`, `weather_code`
    - Daily fields: `temperature_2m_max`, `temperature_2m_min`
    - Daily fields: `weather_code`, `time`
  - Use `#[serde(rename = "...")]` for camelCase JSON fields

**Acceptance criteria:**
- [ ] Structs deserialize from actual API response JSON
- [ ] All required fields present for 4-line output (current temp, feels, high/low, condition)
- [ ] `#[serde(default)]` on optional fields

**Verification:**
- [ ] Unit tests parse sample JSON responses
- [ ] `cargo test` passes on parsing tests

**Dependencies:** Task 2 (errors module)

**Files touched:**
- `src/geolocation.rs`
- `src/weather.rs`

**Estimated scope:** S

---

### Phase 2: API Integration

- [ ] **Task 4: Implement HTTP client for geolocation**
  - Create `src/api.rs` with geolocation function
  - Fetch from `https://ipapi.co/json/`
  - Parse response into `GeoResponse`
  - Return `Result<(f64, f64, String), AppError>`
  - Include User-Agent header (required by ipapi.co)

**Acceptance criteria:**
- [ ] Returns (latitude, longitude, city_name) tuple
- [ ] Handles network failures gracefully
- [ ] Handles invalid JSON gracefully
- [ ] Includes timeout (10 seconds)

**Verification:**
- [ ] Manual test: `curl` the endpoint and verify structure
- [ ] Unit test with mocked response

**Dependencies:** Task 2, Task 3

**Files touched:**
- `src/api.rs`

**Estimated scope:** S

---

- [ ] **Task 5: Implement HTTP client for weather**
  - Fetch from Open-Meteo API
  - Parameters: `latitude`, `longitude`, `current_weather=true`, `daily=weather_code,temperature_2m_max,temperature_2m_min`, `timezone=auto`
  - Parse response into weather data
  - Return current temp, feels-like, high, low, weather code, daily summary

**Acceptance criteria:**
- [ ] Returns all data needed for 4-line output
- [ ] Handles API error responses (non-200)
- [ ] Handles malformed JSON
- [ ] Includes timeout

**Verification:**
- [ ] Unit test with mocked Open-Meteo response
- [ ] Parse tests for weather codes

**Dependencies:** Task 2, Task 3, Task 4

**Files touched:**
- `src/api.rs`

**Estimated scope:** S

---

### Phase 3: CLI and Rendering

- [ ] **Task 6: Implement CLI argument parsing**
  - Use `clap` with `derive` macro
  - Arguments: `--location`, `-l` (optional string)
  - Arguments: `--help`, `--version`
  - Version from `Cargo.toml`

**Acceptance criteria:**
- [ ] `--location` accepts city names like "Oslo" or "San Francisco, CA"
- [ ] `--help` shows usage info
- [ ] `--version` shows version from Cargo.toml

**Verification:**
- [ ] `./target/release/termcast --help` works
- [ ] `./target/release/termcast --location Oslo` parses correctly

**Dependencies:** Task 2

**Files touched:**
- `src/main.rs`

**Estimated scope:** XS

---

- [ ] **Task 7: Implement terminal renderer**
  - Create `src/renderer.rs`
  - Function: `render_weather(weather: WeatherDisplay)`
  - Weather icon mapping based on WMO weather codes (0, 1-3, 45-48, 51-67, 71-77, 80-82, 95-99)
  - Unicode symbols: ☀️, 🌤, 🌫, 🌧, ❄, 🌦, ⛈, ☁
  - Styled output using crossterm:
    - Location: bold white
    - Temperature: bright white
    - Feels: dim
    - High/Low: cyan/magenta or styled consistently
    - Condition: normal

**Acceptance criteria:**
- [ ] Output matches SPEC.md format exactly:
  ```
       ☁ 14°C Oslo
     Feels 11°
     High 17° · Low 8°
     Clear until evening
  ```
- [ ] ANSI codes present for color
- [ ] Works in iTerm2 and macOS Terminal
- [ ] Centered on an 80-column terminal

**Verification:**
- [ ] Manual test: run and screenshot output
- [ ] Verify ANSI codes with `cat -v` or `od`
- [ ] Test with `--location` flag

**Dependencies:** Task 3, Task 5, Task 6

**Files touched:**
- `src/renderer.rs`

**Estimated scope:** M

---

### Phase 4: Integration and Polish

- [ ] **Task 8: Wire up main.rs**
  - Parse CLI arguments
  - Get location (from --location or geolocation API)
  - Fetch weather data
  - Render output
  - Handle errors with styled error messages

**Acceptance criteria:**
- [ ] `./target/release/termcast` works with auto-detected location
- [ ] `./target/release/termcast --location Oslo` works
- [ ] Network errors show styled error message
- [ ] API errors show styled error message

**Verification:**
- [ ] Test without internet (should show styled error)
- [ ] Test with valid --location
- [ ] Test with invalid location

**Dependencies:** Task 4, Task 5, Task 6, Task 7

**Files touched:**
- `src/main.rs`

**Estimated scope:** S

---

- [ ] **Task 9: Add weather code to text description**
  - Add a `description()` method to weather codes
  - Map codes to readable strings (e.g., "Clear", "Partly Cloudy", "Light Rain")

**Acceptance criteria:**
- [ ] Daily forecast line shows meaningful text
- [ ] All WMO codes have reasonable descriptions

**Verification:**
- [ ] Output includes descriptive text, not just icon

**Dependencies:** Task 7

**Files touched:**
- `src/weather.rs` or `src/renderer.rs`

**Estimated scope:** XS

---

### Phase 5: Testing

- [ ] **Task 10: Write unit tests**
  - Test JSON parsing in `tests/` directory
  - Test weather code to icon mapping
  - Test weather code to description mapping
  - Mock HTTP responses using `reqwest` mock or similar

**Acceptance criteria:**
- [ ] All parsing tests pass
- [ ] All mapping tests pass
- [ ] No live API calls in tests

**Verification:**
- [ ] `cargo test` passes 100%

**Dependencies:** All previous tasks

**Files touched:**
- `tests/api_tests.rs`
- Inline `#[cfg(test)]` modules

**Estimated scope:** M

---

- [ ] **Task 11: Code quality checks**
  - Run `cargo fmt --check`
  - Run `cargo clippy`
  - Fix any warnings or style issues

**Acceptance criteria:**
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` passes with no warnings

**Verification:**
- [ ] Commands run without errors

**Dependencies:** All implementation tasks

**Estimated scope:** XS

---

## Checkpoints

### Checkpoint: After Tasks 1-3 (Foundation)
- [ ] Project structure exists
- [ ] `cargo check` compiles
- [ ] Error types are defined
- [ ] Data types parse correctly

### Checkpoint: After Tasks 4-5 (API Layer)
- [ ] Geolocation works (or mocks correctly)
- [ ] Weather API integration complete
- [ ] Unit tests for parsing pass

### Checkpoint: After Tasks 6-9 (Full Integration)
- [ ] `./target/release/termcast` shows 4-line output
- [ ] `--location` flag works
- [ ] `--help` works
- [ ] Errors are styled

### Checkpoint: Final (Tasks 10-11)
- [ ] `cargo test` passes 100%
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` passes
- [ ] All acceptance criteria from SPEC.md met

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Open-Meteo API changes response format | Medium | Use `serde(default)` and handle missing fields gracefully |
| ipapi.co rate limiting | Low | Only called once per run; fallback to manual input |
| Unicode rendering issues in terminals | Low | Test in iTerm2 and macOS Terminal; use standard Unicode |
| Cross-compilation for Linux | Low | Use Docker or GitHub Actions for Linux builds |

---

## Open Questions

- **Binary distribution:** How will users install? (Homebrew, direct download, cargo install?)
  - *Status:* Not in MVP scope — just `cargo build --release`
- **Temperature unit detection:** Auto-detect from locale — should we use system locale or default to Celsius?
  - *Decision:* Default to Celsius, add `--fahrenheit` flag for explicit override (MVP keeps it simple)
- **Future features:** Extended forecast, multiple locations, caching
  - *Status:* Out of scope for MVP — focused on 4-line output

---

## Implementation Order

1. Project Foundation (Tasks 1-3)
2. API Integration (Tasks 4-5)
3. CLI and Rendering (Tasks 6-9)
4. Testing and Polish (Tasks 10-11)
