# Implementation Plan: TermCast Forecast

## Overview

Add a `termcast forecast` subcommand that provides multi-day (1-7 day) and hourly weather forecasts. This is built on top of the existing Open-Meteo `/v1/forecast` endpoint, reusing the current location resolution, config, and rendering patterns.

## Architecture Decisions

1. **New `forecast` module** rather than extending `weather.rs` — the forecast types are distinct from current-weather types, and the spec says not to change `WeatherResponse` or `DailyWeather`.
2. **Clap subcommand** — current CLI uses `#[command(subcommand)]` for `CompletionsCmd`. We add a `Forecast` variant alongside `Completions`. The existing top-level behavior (no subcommand) remains unchanged.
3. **New `get_forecast()` method on `api::Client`** — separate from `get_weather()` since it uses different query parameters (hourly params, `forecast_days`, `precipitation_probability_max`).
4. **Hourly filter logic lives in `forecast.rs`** — it's pure data transformation, no I/O dependencies.
5. **Chrono for date/time parsing** — needed to parse `"2024-01-15T00:00"` hourly timestamps and compute day-of-week names. The alternative (`time` crate or manual parsing) adds more complexity. This is a new dependency; flagged in boundaries.

## Dependency Graph

```
forecast.rs (types + filter logic)
    │
    ├── api.rs (get_forecast method)
    │       │
    │       └── main.rs (subcommand dispatch)
    │
    └── renderer.rs (forecast render functions)
            │
            └── main.rs (render + output)
```

## Task List

### Phase 1: Foundation — Types and Parsing

#### Task 1: Add chrono dependency and create forecast types module

**Description:** Create `src/forecast.rs` with all the data types for forecast API responses and display-ready structs. Add `chrono` to `Cargo.toml`. Export the module from `lib.rs`.

**Acceptance criteria:**
- [ ] `ForecastResponse`, `DailyForecast`, `HourlyForecast` structs deserialize from Open-Meteo JSON
- [ ] `DailyRow`, `HourlyEntry`, `ForecastDisplay` display-ready structs are defined
- [ ] `DailyForecast` has a `to_daily_rows()` method that converts raw daily data to `Vec<DailyRow>` with day names ("Today", "Tomorrow", "Mon", "Tue", etc.)
- [ ] `HourlyForecast` has a `filter_notable()` method that implements the notable-change filter (temp delta > 5, precip delta > 20%, always include first/last, merge < 2h apart)
- [ ] Unit tests for JSON deserialization with inline fixture
- [ ] Unit tests for `to_daily_rows()` with day name computation
- [ ] Unit tests for `filter_notable()` covering: no notable changes, all notable, boundary conditions, merge logic
- [ ] `cargo test` passes

**Verification:**
- [ ] Tests pass: `cargo test -- forecast`
- [ ] Build succeeds: `cargo build`

**Dependencies:** None

**Files likely touched:**
- `Cargo.toml` (add chrono)
- `src/forecast.rs` (new file)
- `src/lib.rs` (add `pub mod forecast`)

**Estimated scope:** Small (1-2 files + new file)

---

### Phase 2: API Client

#### Task 2: Add get_forecast method to api::Client

**Description:** Add a `get_forecast()` method to `api::Client` that constructs the Open-Meteo forecast URL with daily and hourly parameters, fetches the data, and returns a `ForecastDisplay`.

**Acceptance criteria:**
- [ ] `get_forecast(latitude, longitude, location, use_fahrenheit, days, include_hourly)` method exists on `Client`
- [ ] URL includes: `daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max`
- [ ] URL includes hourly params only when `include_hourly` is true: `hourly=temperature_2m,precipitation_probability,weather_code`
- [ ] URL includes `forecast_days={days}`
- [ ] Response is parsed into `ForecastResponse`, then converted to `ForecastDisplay`
- [ ] Error handling follows existing pattern: `AppError::network()`, `AppError::parse()`
- [ ] Tests pass with wiremock mock server (integration test)

**Verification:**
- [ ] Tests pass: `cargo test -- api::tests`
- [ ] Build succeeds: `cargo build`

**Dependencies:** Task 1

**Files likely touched:**
- `src/api.rs` (add `get_forecast` method, add import for forecast types)

**Estimated scope:** Small (1 file)

---

### Phase 3: CLI Subcommand

#### Task 3: Add forecast subcommand to CLI

**Description:** Restructure the clap CLI to support a `forecast` subcommand with `--days`, `--hourly`, and `--ambient` flags. The `--location`, `--at`, and `--config` flags are shared at the top level. The completions subcommand continues to work.

**Acceptance criteria:**
- [ ] `termcast forecast` parses correctly with default values (days=5, no hourly, no ambient)
- [ ] `termcast forecast --days 3` parses with days=3
- [ ] `termcast forecast --hourly` parses with hourly=true
- [ ] `termcast forecast --ambient` parses with ambient=true
- [ ] `termcast forecast --days 7 --hourly -l Oslo` parses correctly
- [ ] `--days` validates range 1-7 (clap validation)
- [ ] `termcast completions bash` still works (subcommand not broken)
- [ ] `termcast` (no subcommand) still works exactly as before
- [ ] Top-level flags (`--location`, `--at`, `--config`) are accessible from forecast subcommand
- [ ] Unit tests for all argument parsing combinations

