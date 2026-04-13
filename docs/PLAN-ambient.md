# Implementation Plan: Ambient TermCast

## Overview

Implement persistent ambient weather awareness for shell prompts and tmux status bars. Weather data caches to disk and is read on demand, with zero daemon complexity — the shell prompt IS the daemon.

## Architecture Decisions

1. **New `cache.rs` module** — Handles read/write of JSON cache at `~/.cache/termcast/current` (XDG-compliant)
2. **Cache entry structure** — `{ timestamp, temperature, weather_code, location, latitude, longitude }`
3. **Ambient mode adds new CLI flags** — `--ambient`, `--cache-ttl`, `--install`
4. **Shell integration via `--install`** — Outputs bash/zsh snippet for manual addition to shell config
5. **Sync file I/O for cache** — Use `std::fs` instead of async. Simple, sufficient for once-per-15-min cadence
6. **Cache-first with auto-fetch** — `--ambient` always tries cache first. If missing or stale, fetch happens inline

## Task List

### Phase 1: Foundation (Cache Module)

---

**Task 1: Create `src/cache.rs` module**

**Description:** Implement cache read/write/freshness logic. Handles path resolution, serialization, timestamp checking.

**Acceptance criteria:**
- [ ] `cache_path() -> PathBuf` returns `~/.cache/termcast/current` or `$XDG_CACHE_HOME/termcast/current`
- [ ] `CacheEntry` struct with `timestamp: i64`, `temperature: f64`, `weather_code: u32`, `location: String`, `latitude: f64`, `longitude: f64`
- [ ] `read_cache(path: &Path) -> Result<Option<CacheEntry>>` returns `None` if file missing/invalid
- [ ] `write_cache(path: &Path, entry: &CacheEntry) -> Result<()>` creates parent dirs if needed, writes JSON
- [ ] `is_cache_fresh(entry: &CacheEntry, ttl_secs: i64) -> bool` checks timestamp against TTL
- [ ] Unit tests for path resolution, read/write, freshness checking

**Verification:**
- `cargo test --lib -- cache`
- Manual: `ls ~/.cache/termcast/` after running termcast

**Dependencies:** None

**Files:** `src/cache.rs` (new)

**Scope:** S (1 file)

---

**Task 2: Extend `src/errors.rs` with cache error variants**

**Description:** Add cache-related error variants to `AppError` enum.

**Acceptance criteria:**
- [ ] `CacheError` variant added (covers read/write/parse failures)
- [ ] Error messages are descriptive and actionable
- [ ] `impl From<std::io::Error>` handles file system errors

**Verification:**
- `cargo test --lib -- errors`
- Existing tests still pass

**Dependencies:** None

**Files:** `src/errors.rs`

**Scope:** XS (add enum variant)

---

### Checkpoint: Foundation
- [ ] `cargo build` compiles without errors
- [ ] `cargo test --lib` passes
- [ ] `cache_path()` resolves correctly in both XDG and fallback cases

---

### Phase 2: CLI Integration

---

**Task 3: Extend CLI arguments in `src/main.rs`**

**Description:** Add `--ambient`, `--cache-ttl`, and `--install` flags to Args struct.

**Acceptance criteria:**
- [ ] `--ambient` flag (boolean) — runs in ambient mode
- [ ] `--cache-ttl <minutes>` flag (integer, default 15)
- [ ] `--install` flag (boolean) — outputs shell integration snippets
- [ ] `--location` flag still works
- [ ] `termcast --help` shows new flags

**Verification:**
- `cargo run -- --help | grep -E "(ambient|cache-ttl|install)"`
- `cargo test -- main`

**Dependencies:** Tasks 1-2

**Files:** `src/main.rs`

**Scope:** S (1 file)

---

### Phase 3: Ambient Mode Implementation

---

**Task 4: Implement `--ambient` mode logic**

**Description:** Core ambient flow: read cache → check freshness → fetch if needed → output.

**Acceptance criteria:**
- [ ] `--ambient` reads cache file first
- [ ] If cache missing: fetch weather, write cache, output
- [ ] If cache stale (or `--cache-ttl` exceeded): fetch weather, write cache, output
- [ ] If cache fresh: output immediately without network call
- [ ] Output format: `☀️ 14°` (icon + temperature, 4-6 chars)
- [ ] Errors go to stderr (not stdout): `"termcast: cache empty"`, `"termcast: cache stale"`, `"termcast: no location"`
- [ ] Uses `--cache-ttl` value (default 15 min)
- [ ] Uses `--location` or auto-detect

**Verification:**
- `./target/release/termcast --ambient` with fresh cache outputs "☀️ 14°"
- `./target/release/termcast --ambient` refetches when cache stale
- `./target/release/termcast --ambient` with no cache triggers initial fetch
- `ls ~/.cache/termcast/current` exists after run
- `cat ~/.cache/termcast/current` is valid JSON

**Dependencies:** Tasks 1-2, Task 3

**Files:** `src/main.rs`, `src/cache.rs`

**Scope:** M (2 files)

---

**Task 5: Implement `--install` output**

**Description:** Output shell integration snippets for bash/zsh/tmux.

**Acceptance criteria:**
- [ ] `--install bash` outputs bash-compatible snippet
- [ ] `--install zsh` outputs zsh-compatible snippet
- [ ] `--install tmux` outputs tmux configuration snippet
- [ ] `--install` (no sub-flag) outputs all three with usage

