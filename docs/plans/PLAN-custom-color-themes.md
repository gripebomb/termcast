# Implementation Plan: Custom Color Themes

## Overview

Replace the 6 hardcoded colors in the terminal renderer with a theme engine that loads named color palettes from config. Users select a theme via `theme = "catppuccin"` in config.toml, browse available themes with `--list-themes`, and preview any theme with `--preview-theme <name>`.

## Architecture Decisions

- **Theme colors as `Color::Rgb`** with ANSI 256 fallback via `COLORTERM` env check — matches spec requirement for true-color with graceful degradation
- **Static built-in themes** — no file I/O for themes, all defined as a const slice in `theme.rs`
- **`&ThemeColors` parameter threading** — renderer functions receive theme colors as a parameter rather than accessing global state, keeping functions testable and side-effect-free
- **Case-insensitive, hyphen/underscore-normalized matching** — all theme lookups normalize the input to lowercase with hyphens for reliable alias resolution

## Dependency Graph

```
theme.rs (ThemeColors struct, built-in definitions, resolve_theme)
    |
    |-- lib.rs (add pub mod theme)
    |
    |-- config.rs (add theme: String field to Defaults)
    |
    |-- renderer.rs (accept &ThemeColors, replace hardcoded colors)
    |       |
    |       +-- main.rs (resolve theme from config, pass to renderer)
    |
    +-- main.rs (--list-themes, --preview-theme flags)
```

## Task List

### Phase 1: Foundation

#### Task 1: Create `theme.rs` with ThemeColors struct and default theme

**Description:** Create the new `theme.rs` module with the `ThemeColors` struct (6 color fields), `Theme` struct (name + description + colors), `builtin_themes()` returning all 11 themes as a static slice, and `resolve_theme()` that normalizes input and returns `&ThemeColors`. Also implement `default_colors()` that returns the current hardcoded colors as `ThemeColors`, and `to_ansi256()` conversion method stub. Register the module in `lib.rs`.

**Acceptance criteria:**
- [x] `ThemeColors` struct has 6 fields: `text`, `dimmed`, `temp_high`, `temp_low`, `precip_high`, `precip_medium` (all `Color`)
- [x] `Theme` struct has `name`, `description`, `colors` fields
- [x] `builtin_themes()` returns 11 themes (default + 10 named)
- [x] `resolve_theme("default")` returns colors matching current hardcoded values
- [x] `resolve_theme("nonexistent")` returns default colors
- [x] `resolve_theme("CATPPUCCIN")` returns Catppuccin Mocha colors (case-insensitive)
- [x] `resolve_theme("catppuccin_mocha")` returns Catppuccin Mocha colors (underscore = hyphen)
- [x] All alias mappings from spec work
- [x] `pub mod theme` added to `lib.rs`

**Verification:**
- [x] `cargo test` passes (new unit tests in `theme.rs`)
- [x] `cargo build` succeeds

**Dependencies:** None

**Files likely touched:**
- `src/theme.rs` (NEW)
- `src/lib.rs` (add `pub mod theme`)

**Estimated scope:** S (2 files)

---

#### Task 2: Add `theme` field to config

**Description:** Add a `theme: String` field to the `Defaults` struct in `config.rs` with a default of empty string. Update the `Default` impl and the `default()` function. Empty string means "use default colors" (no theme).

**Acceptance criteria:**
- [x] `Defaults` struct has a `theme` field defaulting to `""`
- [x] TOML parsing reads `theme = "dracula"` from `[defaults]` section
- [x] Config without `theme` key gets empty string (existing behavior preserved)
- [x] All existing config tests pass unchanged

**Verification:**
- [x] `cargo test` passes (add 2-3 new config tests for theme field)
- [x] `cargo build` succeeds

**Dependencies:** None (parallel with Task 1)

**Files likely touched:**
- `src/config.rs`

**Estimated scope:** XS (1 file)

---

### Checkpoint: Foundation
- [x] All tests pass: `cargo test`
- [x] Build succeeds: `cargo build`
- [x] Theme resolution logic works independently of rendering

---

### Phase 2: Renderer Integration

#### Task 3: Update renderer to accept `&ThemeColors` parameter

**Description:** Update all renderer functions (`render_weather`, `render_forecast`, `render_forecast_hourly`) to accept a `&ThemeColors` parameter and replace all hardcoded color constants with the corresponding theme color slot. Leave `render_error` unchanged (spec: error output stays hardcoded `Color::Red`). Leave `output_ambient_forecast` unchanged (spec: ambient mode not themed).

**Acceptance criteria:**
- [x] `render_weather` signature becomes `render_weather(weather, description, colors: &ThemeColors)`
- [x] `render_forecast` signature becomes `render_forecast(display, colors: &ThemeColors)`
- [x] `render_forecast_hourly` signature becomes `render_forecast_hourly(display, colors: &ThemeColors)`
- [x] All 6 hardcoded colors (`White`, `DarkGrey`, `Cyan`, `Magenta`, `Red`, `Yellow`) replaced with theme color slots per the semantic mapping table
- [x] `render_error` still uses hardcoded `Color::Red` and `Color::White`
- [x] `output_ambient_forecast` unchanged (no colors)
- [x] All existing renderer tests updated to pass `&ThemeColors` default and pass

