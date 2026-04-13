# Spec: Custom Color Themes

## Objective

Replace the 6 hardcoded colors in the terminal renderer with a theme engine that loads named color palettes from config. Users select a theme via `theme = "catppuccin"` in config.toml, browse available themes with `--list-themes`, and preview any theme with `--preview-theme <name>`.

**Target users:** Terminal enthusiasts who customize their emulator's color scheme (Catppuccin, Dracula, Nord, etc.) and want weather output to coordinate.

**Success criteria:**
- A user adds `theme = "dracula"` to config.toml and sees weather output in Dracula's palette on next run
- `termcast --list-themes` prints all 10 theme names with one-line descriptions
- `termcast --preview-theme nord` renders a static demo block in Nord's palette without requiring weather data
- No theme set produces identical output to today's hardcoded colors
- All existing tests continue to pass without modification

## Tech Stack

- **Language:** Rust (edition 2021)
- **Terminal styling:** crossterm 0.28 (already in use)
- **Color mode:** RGB (true-color) via `Color::Rgb { r, g, b }` with ANSI 256-color fallback via `Color::AnsiValue(...)`
- **Config:** TOML via `toml` crate (already in use)
- **CLI:** clap 4.5 with derive macros (already in use)

## Commands

```bash
# Build
cargo build

# Run all tests
cargo test

# Run with a theme
termcast --location Oslo
# (reads theme from config)

# Browse themes
termcast --list-themes

# Preview a specific theme
termcast --preview-theme catppuccin
```

## Project Structure

```
src/
  main.rs       → CLI entry point (add --list-themes, --preview-theme flags)
  lib.rs        → Module declarations (add pub mod theme)
  config.rs     → Config struct (add theme field to Defaults)
  theme.rs      → NEW: Theme struct, built-in theme definitions, resolution logic
  renderer.rs   → Terminal rendering (accept ThemeColors instead of hardcoded colors)
  weather.rs    → Weather data types (unchanged)
  forecast.rs   → Forecast data types (unchanged)
  api.rs        → HTTP client (unchanged)
  cache.rs      → Weather caching (unchanged)
  errors.rs     → Error types (unchanged)
  geolocation.rs → IP location (unchanged)
```

## Code Style

The theme engine follows existing project conventions:

```rust
// src/theme.rs — Theme struct with semantic color slots
use crossterm::style::Color;

pub struct ThemeColors {
    pub text: Color,
    pub dimmed: Color,
    pub temp_high: Color,
    pub temp_low: Color,
    pub precip_high: Color,
    pub precip_medium: Color,
}

pub struct Theme {
    pub name: &'static str,
    pub description: &'static str,
    pub colors: ThemeColors,
}

// Built-in themes as a static slice
pub fn builtin_themes() -> &'static [Theme] { ... }

// Resolve a theme name to its colors, or return defaults
pub fn resolve_theme(name: &str) -> &'static ThemeColors { ... }
```

Renderer functions accept a `&ThemeColors` parameter instead of using hardcoded colors:

```rust
// Before:
pub fn render_weather(weather: &WeatherDisplay, description: &str) -> io::Result<()> {
    stdout.queue(SetForegroundColor(Color::Cyan))?;  // hardcoded

// After:
pub fn render_weather(weather: &WeatherDisplay, description: &str, colors: &ThemeColors) -> io::Result<()> {
    stdout.queue(SetForegroundColor(colors.temp_high))?;  // from theme
```

Config integration follows the existing pattern:

```rust
// In Defaults struct:
#[serde(default)]
pub theme: String,  // empty string = no theme (use default colors)
```

## Semantic Color Mapping

The 6 semantic slots map to current hardcoded usage:

| Semantic slot    | Current hardcoded color | Used in                                    |
|-----------------|----------------------|--------------------------------------------|
| `text`          | `Color::White`       | Main line, condition description, separators, day names (non-today/tomorrow), hourly times |
| `dimmed`        | `Color::DarkGrey`    | "Feels like" line, low precip chance, hourly annotations |
| `temp_high`     | `Color::Cyan`        | High temperature values, "Today" day name  |
| `temp_low`      | `Color::Magenta`     | Low temperature values, "Tomorrow" day name |
| `precip_high`   | `Color::Red`         | Precipitation >= 80%                       |
| `precip_medium` | `Color::Yellow`      | Precipitation 50-79%                       |

## Built-in Themes (10)

Each theme provides all 6 colors as `Color::Rgb { r, g, b }` values. The "default" theme replicates the current hardcoded colors.

### Default
```rust
text: Rgb { r: 255, g: 255, b: 255 },      // White
dimmed: Rgb { r: 169, g: 169, b: 169 },     // DarkGrey
temp_high: Rgb { r: 0, g: 255, b: 255 },    // Cyan
temp_low: Rgb { r: 255, g: 0, b: 255 },     // Magenta
precip_high: Rgb { r: 255, g: 0, b: 0 },    // Red
precip_medium: Rgb { r: 255, g: 255, b: 0 }, // Yellow
```

