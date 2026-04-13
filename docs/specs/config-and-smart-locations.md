# Spec: Config File & Smart Locations

## Objective

TermCast currently works zero-config: auto-detect location via IP, auto-detect temperature units. This feature adds an optional TOML config file that lets users persist preferences (default location, units, cache TTL) and save named locations for quick access via `--at <name>`. Shell completions round out the feature set by making saved location names tab-completable.

**User stories:**
- As a user in a non-US country, I want to force Celsius/Fahrenheit regardless of my IP location
- As a user who checks weather in multiple cities, I want to save `home`, `mom`, `office` and run `termcast --at mom`
- As a user, I want `--at Chicago` to work even without a saved entry (fallback to geocoding)
- As a terminal user, I want tab completion for `--at` to show my saved location names
- As a zero-config user, I want TermCast to work exactly as it does today with no config file

**What success looks like:** `termcast` with no config file produces identical output to today. `termcast --at home` with a config file resolves instantly. Shell completions suggest saved names.

## Tech Stack

- **Language:** Rust 2021 edition
- **CLI:** clap 4.5 (derive mode)
- **Config format:** TOML (via `toml` crate, serde-compatible)
- **Completions:** `clap_complete` 4.5
- **Existing deps used:** `dirs` 5.0 (XDG paths), `serde` 1.0, `thiserror` 2.0

## Commands

```
Build:          cargo build
Run:            cargo run -- [flags]
Test:           cargo test
Test (single):  cargo test test_name
Lint:           cargo clippy -- -D warnings
Format:         cargo fmt
Format check:   cargo fmt -- --check
```

## Project Structure

```
src/
  main.rs        -> CLI entry point, argument parsing, run()
  lib.rs         -> Module declarations
  config.rs      -> NEW -- Config file loading, parsing, XDG path resolution
  api.rs         -> HTTP client (unchanged)
  cache.rs       -> Cache read/write (unchanged)
  errors.rs      -> AppError enum (new variants added)
  geolocation.rs -> GeoResponse type (unchanged)
  renderer.rs    -> Terminal rendering (unchanged)
  weather.rs     -> Weather data types (unchanged)
docs/
  specs/         -> Specification documents (this file)
  ideas/         -> Idea refinement documents
  plans/         -> Implementation plans
```

## Config File Format

### Iteration 1: Defaults

```toml
[defaults]
default_location = "auto"     # "auto" (IP geolocation) or a city name
units = "auto"                # "auto", "celsius", or "fahrenheit"
cache_ttl = 15                # minutes
```

### Iteration 2: Named Locations

```toml
[defaults]
default_location = "home"
units = "auto"
cache_ttl = 15

[locations.home]
city = "Oslo"

[locations.mom]
city = "Chicago"
latitude = 41.88
longitude = -87.63

[locations.office]
city = "San Francisco, CA"
```

### Config resolution rules

1. CLI flags always win (`--location Oslo` overrides everything)
2. Config file values override built-in defaults
3. Missing config file = current behavior (no warning, no prompt)
4. Invalid config fields = log warning to stderr, use built-in default for that field
5. `default_location = "auto"` = IP geolocation (current behavior)
6. `default_location = "home"` = resolve from `[locations.home]`
7. `default_location = "Oslo"` = geocode at runtime (if no matching location entry)
8. `units = "auto"` = current IP-based detection (US -> Fahrenheit, else Celsius)

## CLI Changes

### Iteration 1

```
termcast                    # Uses config defaults or IP auto-detect
termcast -l Oslo            # CLI overrides config
termcast --config ~/my.toml # Use custom config file
```

### Iteration 2

```
termcast --at home          # Resolve from [locations.home]
termcast --at mom           # Resolve from [locations.mom]
termcast --at Chicago       # No saved entry -> geocode "Chicago"
termcast --at home --ambient # Works in ambient mode too
```

### Iteration 3

```
termcast completions bash   # Print bash completion script
termcast completions zsh    # Print zsh completion script
termcast completions fish   # Print fish completion script
```

### Precedence

`--location` > `--at` > config `default_location` > IP auto-detect

## Code Style

Follows existing codebase patterns. Example of the new config module:

```rust
use std::path::PathBuf;
use serde::Deserialize;

/// Application configuration loaded from TOML file.
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub locations: std::collections::HashMap<String, Location>,
}

#[derive(Debug, Deserialize)]
pub struct Defaults {
    #[serde(default = "Defaults::default_location")]
    pub default_location: String,
    #[serde(default = "Defaults::default_units")]
    pub units: String,
    #[serde(default = "Defaults::default_cache_ttl")]
    pub cache_ttl: u64,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            default_location: Self::default_location(),
            units: Self::default_units(),
            cache_ttl: Self::default_cache_ttl(),
        }
    }
}

impl Defaults {
    fn default_location() -> String { "auto".to_string() }
    fn default_units() -> String { "auto".to_string() }
    fn default_cache_ttl() -> u64 { 15 }
}
```

**Key conventions:**
- Module-level doc comments (`//!`) on every file
- `#[cfg(test)] mod tests` at bottom of each file
- `AppError` variants with factory methods for new error types
- Serde derive for all parsed types
- Factory methods on `AppError` follow existing `cache_read`, `cache_write` patterns

## Testing Strategy

**Framework:** Rust built-in `#[test]` with `tempfile` for file-based tests.

**Test locations:** Inline `#[cfg(test)] mod tests` at bottom of each module (existing pattern).

**Coverage expectations:**
- Config parsing: valid TOML, missing file, invalid fields, empty file, partial fields
- Config resolution: XDG path, `--config` override, missing `~/.config/termcast/` dir
- Location resolution: named entry found, named entry not found (fallback to geocode), `--at` precedence vs `--location`
- CLI arg parsing: new `--at` flag, `--config` flag, mutual exclusivity with `--location`
- Completions: `completions bash|zsh|fish` subcommand, invalid shell name

**Test levels:**
- Unit tests: config parsing, path resolution, precedence logic
- Integration tests: full config loading with temp files, `Args::parse_from` for CLI changes
- No mocking needed for completions (pure string output)

## Boundaries

- **Always:** Run `cargo test` before committing, run `cargo clippy`, follow existing module structure, validate config fields on load
- **Ask first:** Adding new dependencies, changing CLI arg names, modifying existing public APIs
- **Never:** Auto-create config files on first run, prompt users to create config, break existing `--location` / `--ambient` / `--cache-ttl` behavior without config, commit without tests passing

## Success Criteria

### Iteration 1 -- Config File
- [ ] `termcast` with no config file behaves identically to current version
- [ ] `termcast` with config file uses `default_location`, `units`, `cache_ttl` from config
- [ ] CLI flags (`--location`, `--cache-ttl`) override config values
- [ ] `--config <path>` loads config from custom path
- [ ] Invalid config fields produce a warning on stderr, not a crash
- [ ] Config file missing is silently ignored (no output)
- [ ] All new code has tests; `cargo test` passes; `cargo clippy` clean

### Iteration 2 -- Smart Locations
- [ ] `[locations.<name>]` entries parsed from config
- [ ] `--at <name>` resolves to saved location's coordinates
- [ ] `--at <city>` with no matching entry falls back to geocoding
- [ ] `--at` works in ambient mode
- [ ] `--location` takes precedence over `--at`
- [ ] `default_location = "home"` resolves from `[locations.home]`
- [ ] All new code has tests; `cargo test` passes; `cargo clippy` clean

### Iteration 3 -- Shell Completions
- [ ] `termcast completions bash` prints valid bash completion script
- [ ] `termcast completions zsh` prints valid zsh completion script
- [ ] `termcast completions fish` prints valid fish completion script
- [ ] Completions include all CLI flags
- [ ] Completions include saved location names for `--at`
- [ ] `termcast completions invalid` returns an error
- [ ] All new code has tests; `cargo test` passes; `cargo clippy` clean

## Open Questions

- ~~Should `--at` also accept city names directly?~~ **Resolved: Yes, `--at` falls back to geocoding if no saved entry matches.**
- ~~Should the config support multiple "profiles"?~~ **Resolved: No, not in v1.**
- ~~Should iteration 1 add `--config <path>`?~~ **Resolved: Yes.**

## Dependency Chain

```
Iteration 1 (Config File + --config)
    └── Iteration 2 (Named Locations + --at)
            └── Iteration 3 (Shell Completions)
```

Sequential. Each iteration builds on the previous.