**Verification:**
- [ ] Tests pass: `cargo test -- main::tests`
- [ ] Build succeeds: `cargo build`

**Dependencies:** Task 1

**Files likely touched:**
- `src/main.rs` (restructure CLI, add `Forecast` subcommand, add dispatch logic)

**Estimated scope:** Medium (1 file, significant restructuring)

---

### Checkpoint: Foundation Complete
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] `termcast` (no subcommand) works unchanged
- [ ] `termcast forecast` parses but may not render yet

---

### Phase 4: Rendering

#### Task 4: Add forecast rendering functions

**Description:** Add `render_forecast()` and `render_forecast_hourly()` functions to `renderer.rs` following the existing crossterm queue-based pattern. Add `output_ambient_forecast()` for the one-line format.

**Acceptance criteria:**
- [ ] `render_forecast(display: &ForecastDisplay)` renders the daily table with centered title, day names, icons, high/low temps, precip chance
- [ ] Day name styling: bold white for regular, cyan for "Today", magenta for "Tomorrow"
- [ ] High temp in cyan, low temp in magenta
- [ ] Precip chance: dim if < 30%, yellow if >= 50%, red if >= 80%, with umbrella prefix
- [ ] `render_forecast_hourly(display: &ForecastDisplay)` renders hourly entries with time, icon, temp, precip, annotations
- [ ] Annotations shown in dim grey for change descriptions ("Rain starting", "Rain clearing")
- [ ] `output_ambient_forecast(display: &ForecastDisplay)` outputs single-line format: `[icon] [high]/[low] ☂ [precip]% [day_name]`
- [ ] Tests verify rendering functions don't panic (same pattern as existing renderer tests)

**Verification:**
- [ ] Tests pass: `cargo test -- renderer::tests`
- [ ] Build succeeds: `cargo build`

**Dependencies:** Task 1

**Files likely touched:**
- `src/renderer.rs` (add 3 new render functions)

**Estimated scope:** Medium (1 file, ~100-150 lines of rendering code)

---

### Phase 5: Integration

#### Task 5: Wire forecast subcommand to API and renderer

**Description:** Connect the forecast subcommand dispatch in `main.rs` to the API client and rendering functions. Handle location resolution (reuse existing logic), unit preference, and error cases.

**Acceptance criteria:**
- [ ] `termcast forecast` resolves location (auto, --location, --at), fetches forecast, renders daily table
- [ ] `termcast forecast --hourly` also shows hourly breakdown
- [ ] `termcast forecast --ambient` outputs single-line format
- [ ] `termcast forecast --days N` passes N to API (validated 1-7)
- [ ] Location resolution shares code with existing path (uses `resolve_full_location`)
- [ ] Error messages are descriptive and follow existing patterns
- [ ] `cargo test` and `cargo clippy` pass

**Verification:**
- [ ] Tests pass: `cargo test`
- [ ] Lint passes: `cargo clippy`
- [ ] Manual test: `cargo run -- forecast`
- [ ] Manual test: `cargo run -- forecast --hourly`
- [ ] Manual test: `cargo run -- forecast --ambient`
- [ ] Manual test: `cargo run -- forecast --days 3 -l Oslo`

**Dependencies:** Task 2, Task 3, Task 4

**Files likely touched:**
- `src/main.rs` (wire forecast dispatch, add `run_forecast` function)

**Estimated scope:** Small-Medium (1 file)

---

### Checkpoint: Core Feature Complete
- [ ] All success criteria from spec are met except shell completions
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] Existing `termcast` behavior unchanged

---

### Phase 6: Polish

#### Task 6: Update shell completions and final verification

**Description:** Verify shell completions include the new `forecast` subcommand and its flags. Run full test suite and clippy. Final manual verification of all CLI combinations.

**Acceptance criteria:**
- [ ] `termcast completions bash` output includes `forecast` subcommand
- [ ] `termcast completions bash` output includes `--days`, `--hourly`, `--ambient` flags
- [ ] `cargo test` passes with zero failures
- [ ] `cargo clippy` passes with zero warnings
- [ ] All success criteria from the spec are verified

**Verification:**
- [ ] `cargo test`
- [ ] `cargo clippy`
- [ ] `cargo run -- completions bash | grep -A5 forecast`

**Dependencies:** Task 5

**Files likely touched:**
- None expected (clap auto-generates completions from derive)

**Estimated scope:** XS (verification only)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Clap subcommand restructuring breaks existing `termcast` behavior | High | Test existing behavior explicitly in Task 3; keep top-level args unchanged |
| Open-Meteo hourly data format differs from assumptions | Medium | Verify with live API early (Task 2 integration test with wiremock based on real response) |
| Chrono adds significant compile time | Low | chrono is widely used; impact is minimal. Could use `time` crate as alternative |
| Hourly filter edge cases (empty data, single entry) | Medium | Explicit test cases in Task 1 for boundary conditions |

## Open Questions

- None (all resolved in spec)
