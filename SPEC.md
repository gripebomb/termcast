# TermCast — Spec

## Objective

Build a weather CLI tool for terminal enthusiasts where **aesthetic quality is the primary value proposition**. The output should be screenshot-worthy, share-worthy, and permanently fixture-worthy in a terminal setup. Supports current weather, multi-day forecasts, ambient mode for shell prompts, configurable color themes, and persistent named locations.

**User story:** "I open my terminal and `termcast` gives me weather that looks as good as my vim config."

**Success criteria:**
- Output renders beautifully in iTerm2, macOS Terminal, kitty, and Alacritty
- No API key required — works on first run with zero config
- Location auto-detected via IP geolocation
- `--location` flag overrides detected location
- Temperature unit auto-detected from locale (US → °F, all others → °C)
- Error output is styled to match aesthetic
- Binary runs on macOS and Linux
- 11 built-in color themes with automatic terminal compatibility
- Multi-day forecast with hourly breakdown
- Ambient mode for shell prompt integration
- TOML config with named locations

## Tech Stack

- **Language:** Rust (2021 edition)
- **Key dependencies:**
  - `crossterm` — cross-platform terminal rendering
  - `reqwest` — HTTP client for API calls
  - `serde` + `serde_json` — JSON parsing
  - `tokio` — async runtime (reqwest feature)
  - `clap` — CLI argument parsing with derive macros
  - `clap_complete` — shell completion generation
  - `thiserror` — structured error types
  - `toml` — config file parsing
  - `chrono` — date/time handling for forecasts
  - `dirs` — XDG directory resolution
  - `urlencoding` — URL encoding for location queries
