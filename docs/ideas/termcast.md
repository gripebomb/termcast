# TermCast — Beautiful Terminal Weather

## Problem Statement

How might we give terminal enthusiasts a weather tool so visually compelling it becomes a permanent fixture in their setup — something they show off, not just use?

## Recommended Direction

Build a weather tool where **beauty is the primary value proposition**, not an afterthought. Not a utilitarian CLI that happens to have colors — a tool designed from the ground up to look stunning in a terminal.

The thesis: for this audience, aesthetic quality *is* the product. Terminal enthusiasts are already optimizing their environment for visual coherence (vim theme, prompt, tmux). A beautiful weather display isn't a bonus — it's the adoption driver. They put it in their screenshots, share their configs, and it becomes part of their identity.

**Visual direction:** CMAN/vim-airline energy. Think carefully chosen Unicode art, subtle gradients via color codes, precise typography. Not ASCII art from 1995 — modern terminal craft. The tool should look like it belongs next to a well-tuned neovim config.

**MVP output (3-4 lines):**
```
     ☁ 14°C Oslo
   Feels 11°
   High 17° · Low 8°
   Clear until evening
```

**MVP also includes:** current conditions, temperature (actual + feels like), today's high/low, one-line forecast, weather condition icon. That's it. No hourly breakdown, no 7-day forecast, no alerts — just the essential 4 facts rendered beautifully.

**Data:** Open-Meteo (no API key required) + ipapi.co for location (no key required). Zero-config on first run.

**Tech:** Rust with crossterm for rendering. Clean, fast, minimal dependencies.

## Key Assumptions to Validate

- [ ] **Aesthetic quality drives adoption for this audience** — test by showing mockups to 5 terminal enthusiasts, measuring whether they say "I'd install this" vs. "cool"
- [ ] **The essential 4-fact output is sufficient** — test by asking 5 people what they actually check weather for (vs. what they think they need)
- [ ] **Open-Meteo + IP geolocation accuracy is good enough** — validate by comparing results against known location for 10 users
- [ ] **Rust/crossterm can produce output that looks good across terminal emulators** — test early on iTerm2, kitty, alacritty, macOS Terminal

## MVP Scope

**In:**
- Current temperature (actual + feels-like)
- Weather condition icon (Unicode)
- Today's high/low
- One-line text forecast
- Auto-detect location via IP geolocation
- No config required on first run
- One binary, macOS + Linux

**Out:**
- Background daemon (just run-it-and-see output for v1)
- Forecast beyond today
- Hourly breakdown
- Alerts
- Multiple locations
- Custom colors/fonts
- tmux/prompt integration

## Not Doing (and Why)

- **Background daemon** — adds complexity and risk. Nail the beautiful one-shot output first. Daemon + alerts is v2.
- **7-day forecast / hourly breakdown** — we're betting that 4 facts is the right scope. If people want more, we'll add it. But start small.
- **API keys** — Open-Meteo + ipapi.co are free, no-auth-required. This is a deliberate choice to remove friction. Don't add auth back in v1.
- **Customization** — no config file, no flags. It just works and looks good. Customization can come when there's something worth customizing.
- **Alerts** — valuable, but orthogonal to the beauty thesis. That's a second product. Get the first one right.

## Open Questions

- What does the "show off" moment look like? When someone shares their terminal, what do they want to show? The output needs to be screenshot-worthy.
- What's the installation story? `cargo install`? Homebrew? Binary download? For this audience, source + `cargo install` might be the aesthetic--correct choice.
- Should output go to stdout (so it works in pipes/chains) or render directly to terminal (for colored output)? Likely stdout with ANSI color codes for piping compatibility.
- Does the user want location auto-detected, or should they configure it? Auto-detect first, config override later.

## Name Consideration

"TermCast" feels right — terminal + forecast, hints at both weather and broadcasting. Open to alternatives (weatherman, termweath, cast, wthr, ...).