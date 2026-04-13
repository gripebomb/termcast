# TermCast

A beautiful weather CLI tool for terminal enthusiasts. Get current weather conditions rendered with aesthetic terminal output.

## Quick Start

```bash
# Build
cargo build --release

# Run (auto-detects location via IP)
./target/release/termcast

# Override location
./target/release/termcast --location Oslo
```

## Features

- **Beautiful output** — ANSI-colored weather display with icons
- **Zero config** — Works on first run with auto-detected location
- **Ambient mode** — Compact output for shell prompts and tmux status bars
- **Persistent caching** — Weather cached to disk for fast shell integration

## Commands

```bash
# Regular weather display
./target/release/termcast
./target/release/termcast --location "San Francisco, CA"

# Ambient mode (for shell prompts)
./target/release/termcast --ambient
./target/release/termcast --ambient --cache-ttl 30 --location Oslo

# Shell integration
./target/release/termcast --install bash    # Bash integration
./target/release/termcast --install zsh     # Zsh integration
./target/release/termcast --install tmux    # tmux integration
./target/release/termcast --install         # All integrations

# Help
./target/release/termcast --help
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

## Output Format

**Regular mode:**
```
     ☀️ 14°C Oslo
   Feels 11°
   High 17° · Low 8°
   Clear
```

**Ambient mode:**
```
☀️ 14°
```

## Cache

Weather data is cached to `~/.cache/termcast/current` (or `$XDG_CACHE_HOME/termcast/current`).

- Default TTL: 15 minutes
- Custom TTL: `--cache-ttl <minutes>`
- Regular mode (`termcast`) always populates the cache
- Ambient mode (`termcast --ambient`) reads from cache first

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
- **APIs:** Open-Meteo (weather), ipapi.co (geolocation)
- **No API keys required**

## Project Structure

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library exports
├── api.rs           # HTTP client for APIs
├── cache.rs         # Weather data caching
├── errors.rs        # Error types
├── geolocation.rs   # IP geolocation types
├── renderer.rs      # Terminal output
└── weather.rs       # Weather data types
```

## License

MIT
