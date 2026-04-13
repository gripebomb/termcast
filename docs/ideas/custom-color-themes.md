# Custom Color Themes

## Problem Statement

How might we let termcast users personalize the visual output to match their terminal aesthetic, without adding complexity to a tool whose strength is simplicity?

## Recommended Direction

Add a theme engine that swaps the 6 hardcoded renderer colors for named palettes, selected via a single `theme` line in config.toml. Ship 10 built-in themes (dark + light variants of Catppuccin, Solarized, Tokyo Night, Gruvbox, plus Dracula and Nord). Provide `--list-themes` and `--preview-theme` flags so users can browse and see live weather output in each palette before committing.

The core value is the theme engine (define palettes, load from config, render with palette). The list/preview flags are a discovery layer on top — they make the feature self-documenting and reduce the need to explain themes in a README.

## Key Assumptions to Validate

- [ ] Users with themed terminals will notice and appreciate coordinated colors (test: ship it, watch for feedback)
- [ ] 10 built-in themes cover the majority of themed terminals people actually use (test: track requests for additional themes)
- [ ] Preview with real weather data is useful even though it doesn't show all semantic colors every time (test: user feedback)
- [ ] Flag-based UX (--list-themes, --preview-theme) is discoverable enough (test: see if users find it without docs)

## MVP Scope

**In:**
- Theme struct with 6 semantic color slots: `text`, `dimmed`, `temp_high`, `temp_low`, `precip_high`, `precip_medium`
- 10 built-in themes: Catppuccin Mocha, Catppuccin Latte, Dracula, Nord, Solarized Dark, Solarized Light, Tokyo Night, Tokyo Night Light, Gruvbox, Gruvbox Light
- `theme = "catppuccin"` field in config.toml under `[defaults]`
- `--list-themes` flag: print theme names with a brief description
- `--preview-theme <name>` flag: render current weather using that theme's colors (uses cached or live data). Falls back to a static demo block showing all semantic colors when no weather data is available
- Renderer reads from active theme instead of hardcoded colors
- Default behavior: no theme set = current hardcoded colors. Also available as named theme `"default"` so users can switch back explicitly

**Out:**
- Custom color definitions (no hex overrides per semantic role)
- Loading external theme files from `~/.config/termcast/themes/`
- Auto-detecting terminal theme from env vars
- Theme support for ambient mode (single icon + temp — not worth the complexity)
- A `themes` subcommand (using flags instead)

## Not Doing (and Why)

- **Per-role color overrides** — Adds config surface area for a feature we explicitly scoped as "presets only." If users want custom colors, that's a separate feature.
- **External theme files** — Extensibility before we know if 6 themes is enough. Premature abstraction.
- **Auto-detection** — Fragile across terminal emulators, incomplete coverage. The `theme = "catppuccin"` one-liner is simpler and more reliable.
- **Ambient mode theming** — Ambient output is a single icon + temp — two colors at most. Not worth the lookup overhead for a format designed to be minimal.
- **Theme gallery / sharing** — Ecosystem play before we have evidence of demand. Ship built-in themes first.

## Resolved Decisions

- **Default behavior:** No theme set = current hardcoded colors. Also available as named theme `"default"` for explicit switching.
- **Offline preview:** `--preview-theme` falls back to a static demo block showing all semantic colors when no cached/live weather data is available.
- **Light + dark:** Include both dark and light theme variants from the start (e.g., Catppuccin Mocha + Latte, Solarized Dark + Light, Tokyo Night + Tokyo Night Light, Gruvbox + Gruvbox Light).