- **APIs:**
  - Weather: [Open-Meteo](https://open-meteo.com/) (free, no key)
  - Geolocation: [ipapi.co](https://ipapi.co/) (free, no key)
  - Geocoding: Open-Meteo geocoding service (free, no key)

## Commands

```sh
# Build release binary
cargo build --release

# Run
cargo run --release
./target/release/termcast

# Override location
./target/release/termcast --location Oslo
./target/release/termcast -l "San Francisco, CA"

# Use saved location
./target/release/termcast --at home

# Forecast
./target/release/termcast forecast
./target/release/termcast forecast --days 7 --hourly
./target/release/termcast forecast --ambient

# Ambient mode
./target/release/termcast --ambient
./target/release/termcast --ambient --cache-ttl 30

# Color themes
./target/release/termcast --list-themes
./target/release/termcast --preview-theme dracula

# Config
./target/release/termcast --config /path/to/config.toml
./target/release/termcast --list-locations

# Shell integration
./target/release/termcast --install bash
./target/release/termcast --install zsh
./target/release/termcast --install tmux
./target/release/termcast --install

# Shell completions
./target/release/termcast completions bash
./target/release/termcast completions zsh
./target/release/termcast completions fish

# Show help
./target/release/termcast --help

# Run tests
cargo test

# Format / lint
cargo fmt --check
cargo clippy
```

## Project Structure

```
termcast/
├── Cargo.toml
├── src/
│   ├── main.rs          → Entry point, CLI argument parsing
│   ├── lib.rs           → Library module exports
│   ├── api.rs           → Open-Meteo + ipapi.co HTTP calls
│   ├── cache.rs         → Weather data caching for ambient mode
│   ├── config.rs        → TOML config loading and named locations
│   ├── errors.rs        → Structured error types
│   ├── forecast.rs      → Multi-day and hourly forecast data types
│   ├── geolocation.rs   → Data types for IP geolocation response
│   ├── renderer.rs      → Terminal rendering with crossterm
│   ├── theme.rs         → Color theme engine with 11 built-in palettes
│   └── weather.rs       → Weather data types and WMO code mappings
└── docs/
    ├── decisions/       → Architecture Decision Records
    ├── specs/           → Feature specifications
    ├── plans/           → Implementation plans
    └── ideas/           → Feature ideas and notes
```

## Code Style

**Output format (regular — 4 lines):**
```
     ☁ 14°C Oslo
   Feels 11°
   High 17° · Low 8°
   Clear until evening
```

**Key conventions:**
- `camelCase` for JSON fields parsed from APIs
- Error messages are styled with crossterm (red, bold)
- No emoji in code — Unicode symbols only (`☁`, `🌤`, `❄`, etc.)
- API structs use `#[derive(Deserialize)]` with serde
- All public functions have doc comments
- Errors use a custom `AppError` enum with `thiserror`
- Colors defined via semantic slots in theme system, never hardcoded in renderer

## Configuration

### Config File

Location: `$XDG_CONFIG_HOME/termcast/config.toml` (defaults to `~/.config/termcast/config.toml`)

```toml
[defaults]
default_location = "auto"    # "auto" or specific location/city
units = "auto"               # "auto", "celsius", "fahrenheit", "C", "F"
cache_ttl = 15               # Minutes for ambient mode caching
theme = ""                   # Color theme name (empty = default)

[locations.home]
city = "Oslo"
latitude = 59.91
longitude = 10.75

[locations.work]
city = "San Francisco, CA"
# Can omit coordinates - will be geocoded
```

### Configuration Priority

CLI args > Config file > Defaults > Auto-detection

## Color Theme System

### Design

The theme system uses 6 semantic color slots that decouple color choices from rendering logic:

| Slot | Visual Role |
|------|-------------|
| `text` | Main display text, conditions |
| `dimmed` | "Feels like", low precipitation |
| `temp_high` | High temperatures, "Today" |
| `temp_low` | Low temperatures, "Tomorrow" |
| `precip_high` | High precipitation (80%+) |
| `precip_medium` | Medium precipitation (50-79%) |

### Built-in Themes

11 themes included: default, catppuccin (mocha/latte), dracula, nord, solarized (dark/light), tokyo-night (dark/light), gruvbox (dark/light).

### Terminal Compatibility

- True-color (RGB) when `COLORTERM=truecolor|24bit`
- ANSI 256-color fallback via 6x6x6 color cube mapping
- Automatic detection at runtime

## Testing Strategy

- **Unit tests** on parsing logic (mock JSON strings)
- **Unit tests** on theme resolution (case, aliases, fallbacks)
- **Unit tests** on ANSI 256-color conversion
- **Unit tests** on config parsing (TOML, defaults, XDG paths)
- **Integration tests** with mocked HTTP responses using `wiremock`
- No live API calls in tests
- Run `cargo test` before any commit

## Boundaries

**Always:**
- Run `cargo test` before commits
- Format with `cargo fmt`
- Handle API errors gracefully (no panics on network failure)
- Use `thiserror` for structured errors
- Use semantic color slots from theme system (never hardcode colors in renderer)

**Ask first:**
- Adding new dependencies beyond the core ones listed above
- Changing the output format or rendering approach
- Adding new semantic color slots

**Never:**
- Commit API keys or secrets (none expected, but guard against it)
- Call live APIs in tests
- Remove failing tests
- Hardcode colors in the renderer

## Success Criteria

1. [x] `cargo build --release` compiles without warnings
2. [x] `cargo test` passes 100%
3. [x] `./target/release/termcast` outputs 4 lines of styled weather
4. [x] `./target/release/termcast --location Oslo` overrides IP location
5. [x] `./target/release/termcast --help` shows usage info
6. [x] Error on network failure renders styled error (not raw panic)
7. [x] Output contains ANSI color codes when piped through `cat`
8. [x] Binary runs without needing `cargo run` (static binary or linked lib)
9. [x] Output renders correctly in iTerm2 and macOS Terminal
10. [x] `--list-themes` shows all 11 built-in themes
11. [x] `--preview-theme <name>` renders demo output in chosen theme
12. [x] Config `theme` field persists theme choice
13. [x] ANSI 256-color fallback works on non-truecolor terminals
14. [x] `forecast` subcommand renders multi-day forecast
15. [x] `forecast --hourly` shows notable hourly changes
16. [x] Ambient mode outputs compact format for shell prompts
17. [x] Config file supports named locations with optional coordinates
18. [x] Shell completions for bash, zsh, fish, elvish, powershell

## Open Questions

- ~~Output method~~ — stdout + ANSI (resolved)
- ~~Location flag~~ — include `--location` / `-l` (resolved)
- ~~Error styling~~ — colorized (resolved)
- ~~Temperature units~~ — locale detection implemented: US → °F, others → °C (resolved)
- ~~Rust edition~~ — 2021 (resolved)
- ~~Ambient mode~~ — implemented (see docs/SPEC-ambient.md)
- ~~Color themes~~ — implemented with 11 built-in palettes and ANSI fallback
- ~~Config file~~ — TOML with named locations, units, and theme
- ~~Multi-day forecast~~ — implemented with `forecast` subcommand
