# AGENTS.md

This file provides guidance to AI coding agents (Claude Code, Cursor, Copilot, OpenCode, etc.) when working with code in this repository.

## Project Overview

TermCast is a Rust CLI tool that renders beautiful weather output in the terminal. It uses Open-Meteo (free, no API key) for weather data and ipapi.co for IP-based geolocation.

## Quick Reference

```bash
cargo build --release     # Build
cargo test                # Run tests
cargo fmt --check         # Check formatting
cargo clippy              # Lint
./target/release/termcast # Run (auto-detects location)
```

## Project Structure

```
src/
├── main.rs          # CLI entry point (clap derive), orchestration
├── lib.rs           # Module declarations and re-exports
├── api.rs           # HTTP client (reqwest) for weather/geolocation APIs
├── cache.rs         # Disk cache for ambient mode (~/.cache/termcast/)
├── config.rs        # TOML config loading, XDG paths, named locations
├── errors.rs        # AppError enum with thiserror
├── forecast.rs      # Multi-day and hourly forecast data types
├── geolocation.rs   # IP geolocation response types (ipapi.co)
├── renderer.rs      # Terminal rendering with crossterm
├── theme.rs         # Color theme engine (11 built-in palettes)
└── weather.rs       # Weather data types, WMO code mappings
```

## Architecture

- **Async:** tokio runtime, reqwest for HTTP
- **CLI:** clap with derive macros; subcommands: `forecast`, `completions`
- **Rendering:** crossterm for terminal styling; 80-column centered output
- **Colors:** Semantic color slots via theme system (never hardcode colors in renderer)
- **Config:** TOML at `$XDG_CONFIG_HOME/termcast/config.toml`
- **Cache:** JSON at `$XDG_CACHE_HOME/termcast/current`

## Key Conventions

- **Error handling:** Use `AppError` enum with `thiserror`. Never panic on expected failures (network, API errors).
- **Color usage:** Always use `ThemeColors` from the theme system. The renderer accepts `&ThemeColors` — never pass raw `Color::Rgb` or `Color::AnsiValue`.
- **API structs:** Use `#[derive(Deserialize)]` with `#[serde(rename = "camelCase")]` for JSON fields.
- **Unicode only:** Weather icons are Unicode symbols, not emoji. Use `☀️`, `🌤`, `🌧`, etc.
- **No live API calls in tests.** Use `wiremock` for mocked HTTP responses.
- **Run `cargo test` before committing.**

## Configuration Priority

CLI args > Config file > Defaults > Auto-detection (IP geolocation, locale)

## Adding a New Theme

Edit `src/theme.rs` — add a `Theme` entry to the `BUILTIN_THEMES` array with all 6 color slots (`text`, `dimmed`, `temp_high`, `temp_low`, `precip_high`, `precip_medium`). Optionally add aliases. Add a test for the new theme in the test module.

## Adding a New CLI Flag

1. Add the field to `Args` struct in `src/main.rs` with `#[arg(...)]` attribute
2. Handle it in the `run()` function
3. Add parsing tests in the `#[cfg(test)]` module
4. Update README.md commands section

## Documentation

- `SPEC.md` — Full project specification and success criteria
- `PLAN.md` — Implementation plan with task list
- `docs/decisions/` — Architecture Decision Records
- `docs/specs/` — Feature-specific specifications
- `docs/plans/` — Feature implementation plans
