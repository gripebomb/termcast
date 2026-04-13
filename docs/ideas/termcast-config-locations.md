# TermCast Config & Smart Locations

## Problem Statement

**How might we** let TermCast users persist their preferences and quickly check weather in meaningful places, without breaking the zero-config first-run experience?

## Recommended Direction

Start with a TOML config file at `~/.config/termcast/config.toml` that supports default settings and named location entries. The config is purely optional -- TermCast works exactly as it does today without one. CLI flags always override config values.

The config file doubles as a "location book" with named entries, enabling a new `--at <name>` flag for quick access to saved places. This replaces the need for a separate location management subcommand -- users edit the TOML directly (the Unix way).

Ship in three iterations: (1) config file + defaults, (2) named locations + `--at` flag, (3) shell completions including location names.

## Key Assumptions to Validate

- [ ] Users will create and edit TOML files manually (test: ship it, see if anyone asks for a `config` subcommand)
- [ ] Named locations are more useful than free-form `--location` strings (test: does `--at home` feel better than `--location "San Francisco"`?)
- [ ] The config format won't need significant changes after v1 (test: keep it minimal, resist adding fields)
- [ ] Zero-config users won't feel like they're "missing out" (test: no warnings, no prompts to create config)

## MVP Scope -- Iteration 1: Config File

**In:**
- Read `~/.config/termcast/config.toml` (with XDG fallback)
- Config fields: `default_location` (string or "auto"), `units` ("auto", "celsius", "fahrenheit"), `cache_ttl` (integer)
- CLI flags override config values
- Graceful handling of missing/invalid config (log warning, continue with defaults)

**Example config:**
```toml
[defaults]
default_location = "auto"
units = "auto"
cache_ttl = 15
```

**Out:**
- Config creation via CLI command (users create files manually)
- Location book (that's iteration 2)
- Shell completions (iteration 3)
- Config migration/versioning

## MVP Scope -- Iteration 2: Smart Locations

**In:**
- `[locations.<name>]` sections in config with `city`, optional `latitude`/`longitude`
- `--at <name>` flag that resolves named location
- Tab-completable location names (prep for iteration 3)

**Example config:**
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
```

**Out:**
- Location add/remove subcommands
- Geocoding at save time
- Location auto-learning from usage

## MVP Scope -- Iteration 3: Shell Completions

**In:**
- Shell completion generation for bash, zsh, fish
- Completions include flag names AND saved location names
- `termcast completions <shell>` command to output completion script

**Out:**
- Dynamic completions (weather conditions, etc.)
- Completion installation automation

## Not Doing (and Why)

- **Offline/aggressive cache** -- didn't make the cut. Current cache is sufficient for now. Revisit if users report offline pain.
- **Config subcommands** (`termcast config set ...`) -- adds CLI surface complexity. TOML is human-readable; let users edit files. Add subcommands only if users ask.
- **Geocoding on save** -- adds API dependency and complexity. Store city names and let Open-Meteo resolve them (or store lat/lon for precision).
- **Config file auto-creation** -- don't litter filesystem on first run. Only create when user wants it.
- **Format/theming config** -- no one has asked for it. The current output is the "opinionated default."

## Open Questions

- Should `--at` also accept city names directly (not just saved aliases), making it a superset of `--location`?
- Should the config support multiple "profiles" (e.g., `[profiles.work]` with different defaults)?
- Should iteration 1 also add `--config <path>` for non-standard config locations?

## Dependency Chain

```
Iteration 1 (Config File)
    └── Iteration 2 (Smart Locations)
            └── Iteration 3 (Shell Completions)
```

Each iteration builds on the previous. No parallel work needed.