### Catppuccin Mocha
```rust
text: Rgb { r: 205, g: 214, b: 244 },       // Text
dimmed: Rgb { r: 147, g: 153, b: 178 },     // Overlay0
temp_high: Rgb { r: 137, g: 180, b: 250 },  // Blue
temp_low: Rgb { r: 203, g: 166, b: 247 },   // Mauve
precip_high: Rgb { r: 243, g: 139, b: 168 },// Red
precip_medium: Rgb { r: 249, g: 226, b: 175 },// Yellow
```

### Catppuccin Latte
```rust
text: Rgb { r: 76, g: 79, b: 105 },         // Text
dimmed: Rgb { r: 150, g: 153, b: 166 },     // Overlay0
temp_high: Rgb { r: 30, g: 102, b: 245 },   // Blue
temp_low: Rgb { r: 136, g: 57, b: 239 },    // Mauve
precip_high: Rgb { r: 210, g: 15, b: 57 },  // Red
precip_medium: Rgb { r: 223, g: 186, b: 52 },// Yellow
```

### Dracula
```rust
text: Rgb { r: 248, g: 248, b: 242 },       // Foreground
dimmed: Rgb { r: 98, g: 114, b: 164 },      // Comment
temp_high: Rgb { r: 189, g: 147, b: 249 },  // Purple
temp_low: Rgb { r: 255, g: 121, b: 198 },   // Pink
precip_high: Rgb { r: 255, g: 85, b: 85 },  // Red
precip_medium: Rgb { r: 241, g: 250, b: 140 },// Yellow
```

### Nord
```rust
text: Rgb { r: 216, g: 222, b: 233 },       // Snow Storm 2
dimmed: Rgb { r: 76, g: 86, b: 106 },       // Polar Night 3
temp_high: Rgb { r: 136, g: 192, b: 208 },  // Frost Blue
temp_low: Rgb { r: 180, g: 142, b: 173 },   // Frost
precip_high: Rgb { r: 191, g: 97, b: 106 }, // Aurora Red
precip_medium: Rgb { r: 235, g: 203, b: 139 },// Aurora Yellow
```

### Solarized Dark
```rust
text: Rgb { r: 147, g: 161, b: 161 },       // Base0
dimmed: Rgb { r: 88, g: 110, b: 117 },      // Base01
temp_high: Rgb { r: 42, g: 161, b: 152 },   // Cyan
temp_low: Rgb { r: 108, g: 113, b: 196 },   // Violet
precip_high: Rgb { r: 220, g: 50, b: 47 },  // Red
precip_medium: Rgb { r: 181, g: 137, b: 0 },// Yellow
```

### Solarized Light
```rust
text: Rgb { r: 101, g: 123, b: 131 },       // Base00
dimmed: Rgb { r: 147, g: 161, b: 161 },     // Base0
temp_high: Rgb { r: 42, g: 161, b: 152 },   // Cyan
temp_low: Rgb { r: 108, g: 113, b: 196 },   // Violet
precip_high: Rgb { r: 220, g: 50, b: 47 },  // Red
precip_medium: Rgb { r: 181, g: 137, b: 0 },// Yellow
```

### Tokyo Night
```rust
text: Rgb { r: 169, g: 177, b: 214 },       // fg
dimmed: Rgb { r: 86, g: 95, b: 137 },       // dark3
temp_high: Rgb { r: 122, g: 162, b: 247 },  // blue
temp_low: Rgb { r: 187, g: 154, b: 247 },   // purple
precip_high: Rgb { r: 247, g: 118, b: 142 },// red
precip_medium: Rgb { r: 224, g: 175, b: 104 },// orange
```

### Tokyo Night Light
```rust
text: Rgb { r: 52, g: 59, b: 85 },          // fg
dimmed: Rgb { r: 150, g: 153, b: 166 },     // comment
temp_high: Rgb { r: 50, g: 107, b: 215 },   // blue
temp_low: Rgb { r: 116, g: 75, b: 207 },    // purple
precip_high: Rgb { r: 225, g: 65, b: 80 },  // red
precip_medium: Rgb { r: 188, g: 144, b: 40 },// orange
```

### Gruvbox
```rust
text: Rgb { r: 235, g: 219, b: 178 },       // fg1
dimmed: Rgb { r: 146, g: 131, b: 116 },     // fg3
temp_high: Rgb { r: 104, g: 157, b: 106 },  // green bright
temp_low: Rgb { r: 215, g: 153, b: 33 },    // yellow bright
precip_high: Rgb { r: 204, g: 36, b: 29 },  // red bright
precip_medium: Rgb { r: 214, g: 93, b: 14 },// orange bright
```

### Gruvbox Light
```rust
text: Rgb { r: 60, g: 56, b: 54 },          // fg1
dimmed: Rgb { r: 124, g: 111, b: 100 },     // fg3
temp_high: Rgb { r: 50, g: 130, b: 52 },    // green
temp_low: Rgb { r: 188, g: 134, b: 24 },    // yellow
precip_high: Rgb { r: 157, g: 0, b: 6 },    // red
precip_medium: Rgb { r: 214, g: 93, b: 14 },// orange
```

