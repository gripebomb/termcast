# Implementation Plan: TermCast

## Overview

Build a weather CLI tool in Rust with beautiful terminal output. All phases are complete.

## Architecture Decisions

1. **Async runtime via `tokio`** — Required for `reqwest` async features, lightweight for CLI
2. **`thiserror` for errors** — Provides structured error types with `?` operator support
3. **Crossterm for rendering** — Cross-platform terminal capabilities with ANSI support
4. **Minimal dependencies** — Only what's specified in SPEC.md to keep binary small
5. **Semantic color slots** — Theme system uses 6 named color roles, never hardcoded in renderer
6. **Static built-in themes** — No file I/O for themes, all compiled into the binary
7. **ANSI 256-color fallback** — Automatic color cube mapping for non-truecolor terminals

## Task List

### Phase 1: Project Foundation

- [x] **Task 1: Initialize Rust project structure**
  - Created `Cargo.toml` with all dependencies
  - Set Rust edition to 2021
  - Created `src/` directory structure

- [x] **Task 2: Implement error types**
  - Created `src/errors.rs` with `AppError` enum
  - Variants: `NetworkError`, `ParseError`, `GeolocationError`, `WeatherError`, `InvalidArg`
  - Implemented `std::error::Error` and `Display` traits via thiserror

- [x] **Task 3: Implement data types for API responses**
  - Created `src/geolocation.rs` — `GeoResponse` struct
  - Created `src/weather.rs` — `WeatherResponse` struct with WMO code mappings
  - Used `#[serde(rename = "...")]` for camelCase JSON fields

### Phase 2: API Integration

- [x] **Task 4: Implement HTTP client for geolocation**
  - Fetch from `https://ipapi.co/json/`
  - Parse response into `GeoResponse`
  - Includes User-Agent header and timeout

- [x] **Task 5: Implement HTTP client for weather**
  - Fetch from Open-Meteo API
  - Returns current temp, feels-like, high, low, weather code, daily summary
  - Handles API error responses and malformed JSON

### Phase 3: CLI and Rendering

- [x] **Task 6: Implement CLI argument parsing**
  - Used `clap` with `derive` macro
  - Arguments: `--location`, `-l`, `--help`, `--version`, `--ambient`, `--cache-ttl`, `--install`, `--at`, `--config`, `--list-themes`, `--preview-theme`
  - Subcommands: `forecast`, `completions`

- [x] **Task 7: Implement terminal renderer**
  - Created `src/renderer.rs`
  - Weather icon mapping based on WMO weather codes
  - Styled output using crossterm with theme colors
  - 80-column centered output

- [x] **Task 8: Wire up main.rs**
  - Parse CLI arguments, get location, fetch weather, render output
  - Handle errors with styled error messages

- [x] **Task 9: Add weather code to text description**
  - `description()` method mapping WMO codes to readable strings

### Phase 4: Ambient Mode and Caching

- [x] **Task 10: Implement ambient mode**
  - Compact output format for shell prompts: `☀️ 14°C`
  - `--ambient` flag with cache-first strategy
  - `--install` for shell integration snippets (bash, zsh, tmux)

- [x] **Task 11: Implement weather caching**
  - Disk cache at `~/.cache/termcast/current`
  - Configurable TTL (default 15 minutes)
  - Regular mode populates cache, ambient mode reads first

### Phase 5: Configuration

- [x] **Task 12: Implement TOML config loading**
  - `src/config.rs` with `Defaults` and `Location` structs
  - XDG config directory support
  - Named locations with optional coordinates
  - Unit preference (auto/celsius/fahrenheit)
  - `--at` flag for saved location resolution

- [x] **Task 13: Add shell completions**
  - `completions` subcommand using `clap_complete`
  - Supports bash, zsh, fish, elvish, powershell

### Phase 6: Forecast

- [x] **Task 14: Implement multi-day forecast**
  - `src/forecast.rs` with daily and hourly data types
  - `forecast` subcommand with `--days` (1-7) and `--hourly` flags
  - `--ambient` for compact one-line forecast output

### Phase 7: Color Themes

