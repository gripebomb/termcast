# TermCast — Spec

## Objective

Build a weather CLI tool for terminal enthusiasts where **aesthetic quality is the primary value proposition**. The output should be screenshot-worthy, share-worthy, and permanently fixture-worthy in a terminal setup. MVP delivers the essential 4 facts rendered beautifully: temperature, high/low, condition icon, one-line forecast.

**User story:** "I open my terminal and `termcast` gives me weather that looks as good as my vim config."

**Success criteria:**
- Output renders beautifully in iTerm2, macOS Terminal, and kitty
- No API key required — works on first run with zero config
- Location auto-detected via IP geolocation
- `--location` flag overrides detected location
- Temperature unit auto-detected (°C / °F)
- Error output is styled to match aesthetic
- Binary runs on macOS and Linux

## Tech Stack

- **Language:** Rust (2021 edition)
- **Key dependencies:**
  - `crossterm` — cross-platform terminal rendering
  - `reqwest` — HTTP client for API calls
  - `serde` + `serde_json` — JSON parsing
  - `tokio` — async runtime (reqwest feature)
- **APIs:**
  - Weather: [Open-Meteo](https://open-meteo.com/) (free, no key)
  - Geolocation: [ipapi.co](https://ipapi.co/) (free, no key)

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
│   ├── api.rs           → Open-Meteo + ipapi.co HTTP calls
│   ├── weather.rs       → Data types for weather response
│   ├── geolocation.rs  → Data types for IP geolocation response
│   ├── renderer.rs      → Terminal rendering with crossterm
│   └── errors.rs        → Structured error types
├── tests/
│   └── api_tests.rs     → Integration tests with mocked HTTP
└── SPEC.md
```

## Code Style

**Output format (MVP — 4 lines):**
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

**Example weather icon mapping:**
```rust
match code {
    0  => "☀️",  // Clear
    1..=3 => "🌤", // Partly cloudy
    45..=48 => "🌫", // Fog
    51..=67 => "🌧", // Rain/drizzle
    71..=77 => "❄",  // Snow
    80..=82 => "🌦", // Showers
    95..=99 => "⛈",  // Thunderstorm
    _ => "☁",
}
```

## Testing Strategy

- **Unit tests** on parsing logic (mock JSON strings)
- **Integration tests** with mocked HTTP responses using `wiremock` or `reqwest` mock
- No live API calls in tests
- Run `cargo test` before any commit

## Boundaries

**Always:**
- Run `cargo test` before commits
- Format with `cargo fmt`
- Handle API errors gracefully (no panics on network failure)
- Use `thiserror` for structured errors

**Ask first:**
- Adding new dependencies beyond the core ones listed above
- Changing the output format or rendering approach

**Never:**
- Commit API keys or secrets (none expected, but guard against it)
- Call live APIs in tests
- Remove failing tests

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

## Ambient Mode (Extended Feature)

See [docs/SPEC-ambient.md](docs/SPEC-ambient.md) for the full ambient mode specification.

**Quick reference:**

```bash
# Ambient mode — compact output for shell prompts
./target/release/termcast --ambient          # Output: "☀️ 14°"

# Cache TTL in minutes (default: 15)
./target/release/termcast --cache-ttl 30

# Shell integration snippets
./target/release/termcast --install bash
./target/release/termcast --install zsh
./target/release/termcast --install tmux
```

## Open Questions

- ~~Output method~~ — stdout + ANSI (resolved)
- ~~Location flag~~ — include `--location` / `-l` (resolved)
- ~~Error styling~~ — colorized (resolved)
- ~~Temperature units~~ — auto-detect from locale (resolved)
- ~~Rust edition~~ — 2021 (resolved)
- ~~Ambient mode~~ — implemented (see docs/SPEC-ambient.md)