## RGB with ANSI 256 Fallback

True-color terminals render `Color::Rgb { r, g, b }` directly. For terminals without true-color support, map each RGB value to the nearest ANSI 256-color index.

Detection: check `COLORTERM` environment variable. If set to `truecolor` or `24bit`, use RGB directly. Otherwise, convert to nearest `Color::AnsiValue(...)`.

```rust
impl ThemeColors {
    /// Returns this theme's colors converted to ANSI 256-color values.
    pub fn to_ansi256(&self) -> ThemeColors { ... }
}
```

## CLI Changes

### New flags on `Args`

```rust
/// List available color themes with descriptions.
#[arg(long)]
list_themes: bool,

/// Preview a color theme with a demo weather display.
#[arg(long)]
preview_theme: Option<String>,
```

### Flag behavior

**`--list-themes`**: Prints all built-in theme names (including "default") with one-line descriptions, one per line. Does not fetch weather. Returns immediately.

```
$ termcast --list-themes
  default           Current hardcoded colors (white, cyan, magenta)
  catppuccin        Catppuccin Mocha - warm dark pastels
  catppuccin-latte  Catppuccin Latte - warm light pastels
  dracula           Dracula - dark with vivid accents
  nord              Nord - arctic blue-gray palette
  solarized         Solarized Dark - warm earth tones
  solarized-light   Solarized Light - warm cream with accents
  tokyo-night       Tokyo Night - deep blue city nights
  tokyo-night-light Tokyo Night Light - cool day variant
  gruvbox           Gruvbox - warm retro earth tones
  gruvbox-light     Gruvbox Light - warm light retro tones
```

**`--preview-theme <name>`**: Renders a static demo weather display using the named theme's colors. Does not fetch weather data. The demo block shows all 6 semantic colors in context:

```
$ termcast --preview-theme dracula
       ☀️  14°C Demo City
     Feels 11°C
     High 17°C · Low 8°C
     Clear skies and warm

  Today     ☀️  17°C/8°C     ☂ 5%
  Tomorrow  🌤 15°C/7°C     ☂ 60%
  Wed       🌧 12°C/5°C     ☂ 85%
```

Error on unknown theme name:
```
$ termcast --preview-theme nonexistent
termcast: unknown theme 'nonexistent'. Use --list-themes to see available themes.
```

## Config Integration

Add `theme` field to `[defaults]` section:

```toml
[defaults]
default_location = "home"
units = "auto"
cache_ttl = 15
theme = "catppuccin"
```

- Empty string or missing field = no theme (current hardcoded colors, same as `"default"`)
- Unknown theme name = warning to stderr, fall back to default colors
- Theme name matching is case-insensitive and supports hyphens/underscores interchangeably (e.g., `"catppuccin-mocha"`, `"catppuccin_mocha"`, `"catppuccin"` all map to Catppuccin Mocha)

## Theme Name Aliases

| Alias              | Canonical theme   |
|--------------------|-------------------|
| `catppuccin`       | Catppuccin Mocha  |
| `catppuccin-mocha` | Catppuccin Mocha  |
| `catppuccin-latte` | Catppuccin Latte  |
| `solarized`        | Solarized Dark    |
| `solarized-dark`   | Solarized Dark    |
| `tokyo-night`      | Tokyo Night       |
| `gruvbox`          | Gruvbox           |
| `gruvbox-dark`     | Gruvbox           |

## Error Styling

Error output (`render_error`, "Error:" prefix) remains hardcoded as `Color::Red` — not part of themes. Keeps error visibility consistent regardless of theme.

## Testing Strategy

- **Unit tests in `theme.rs`:** Verify each built-in theme resolves correctly, unknown theme fallback, alias resolution, case-insensitive matching
- **Unit tests in `config.rs`:** Verify `theme` field parsing from TOML, default empty string behavior
- **Unit tests in `renderer.rs`:** Update existing tests to pass `ThemeColors::default()`, add test for `render_preview_theme` output
- **CLI tests in `main.rs`:** Test `--list-themes` and `--preview-theme` flag parsing
- **Integration:** `cargo test` runs everything. No new test dependencies needed.

## Boundaries

### Always
- Run `cargo test` before committing
- Maintain all existing tests (update signatures, don't delete)
- Use `Color::Rgb` for theme colors (with ANSI 256 fallback)
- All 6 semantic colors defined for every theme
- Theme names are lowercase, hyphenated

### Ask first
- Adding new theme palettes beyond the 10 specified
- Changing the `ThemeColors` struct (adding/removing semantic slots)
- Adding new CLI flags beyond `--list-themes` and `--preview-theme`
- Changing the theme name format or alias convention

### Never
- Theme ambient mode output (too minimal to benefit)
- Add external theme file loading from `~/.config/termcast/themes/`
- Auto-detect terminal theme from environment variables
- Add per-role hex overrides in config
- Create a `themes` subcommand (flags are sufficient)

## Open Questions

None — all design decisions resolved during idea refinement.
