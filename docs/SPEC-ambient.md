# Spec: Ambient TermCast

## Objective

Enable persistent ambient weather awareness without a background daemon. Weather data is cached to disk and read by shell prompt hooks or tmux status bars on every render, providing "always visible" weather with zero daemon lifecycle complexity.

**User story:** "Every time my shell prompt appears, I see the current temperature and conditions without doing anything."

**Key design principle:** The shell prompt IS the daemon. No background process. No init scripts. No crash recovery.

## Tech Stack

- **Language:** Rust 2021 edition (existing)
- **Existing dependencies:**
  - `clap` — CLI argument parsing
  - `reqwest` — HTTP client
  - `serde` + `serde_json` — JSON serialization
  - `tokio` — async runtime
  - `thiserror` — error handling
- **New dependencies:** None required
- **APIs:** Same as existing (Open-Meteo, ipapi.co)

## Commands

```sh
# Build
cargo build --release

# Ambient mode (reads cache, outputs "☀️ 14°" or error)
./target/release/termcast --ambient

# Custom cache TTL (default 15 minutes)
./target/release/termcast --cache-ttl 30 --location "New York"
./target/release/termcast --cache-ttl 10  # Uses auto-detected location

# Regular weather (also populates cache)
./target/release/termcast
./target/release/termcast --location Oslo

# Shell integration setup (prints script to stdout)
./target/release/termcast --install

# Help
./target/release/termcast --help
./target/release/termcast --help-ambient  # Extended ambient help

# Run tests
cargo test

# Format / lint
cargo fmt --check
cargo clippy
```

## Project Structure

```
termcast/
├── src/
│   ├── main.rs          → CLI entry, argument parsing (MODIFIED)
│   ├── api.rs           → Open-Meteo + ipapi.co HTTP calls (unchanged)
│   ├── cache.rs         → NEW: Cache read/write logic
│   ├── weather.rs       → Weather types and icon mapping (unchanged)
│   ├── geolocation.rs   → Geolocation types (unchanged)
│   ├── renderer.rs       → Terminal rendering (unchanged)
│   └── errors.rs        → Error types (extended for cache errors)
├── integration/
│   ├── termcast.sh      → NEW: Shell integration scripts (bash/zsh)
│   └── termcast.tmux    → NEW: tmux integration snippet
├── docs/
│   ├── SPEC.md          → Original spec
│   └── SPEC-ambient.md  → THIS SPEC
└── tests/
    ├── api_tests.rs     → API integration tests (existing pattern)
    └── cache_tests.rs   → NEW: Cache unit tests
```

## Code Style

### Existing Conventions (preserved)

- `camelCase` for JSON fields parsed from APIs
- Error messages styled with `thiserror` enum variants
- All public functions have doc comments
- Tests in `#[cfg(test)]` modules within each source file

### New Conventions for Ambient

```rust
// Cache file structure (~/.cache/termcast/current)
struct CacheEntry {
    timestamp: i64,           // Unix timestamp of when data was fetched
    temperature: f64,         // Current temperature in Celsius
    weather_code: u32,        // WMO weather code
    location: String,         // Location name
    latitude: f64,            // For potential future use
    longitude: f64,           // For potential future use
}

// Ambient output format
// Output: "☀️ 14°" (icon + temperature, fits in any status bar)
// Error: "termcast: cache empty" or "termcast: cache stale"
```

### Cache Operations

```rust
// In cache.rs
pub fn read_cache(path: &Path) -> Result<Option<CacheEntry>>
pub fn write_cache(path: &Path, entry: &CacheEntry) -> Result<()>
pub fn is_cache_fresh(entry: &CacheEntry, ttl_secs: i64) -> bool

// Location: ~/.cache/termcast/current (XDG-compliant)
pub fn cache_path() -> PathBuf  // Respects $XDG_CACHE_HOME if set
```

### Shell Integration Script Pattern

```bash
# ~/.bashrc or ~/.zshrc
# Add to PROMPT_COMMAND (bash) or precmd hook (zsh)
termcast_prompt() {
    local weather=$(termcast --ambient 2>/dev/null)
    if [ -n "$weather" ]; then
        echo -n " $weather"
    fi
}

# Bash
PROMPT_COMMAND="${PROMPT_COMMAND:+${PROMPT_COMMAND}; }termcast_prompt"

# Zsh
precmd_functions+=(termcast_prompt)
```

### tmux Integration Pattern

```bash
# ~/.tmux.conf
set -g status-right "#{?window_icons,#{window_icons},}#{=22:pane_title}  #{=30:status-right-style} #[fg=colour233,bg=colour241,bold] #(/usr/local/bin/termcast --ambient) "
```

## Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Ambient Mode Flow                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Shell Prompt / tmux status bar                             │
│       │                                                      │
│       ▼                                                      │
│  termcast --ambient                                         │
│       │                                                      │
│       ▼                                                      │
│  Read ~/.cache/termcast/current                             │
│       │                                                      │
│       ├── Cache exists? ──No──▶ Fetch weather, write cache   │
│       │                              │                       │
│       │                              ▼                       │
│       │                         Read cache again             │
│       │                              │                       │
│       ▼                              ▼                       │
│  Check timestamp (fresh < TTL?)                             │
│       │                                                      │
│       ├── Fresh ──▶ Output "☀️ 14°"                         │
│       │                                                      │
│       └── Stale ──▶ Fetch weather, write cache, output      │
│                                                              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                  Regular Mode Flow                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  termcast (no --ambient flag)                               │
│       │                                                      │
│       ▼                                                      │
│  Fetch weather from API                                     │
│       │                                                      │
│       ▼                                                      │
│  Write to ~/.cache/termcast/current                         │
│       │                                                      │
│       ▼                                                      │
│  Render styled output to terminal                            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Ambient Output Format

**Success case:**
```
☀️ 14°
```
- Weather icon (1-2 Unicode chars)
- Space
- Temperature as integer with degree symbol
- Total: 4-6 characters, fits in any prompt or status bar

**Error cases:**
```
termcast: cache empty
```
```
termcast: cache stale
```
```
termcast: no location
```

Errors go to stderr, not stdout, so they don't pollute prompt display.

## Testing Strategy

### Unit Tests (per-module)

- **cache.rs**: Test cache read/write, freshness checking, path resolution
- **weather.rs**: Existing tests remain, no changes needed

### Integration Tests (tests/ directory)

- **cache_tests.rs**: Mock file system operations, test TTL logic
- **api_tests.rs**: Continue using wiremock for HTTP mocks

### Manual Verification Checklist

- [ ] `termcast --ambient` with fresh cache outputs "☀️ 14°"
- [ ] `termcast --ambient` with stale cache triggers refetch
- [ ] `termcast --ambient` with no cache triggers initial fetch
- [ ] `termcast` (regular mode) populates cache
- [ ] Shell prompt integration works in bash
- [ ] Shell prompt integration works in zsh
- [ ] tmux status bar integration works
- [ ] Error messages go to stderr, not stdout
- [ ] Cache respects custom `--cache-ttl` value
- [ ] Cache uses XDG path when `$XDG_CACHE_HOME` is set

## Boundaries

**Always:**
- Run `cargo test` before commits
- Format with `cargo fmt`
- Handle errors gracefully (no panics)
- Keep ambient output under 10 characters
- Write errors to stderr, not stdout

**Ask first:**
- Adding new dependencies
- Changing cache file location
- Modifying the 4-6 character output format
- Adding temperature unit conversion to ambient mode

**Never:**
- Add a background daemon process
- Create a systemd/init script
- Store secrets or API keys
- Call live APIs in tests
- Add weather alerts or notifications

## Success Criteria

1. [ ] `cargo build --release` compiles without warnings
2. [ ] `cargo test` passes 100%
3. [ ] `./target/release/termcast --ambient` outputs "☀️ 14°" (or similar) with fresh cache
4. [ ] `./target/release/termcast --ambient` refetches when cache is stale
5. [ ] `./target/release/termcast --ambient --cache-ttl 30` uses custom TTL
6. [ ] `./target/release/termcast` (regular) populates cache
7. [ ] `termcast --install` outputs valid bash/zsh script
8. [ ] Error messages go to stderr
9. [ ] Cache file is at `~/.cache/termcast/current` (or `$XDG_CACHE_HOME/termcast/current`)
10. [ ] Ambient output renders correctly in tmux status bar
11. [ ] Ambient output renders correctly in shell prompts (bash and zsh)

## Open Questions

### For This Spec (resolved inline)

- [x] **Cache format**: JSON with timestamp, temperature, weather_code, location
- [x] **Error output**: stderr for errors, stdout for data
- [x] **Location**: Same auto-detect logic as existing `--location` flag
- [x] **Temperature units**: Celsius only in ambient mode (keeps output simple)

### For Implementation (resolved)

1. **Install UX**: `--install` outputs the snippet only. User manually adds to their shell config. Safe, reversible, respects user autonomy.

2. **tmux integration**: Create a standalone `integration/termcast.tmux` file that users can source, plus documentation in `--help`.

3. **Cache permissions**: No special handling. Single-user by design, standard file permissions apply.

4. **Cache initial state**: `--ambient` fetches immediately if cache is empty. Users shouldn't need to run `termcast` first.

### Future Considerations (out of scope)

- Temperature unit toggle for ambient (°C vs °F)
- Colorized temperature in tmux (requires `status-interval` + script evaluation)
- Multi-location support
- Weather alerts
- Prompt animation

## Related Documentation

- Original spec: `docs/SPEC.md`
- Idea doc: `docs/ideas/termcast-ambient.md`
- This spec: `docs/SPEC-ambient.md`