**Verification:**
- [x] `cargo test` passes
- [x] `cargo build` succeeds

**Dependencies:** Task 1 (needs `ThemeColors` type)

**Files likely touched:**
- `src/renderer.rs`
- `src/main.rs` (callers of renderer functions)

**Estimated scope:** M (2 files, many call sites)

---

#### Task 4: Wire theme resolution into main.rs

**Description:** Update all call sites in `main.rs` that invoke renderer functions to resolve the theme from config and pass `&ThemeColors`. Add theme resolution after config loading in `run()`, `fetch_and_display_weather()`, and `run_forecast()`.

**Acceptance criteria:**
- [x] `run()` resolves theme from `cfg.defaults.theme` and passes colors to `fetch_and_display_weather()`
- [x] `run_forecast()` resolves theme from config and passes colors to `render_forecast()` / `render_forecast_hourly()`
- [x] Unknown theme name prints warning to stderr and falls back to default colors
- [x] Empty string or missing theme = default colors (identical to current output)

**Verification:**
- [x] `cargo test` passes
- [x] `cargo build` succeeds
- [x] `cargo run -- -l Oslo` with no theme in config produces identical output to pre-theme code

**Dependencies:** Task 2, Task 3

**Files likely touched:**
- `src/main.rs`

**Estimated scope:** S (1 file)

---

### Checkpoint: Renderer Integration
- [x] All tests pass: `cargo test`
- [x] Build succeeds: `cargo build`
- [x] Running `termcast -l Oslo` produces correct output with default (no theme)
- [x] Adding `theme = "dracula"` to config changes output colors

---

### Phase 3: CLI Flags

#### Task 5: Add `--list-themes` and `--preview-theme` CLI flags

**Description:** Add two new flags to the `Args` struct: `--list-themes` (bool) and `--preview-theme` (Option<String>). Handle them in `run()` before any weather fetching. `--list-themes` prints all theme names with descriptions. `--preview-theme` renders a static demo block using the named theme's colors. Error on unknown theme name.

**Acceptance criteria:**
- [x] `--list-themes` flag parses correctly
- [x] `--list-themes` prints all 11 theme names with descriptions, one per line
- [x] `--preview-theme catppuccin` renders demo block with Catppuccin colors
- [x] `--preview-theme nonexistent` prints error message and exits with code 1
- [x] Both flags return immediately without fetching weather
- [x] Existing CLI tests continue to pass

**Verification:**
- [x] `cargo test` passes (add 3-4 new tests for flag parsing)
- [x] `cargo build` succeeds
- [x] `cargo run -- --list-themes` prints expected output
- [x] `cargo run -- --preview-theme dracula` renders demo block

**Dependencies:** Task 1, Task 3

**Files likely touched:**
- `src/main.rs` (CLI flags, flag handling)
- `src/renderer.rs` (new `render_preview_theme` function)

**Estimated scope:** M (2 files)

---

### Phase 4: Polish

#### Task 6: ANSI 256-color fallback

**Description:** Implement `to_ansi256()` on `ThemeColors` that converts RGB values to the nearest ANSI 256-color index. Check `COLORTERM` environment variable at render time — if not `truecolor` or `24bit`, use the ANSI 256 fallback colors.

**Acceptance criteria:**
- [x] `ThemeColors::to_ansi256()` converts each `Color::Rgb` to `Color::AnsiValue`
- [x] Renderer checks `COLORTERM` and uses ANSI 256 when true-color not supported
- [x] True-color terminals (COLORTERM=truecolor) use RGB directly
- [x] Default colors still render correctly in both modes

**Verification:**
- [x] `cargo test` passes (add tests for ANSI conversion)
- [x] `cargo build` succeeds

**Dependencies:** Task 1, Task 3

**Files likely touched:**
- `src/theme.rs` (to_ansi256 implementation)

**Estimated scope:** S (1 file)

---

### Checkpoint: Complete
- [x] All acceptance criteria from spec met
- [x] `cargo test` passes
- [x] `cargo build --release` succeeds
- [x] Manual verification of all 11 themes via `--preview-theme`
- [x] Ready for code review

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Renderer function signature changes break callers | Medium | Update all call sites in same task (Task 3), verify with `cargo test` |
| ANSI 256 color conversion inaccuracy | Low | Use standard RGB-to-216-cube algorithm; test against known values |
| Theme name collision or ambiguity | Low | Normalize all names to lowercase-hyphenated before comparison |
| Forgetting to thread `&ThemeColors` through a code path | Medium | Compiler catches it — function signatures require the parameter |

## Open Questions

None — all design decisions resolved in spec.
