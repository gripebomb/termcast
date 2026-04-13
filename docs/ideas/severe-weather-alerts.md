# Severe Weather Alerts (NWS)

## Problem Statement

How might we surface active severe weather warnings in termcast's minimal terminal display so US users see real NWS alerts without any configuration?

## Recommended Direction

Add NWS alert fetching as a conditional step in termcast's existing data pipeline. When the user's location is in the US (already detected via ipapi's `country_code`), make a second API call to `https://api.weather.gov/alerts/active?point={lat},{lon}`. Parse the response into a simple `Alert` type with event name, severity, and time range. Render as a single color-coded line below the existing 4-line weather display. In ambient mode, append a `⚠` indicator when active alerts exist.

The NWS API is free, requires no API key, and handles the spatial matching server-side via the `?point=` parameter -- no geometry library needed. For non-US users, nothing changes. Zero-config identity is preserved.

**Visual design:**
- Warning (tornado, severe thunderstorm, flash flood): bold red text
- Watch: yellow text
- Advisory: dimmed/dim yellow
- Format: `⚠ Tornado Warning until 6:00 PM`
- Ambient: `☀️ 14°C ⚠` (alert icon appended when active)

## Key Assumptions to Validate

- [ ] NWS API response time is acceptable (not slowing down every invocation by >500ms) -- test with real calls
- [ ] NWS API is reliable enough for a CLI tool (not behind inconsistent Cloudflare blocks) -- test over multiple days
- [ ] The `?point=` endpoint returns usable data for all US locations -- test with various lat/lon pairs
- [ ] Users want alerts by default rather than behind a flag -- ship default-on, add `--no-alerts` to opt out
- [ ] Single-line display is enough -- the full alert description can wait for a future `termcast alerts` subcommand

## MVP Scope

**In:**
- New `src/alerts.rs` module: NWS API client + `Alert` data type
- Severity-based color rendering in `renderer.rs`
- Conditional fetch: only when `country_code == "US"` (already available from ipapi)
- Compact alert line below weather display (normal mode)
- `⚠` indicator in ambient mode output
- `--no-alerts` CLI flag to disable
- Graceful failure: if NWS API is down, show weather without alerts (no error)

**Out:**
- Full alert descriptions (event name + time only for MVP)
- Multi-alert display (show highest severity alert only)
- Derived conditions for non-US users (explicitly deferred)
- `termcast alerts` subcommand (future enhancement)
- Alert history or notification/daemon mode

## Not Doing (and Why)

- **Derived conditions for non-US users** -- adds complexity and a trust tier that's more confusing than helpful. Non-US coverage requires per-region APIs with different data formats, maintenance burden that doesn't justify the value.
- **Full alert descriptions in the terminal** -- NWS descriptions can be multiple paragraphs. The 80-column format doesn't suit them. A future `termcast alerts` subcommand could show full details.
- **Alert notification/watch mode** -- termcast is a snapshot tool, not a daemon. Running it in a loop to watch for new alerts is a fundamentally different product.
- **Multiple simultaneous alerts** -- showing 5 alerts when a hurricane is approaching creates layout chaos. Show the most severe one, indicate count if needed.

## Open Questions

- Should the alert line be above or below the condition description line? (Currently assuming below -- feels like weather first, alerts second)
- Should `--forecast` also show alert indicators on days where alerts are active?
- What's the right UX when there are zero alerts vs when the NWS API call fails? Both should be invisible, but testing will confirm.
