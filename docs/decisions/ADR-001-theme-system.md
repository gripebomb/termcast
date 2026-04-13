# ADR-001: Static built-in color themes with semantic color slots

## Status
Accepted

## Date
2026-04-13

## Context
TermCast renders weather output with colored text (temperatures, precipitation, conditions). Originally, colors were hardcoded directly in the renderer as `Color::Rgb` values. Users wanted to match the weather output to their terminal color scheme (catppuccin, dracula, nord, etc.).

Key requirements:
- Support multiple named color palettes
- Zero config — default behavior unchanged
- No runtime file I/O or network requests for themes
- Work across different terminal emulators
- Easy to add new themes in the future

## Decision
Use a static theme engine with semantic color slots, compiled into the binary.

### Semantic Color Slots
Define 6 named color roles that the renderer uses instead of hardcoded colors:

| Slot | Usage |
|------|-------|
| `text` | Main text, conditions, separators |
| `dimmed` | "Feels like", low precipitation, secondary info |
| `temp_high` | High temperatures, "Today" label |
| `temp_low` | Low temperatures, "Tomorrow" label |
| `precip_high` | High precipitation (80%+) |
| `precip_medium` | Medium precipitation (50-79%) |

### Static Themes
All themes are defined as a `static` array of `Theme` structs in `src/theme.rs`. No theme files are loaded from disk. Each theme provides RGB values for all 6 slots.

### Theme Resolution
- Case-insensitive matching (`Dracula` = `dracula` = `DRACULA`)
- Hyphens and underscores equivalent (`tokyo_night` = `tokyo-night`)
- Alias support (`catppuccin-mocha` → `catppuccin`, `gruvbox-dark` → `gruvbox`)
- Unknown names fall back to default with a warning to stderr

## Alternatives Considered

### File-based themes (TOML/JSON theme files in config directory)
- **Pros:** Users can create custom themes without recompiling; community themes can be shared as files
- **Cons:** File I/O on every render; need error handling for malformed files; need a file format spec; need directory creation; themes could be inconsistent (missing slots)
- **Rejected:** Adds complexity for marginal benefit. Most users want popular terminal themes, which we can bundle. Can be added later without breaking changes.

### Crossterm named colors only (no RGB)
- **Pros:** Maximum terminal compatibility; no fallback logic needed
- **Cons:** Only 16 named colors; cannot match popular themes like catppuccin or dracula which use specific RGB values
- **Rejected:** Defeats the purpose of themes. The aesthetic quality depends on precise color matching.

### Dynamic theme loading from a crate or plugin system
- **Pros:** Extensible without modifying source
- **Cons:** Massive over-engineering for a CLI tool; adds build complexity
- **Rejected:** YAGNI. 11 built-in themes cover the popular terminal color schemes.

## Consequences
- Adding a new theme requires editing `src/theme.rs` and recompiling
- All themes are guaranteed to define all 6 color slots (compile-time safety)
- No theme file parsing errors to handle at runtime
- Binary size increase is negligible (11 themes x 6 RGB values = ~198 bytes)
- User-defined theme files can be added in a future version by loading from `$XDG_CONFIG_HOME/termcast/themes/` and merging with built-ins, without breaking the current API
