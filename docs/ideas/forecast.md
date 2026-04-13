# TermCast Forecast

## Problem Statement

How might we give TermCast users a fast, beautiful forecast experience in their terminal so they can plan ahead without reaching for their phone or browser?

## Recommended Direction

Add a `termcast forecast` subcommand that renders a clean 5-day forecast table using Open-Meteo data — the same free, no-API-key source already in use. The table shows day, weather icon, high/low temperatures, and precipitation probability. An `--hourly` flag provides a compact hour-by-hour view for today, showing only notable changes rather than all 24 hours. Ambient mode gets a compact forecast line showing tomorrow's high/low and the next precipitation event.

This direction works because TermCast already fetches daily data from Open-Meteo and has the rendering infrastructure. We're extending what exists, not building new plumbing. The visual quality — Unicode icons, ANSI colors, clean alignment — is what separates this from `wttr.in` and other text-based forecast tools.

## Key Assumptions to Validate

- [ ] Open-Meteo hourly precip probability is accurate enough to be useful — compare against a known-good source for your location
- [ ] The clap setup supports subcommands cleanly — verify the current argument parser can accommodate `termcast forecast` alongside existing flags
- [ ] The hourly "notable changes only" filter (precip start/stop, temp shifts > 5°) produces a useful summary — test with real data to confirm it doesn't hide important information

## MVP Scope

**In:**
- `termcast forecast` — 5-day table: day name, icon, high/low, precip chance
- `termcast forecast --hourly` — compact hourly view for today (notable changes only)
- `termcast forecast --days N` — configurable forecast length (1-7)
- Ambient forecast mode — tomorrow's high/low + next precip event in one line
- Precipitation probability in daily table
- Open-Meteo hourly + daily API parameters added to existing fetch

**Out (for MVP):**
- Minute-by-minute precipitation (not available on Open-Meteo)
- Wind speed/direction in forecast table
- UV index, humidity, or other secondary variables
- Historical weather data
- Forecast alerts/notifications (`termcast watch`)
- Configurable forecast defaults in config file

## Not Doing (and Why)

- **Minute-by-minute precipitation** — Open-Meteo doesn't offer it, and adding OWM as a second source breaks the zero-config philosophy
- **`termcast watch` / background alerts** — This is a fundamentally different product (daemon vs. CLI). Out of scope for this feature
- **Wind, UV, humidity in the table** — Scope creep. The table should be scannable; adding more columns makes it harder to read. Can be added later via flags if users ask
- **Config file forecast defaults** — The `--days` flag is enough for MVP. Config defaults are a convenience, not a necessity
- **7-day default** — Forecasts beyond 5 days are unreliable. Users can opt in with `--days 7` if they want the noise

## Open Questions

- Should `termcast forecast` with no location fall back to auto-detected IP location (like current behavior), or should it require an explicit location?
- Should the hourly view cover today only, or should `--hourly` accept a date like `--hourly tomorrow`?
- What's the right threshold for "notable" temperature changes in the hourly filter — is 5° too aggressive or too lenient?