**Verification:**
- `cargo run -- --install bash` → valid bash syntax
- `cargo run -- --install zsh` → valid zsh syntax
- `cargo run -- --install tmux` → valid tmux config

**Dependencies:** Task 3

**Files:** `src/main.rs`

**Scope:** S (1 file)

---

**Task 6: Ensure regular `termcast` populates cache**

**Description:** Modify existing flow to write cache after fetching weather (both regular and ambient modes).

**Acceptance criteria:**
- [ ] `termcast` (no flags) fetches and displays weather AND writes cache
- [ ] `termcast --location Oslo` fetches and displays weather AND writes cache
- [ ] Cache written after successful API call

**Verification:**
- Delete `~/.cache/termcast/current`
- `cargo run` → cache file created
- Check cache timestamp is recent

**Dependencies:** Task 1, Task 4

**Files:** `src/main.rs`, `src/cache.rs`

**Scope:** S (2 files)

---

### Checkpoint: Core Integration
- [ ] `cargo build --release` compiles without warnings
- [ ] `cargo test` passes 100%
- [ ] `--ambient` outputs weather format `☀️ 14°`
- [ ] `--install` outputs valid shell snippets
- [ ] Regular `termcast` populates cache

---

### Phase 4: Integration Scripts

---

**Task 7: Create `integration/termcast.sh`**

**Description:** Shell integration script that users can source. Contains `termcast_prompt()` function and setup instructions.

**Acceptance criteria:**
- [ ] Script detects bash vs zsh and sets appropriate hooks
- [ ] Contains `termcast_prompt()` function
- [ ] Function calls `termcast --ambient`
- [ ] Outputs to stdout for prompt integration
- [ ] Comments explain how to install

**Verification:**
- `source integration/termcast.sh` → no errors
- `type termcast_prompt` → function exists

**Dependencies:** Tasks 4-5

**Files:** `integration/termcast.sh` (new)

**Scope:** S (1 file)

---

**Task 8: Create `integration/termcast.tmux`**

**Description:** tmux integration snippet for `status-right`.

**Acceptance criteria:**
- [ ] Contains tmux configuration snippet
- [ ] Uses `#{...}` syntax for proper tmux evaluation
- [ ] Adds weather to right side of status bar
- [ ] Comments explain how to install

**Verification:**
- Manual: source file in tmux config and verify status bar shows weather

**Dependencies:** Tasks 4-5

**Files:** `integration/termcast.tmux` (new)

**Scope:** S (1 file)

---

### Phase 5: Testing

---

**Task 9: Add unit tests for cache module**

**Description:** Comprehensive tests for cache.rs functions.

**Acceptance criteria:**
- [ ] Test `cache_path()` with XDG set
- [ ] Test `cache_path()` with XDG not set
- [ ] Test `is_cache_fresh()` with fresh timestamp
- [ ] Test `is_cache_fresh()` with stale timestamp
- [ ] Test `read_cache()` with valid JSON
- [ ] Test `read_cache()` with invalid JSON (graceful error)
- [ ] Test `read_cache()` with missing file

**Verification:**
- `cargo test -- cache`

**Dependencies:** Task 1

**Files:** `src/cache.rs` (test module)

**Scope:** S (1 file)

---

**Task 10: Manual verification checklist**

**Description:** Run through full manual verification to ensure everything works end-to-end.

**Acceptance criteria:**
- [ ] `--ambient` with fresh cache outputs quickly
- [ ] `--ambient` with stale cache triggers refetch
- [ ] `--ambient` with no cache triggers initial fetch
- [ ] `--ambient --cache-ttl 30` uses custom TTL
- [ ] `--install bash/zsh/tmux` outputs valid snippets
- [ ] Shell prompt integration works (bash)
- [ ] Shell prompt integration works (zsh)
- [ ] tmux status bar shows weather
- [ ] Error messages go to stderr, not stdout

**Verification:**
- Manual testing per checklist item

**Dependencies:** Tasks 4-9

**Files:** All

**Scope:** M (verification)

---

### Checkpoint: Complete
- [ ] All tasks complete
- [ ] `cargo build --release` succeeds
- [ ] `cargo test` passes
- [ ] All acceptance criteria verified
- [ ] Spec updated to reflect any implementation changes
- [ ] Ready for code review

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Cache file locking on NFS/home dirs | Low | Use standard file operations, document limitation |
| Network timeout on cold start | Med | 10s timeout already set in reqwest client |
| Unicode rendering issues in terminals | Low | Test on iTerm2/macOS Terminal/kitty; use widely-supported emoji |
| Cache directory permissions | Low | Create with standard perms; single-user by design |

## Dependencies Summary

```
Task 1 (cache.rs) ──────────────────────────────────────────────┐
    │                                                            │
    ├── Task 2 (errors.rs)                                       │
    │       └── Checkpoint: Foundation                          │
    │                                                            │
Task 3 (main.rs CLI) ───────────────────────────────────────────┤
    │                                                            │
    ├── Task 4 (--ambient logic) ────────────────────────────────┤
    │       └── Task 6 (regular mode cache) ─────────────────────┤
    │               └── Checkpoint: Core                         │
    │                                                            │
    ├── Task 5 (--install) ──────────────────────────────────────┤
    │                                                            │
    └── Tasks 7-8 (integration scripts) ────────────────────────┤
                                                                 │
Task 9 (tests) ──────────────────────────────────────────────────┘
    │
Task 10 (manual verification)
    │
    └── Checkpoint: Complete
```
