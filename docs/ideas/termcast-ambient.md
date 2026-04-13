# Ambient TermCast

## Problem Statement

**How might we give terminal enthusiasts persistent weather awareness — so current conditions are visible at a glance without any conscious action — without adding system complexity?**

## Recommended Direction

**The shell prompt IS the daemon.** No background process. Instead, a `termcast_prompt()` shell function called on every prompt render via `$PROMPT_COMMAND` / precmd hook. It checks `~/.cache/termcast/current` — if the cache is fresh (≤15 min old), it reads and prints in <1ms. If stale, it fetches from Open-Meteo, writes the cache, then prints.

This means:
- **Weather is always visible** — every new shell prompt shows current conditions
- **tmux status bar works too** — `termcast --ambient` reads the same cache, outputs `☀️ 14°` for `status-right`
- **Zero daemon lifecycle** — no startup, no crash recovery, no init scripts
- **Same API calls as daemon** — fetches only when cache is stale, not on every prompt

The tmux bar and shell prompt share the same cached state. Both surfaces update roughly every 15 minutes without coordinating — they just read the cache timestamp.

**Ambient output format:** `☀️ 14°` — weather icon + temperature, fits in any status bar or prompt segment.

## Key Assumptions to Validate

- [ ] **Terminal enthusiasts will integrate this into their setup** — install script handles tmux/prompt config automatically, not manual
- [ ] **15-minute TTL is the right cadence** — too frequent = API churn; too infrequent = stale data on cold mornings
- [ ] **The 6-10 char format is enough context** — no feels-like, no condition text. Validate by asking 5 people if that's sufficient
- [ ] **Cache misses (first run, expired) are fast enough** — network fetch shouldn't noticeably slow the prompt
- [ ] **Open-Meteo free tier handles this fetch rate** — ~4 fetches/hour when terminals are open. Likely fine, but worth a quick rate-limit check

## MVP Scope

**In:**
- `termcast --cache-ttl <minutes>` flag (default 15)
- `termcast --ambient` subcommand — reads cache, outputs `☀️ 14°` or `termcast: cache empty` if no cache
- Shell integration script (bash/zsh functions + `$PROMPT_COMMAND` setup)
- tmux integration snippet (adds to `status-right`)
- Auto-cache on any termcast invocation (existing command also populates cache)
- `~/.cache/termcast/` for cache storage (XDG-compliant)

**Out:**
- Background daemon / systemd service
- Multiple cached locations (single user, single location)
- Weather alerts / push notifications
- Config file / configuration system
- Rain prediction / change alerts (future feature)
- Prompt character-by-character animation

## Not Doing (and Why)

- **Background daemon** — adds init system complexity, crash recovery, and a process to manage. The prompt hook delivers the same user outcome (ambient awareness) at 1/10th the complexity. Earn the daemon when the simpler approach proves insufficient.
- **Feels-like temperature / condition text in the ambient output** — 6-10 chars is tight. Keep it minimal. Full TermCast output still available via `termcast` command.
- **Change alerts ("Rain in 30min")** — interesting, but different product. Validate the ambient display thesis first before adding event-driven behavior.
- **Multi-location support** — ambient is inherently local. One user, one location, one ambient display.
- **Persistent notification daemon for when terminals are closed** — out of scope. If terminals are closed, ambient awareness isn't the right UX anyway.

## Open Questions

- What does the install/setup UX look like? Single script that detects shell and tmux, adds the integration? Or manual config snippet?
- Should `termcast --ambient` output different formats for tmux vs. shell prompt (tmux might want `[☀️ 14°]` with brackets, shell prompt might want just `☀️ 14° `)?
- How to handle the initial "empty cache" state — should `--ambient` fetch immediately if cache is empty, or show nothing?
- Should the ambient display adapt color (e.g., temperature-based coloring in tmux)? That requires tmux to evaluate the value — tmux status bars are static strings unless you use `status-interval` + a script. Worth exploring for v2.
