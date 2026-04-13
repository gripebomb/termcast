# TermCast

A beautiful weather CLI tool for terminal enthusiasts. Get current weather and multi-day forecasts rendered with aesthetic terminal output and customizable color themes.

## Quick Start

```bash
# Build
cargo build --release

# Run (auto-detects location via IP)
./target/release/termcast

# Override location
./target/release/termcast --location Oslo

# Multi-day forecast
./target/release/termcast forecast --days 7 --hourly
```

## Features

- **Beautiful output** — ANSI-colored weather display with icons, centered in 80 columns
- **Zero config** — Works on first run with auto-detected location
- **Color themes** — 11 built-in themes (catppuccin, dracula, nord, tokyo-night, and more)
- **Multi-day forecast** — Up to 7-day forecast with optional hourly breakdown
- **Ambient mode** — Compact output for shell prompts and tmux status bars
- **Config file** — TOML config with named locations, units, and theme preference
- **Shell completions** — bash, zsh, fish, elvish, powershell
- **No API keys required** — Uses free Open-Meteo and ipapi.co APIs

## Commands

```bash
# Current weather
./target/release/termcast
./target/release/termcast --location "San Francisco, CA"
./target/release/termcast --at home              # Use saved location

# Forecast
./target/release/termcast forecast               # 5-day forecast
./target/release/termcast forecast --days 7       # 7-day forecast
./target/release/termcast forecast --hourly       # With hourly breakdown
./target/release/termcast forecast --ambient      # Compact one-line

# Ambient mode (for shell prompts)
./target/release/termcast --ambient
./target/release/termcast --ambient --cache-ttl 30 --location Oslo

# Color themes
./target/release/termcast --list-themes           # List all themes
./target/release/termcast --preview-theme dracula # Preview a theme

# Shell integration
./target/release/termcast --install bash          # Bash integration
./target/release/termcast --install zsh           # Zsh integration
./target/release/termcast --install tmux          # tmux integration
./target/release/termcast --install               # All integrations

# Shell completions
./target/release/termcast completions bash
./target/release/termcast completions zsh

# Help
./target/release/termcast --help
```

## Color Themes

TermCast includes 11 built-in color themes. All themes define 6 semantic color slots that map to specific visual roles in the weather output.

### Built-in Themes

| Theme | Description |
|-------|-------------|
| `default` | White, cyan, and magenta (original colors) |
| `catppuccin` | Catppuccin Mocha — warm dark pastels |
| `catppuccin-latte` | Catppuccin Latte — warm light pastels |
| `dracula` | Dracula — dark with vivid accents |
| `nord` | Nord — arctic blue-gray palette |
| `solarized` | Solarized Dark — warm earth tones |
| `solarized-light` | Solarized Light — warm cream with accents |
| `tokyo-night` | Tokyo Night — deep blue city nights |
| `tokyo-night-light` | Tokyo Night Light — cool day variant |
| `gruvbox` | Gruvbox — warm retro earth tones |
| `gruvbox-light` | Gruvbox Light — warm light retro tones |

Theme names are case-insensitive. `Catppuccin`, `catppuccin`, and `CATPPUCCIN` all work. Hyphens and underscores are equivalent (`catppuccin_mocha` = `catppuccin-mocha`).

### Setting a Theme

Via CLI (not persistent):
```bash
# Preview before committing
./target/release/termcast --preview-theme nord
```

Via config file (persistent):
```toml
[defaults]
theme = "dracula"
```

### Terminal Compatibility

TermCast detects your terminal's color support automatically:
- **True-color terminals** (iTerm2, kitty, Alacritty, Windows Terminal) — full RGB colors
- **256-color terminals** — automatic ANSI 256-color fallback via the 6x6x6 color cube

Detection uses the `COLORTERM` environment variable (`truecolor` or `24bit`).

## Configuration

Config file location: `~/.config/termcast/config.toml` (or `$XDG_CONFIG_HOME/termcast/config.toml`).

```toml
[defaults]
default_location = "auto"    # "auto" or a city name
units = "auto"               # "auto", "celsius", "fahrenheit"
cache_ttl = 15               # Cache TTL in minutes
theme = ""                   # Color theme name (empty = default)

[locations.home]
city = "Oslo"
latitude = 59.91
longitude = 10.75

[locations.work]
city = "San Francisco, CA"
# Omit coordinates to auto-geocode
```

### Configuration Priority

CLI args > Config file > Defaults > Auto-detection

### Using Saved Locations

```bash
# Resolve against config locations first, then fall back to geocoding
./target/release/termcast --at home
./target/release/termcast --at work
```

## Output Format

**Regular mode:**
```
     ☀️ 14°C Oslo
   Feels 11°
   High 17° · Low 8°
   Clear
```

**Forecast:**
```
        Forecast for Oslo

  Today     ☀️  17°C/8°C    ☂ 5%
  Tomorrow  🌤 15°C/7°C    ☂ 60%
  Wed       🌧 12°C/5°C    ☂ 85%
```

**Ambient mode:**
```
☀️ 14°C
```

## Shell Integration

### Bash

Add to `~/.bashrc` or `~/.bash_profile`:

```bash
termcast_prompt() {
    termcast --ambient
}
export PS1='\u@\h \w $(termcast_prompt)$ '
```

### Zsh

Add to `~/.zshrc`:

```zsh
termcast_prompt() {
    termcast --ambient
}
PROMPT='%n@%m %~ $(termcast_prompt)% '
```

### tmux

Add to `~/.tmux.conf`:

```tmux
set -g status-right '#(termcast --ambient)'
```

## Cache

Weather data is cached to `~/.cache/termcast/current` (or `$XDG_CACHE_HOME/termcast/current`).

- Default TTL: 15 minutes
- Custom TTL: `--cache-ttl <minutes>`
- Regular mode always populates the cache
- Ambient mode reads from cache first, fetches if stale

## Development

```bash
# Run tests
cargo test

# Build release
cargo build --release

# Format
cargo fmt --check
cargo fmt

# Lint
cargo clippy
```

## Tech Stack

- **Language:** Rust 2021
- **APIs:** Open-Meteo (weather, geocoding), ipapi.co (geolocation)
- **No API keys required**

## Project Structure

```
src/
├── main.rs          # CLI entry point with clap argument parsing
├── lib.rs           # Library exports
├── api.rs           # HTTP client for weather/geolocation APIs
├── cache.rs         # Weather data caching for ambient mode
├── config.rs        # TOML config loading and named locations
├── errors.rs        # Structured error types with thiserror
├── forecast.rs      # Multi-day and hourly forecast data
├── geolocation.rs   # IP geolocation response types
├── renderer.rs      # Terminal output rendering with crossterm
├── theme.rs         # Color theme engine with 11 built-in palettes
└── weather.rs       # Weather data types and WMO code mappings
```

## Architecture Decisions

See [docs/decisions/](docs/decisions/) for Architecture Decision Records.

## License

MIT