- [x] **Task 15: Implement theme engine**
  - `src/theme.rs` with `ThemeColors` struct (6 semantic slots)
  - 11 built-in themes: default, catppuccin, catppuccin-latte, dracula, nord, solarized, solarized-light, tokyo-night, tokyo-night-light, gruvbox, gruvbox-light
  - Case-insensitive name resolution with hyphen/underscore normalization
  - Theme aliases (e.g., `catppuccin-mocha` → `catppuccin`)

- [x] **Task 16: Add theme config and CLI flags**
  - `theme` field in config `[defaults]`
  - `--list-themes` flag to show available themes
  - `--preview-theme <name>` to render demo with chosen theme
  - Warning on unknown theme names

- [x] **Task 17: Implement ANSI 256-color fallback**
  - `rgb_to_ansi256()` function mapping RGB to 6x6x6 color cube
  - Greyscale ramp handling (232-255)
  - `supports_truecolor()` detecting `COLORTERM` env var
  - Automatic color adaptation in `main.rs`

### Phase 8: Testing

- [x] **Task 18: Write unit tests**
  - Theme resolution tests (case, aliases, fallbacks)
  - ANSI 256-color conversion tests
  - Config parsing tests (TOML, defaults, XDG paths)
  - CLI argument parsing tests
  - Renderer output tests
  - Forecast data construction tests

- [x] **Task 19: Code quality checks**
  - `cargo fmt --check` passes
  - `cargo clippy` passes with no warnings

## Checkpoints

### Checkpoint: Foundation (Phase 1)
- [x] Project structure exists
- [x] `cargo check` compiles
- [x] Error types are defined
- [x] Data types parse correctly

### Checkpoint: API Layer (Phase 2)
- [x] Geolocation works
- [x] Weather API integration complete
- [x] Unit tests for parsing pass

### Checkpoint: Full Integration (Phase 3)
- [x] `./target/release/termcast` shows 4-line output
- [x] `--location` flag works
- [x] `--help` works
- [x] Errors are styled

### Checkpoint: Ambient Mode (Phase 4)
- [x] `--ambient` outputs compact format
- [x] Cache reads and writes correctly
- [x] Shell integration snippets work

### Checkpoint: Config (Phase 5)
- [x] TOML config loads from XDG path
- [x] Named locations resolve correctly
- [x] Shell completions generate

### Checkpoint: Forecast (Phase 6)
- [x] Multi-day forecast renders
- [x] Hourly breakdown works
- [x] Ambient forecast mode works

### Checkpoint: Themes (Phase 7)
- [x] `--list-themes` shows all 11 themes
- [x] `--preview-theme` renders demo
- [x] Config `theme` field works
- [x] ANSI 256-color fallback works

### Checkpoint: Final (Phase 8)
- [x] `cargo test` passes 100%
- [x] `cargo fmt --check` passes
- [x] `cargo clippy` passes
- [x] All acceptance criteria from SPEC.md met

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Open-Meteo API changes response format | Medium | Use `serde(default)` and handle missing fields gracefully |
| ipapi.co rate limiting | Low | Only called once per run; fallback to manual input |
| Unicode rendering issues in terminals | Low | Test in iTerm2 and macOS Terminal; use standard Unicode |
| Cross-compilation for Linux | Low | Use Docker or GitHub Actions for Linux builds |
| Theme colors look wrong on 256-color terminals | Low | ANSI fallback uses color cube mapping for best approximation |

## Open Questions

- **Binary distribution:** How will users install? (Homebrew, direct download, cargo install?)
  - *Status:* Not in MVP scope — just `cargo build --release`
- **User-defined themes:** Allow loading theme files from config directory?
  - *Status:* Deferred — static built-in themes cover the common cases
- **Future features:** Extended forecast beyond 7 days, radar/map output, weather alerts
  - *Status:* Out of scope — focused on core weather display

## Implementation Order

1. ~~Project Foundation (Tasks 1-3)~~
2. ~~API Integration (Tasks 4-5)~~
3. ~~CLI and Rendering (Tasks 6-9)~~
4. ~~Ambient Mode and Caching (Tasks 10-11)~~
5. ~~Configuration (Tasks 12-13)~~
6. ~~Forecast (Task 14)~~
7. ~~Color Themes (Tasks 15-17)~~
8. ~~Testing and Polish (Tasks 18-19)~~
