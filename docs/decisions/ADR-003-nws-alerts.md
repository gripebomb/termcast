# ADR-003: NWS Severe Weather Alert Display with Silent Failure

## Status
Accepted

## Date
2026-04-13

## Context

TermCast fetches weather data from Open-Meteo. Users in the US may have active NWS severe weather alerts for their location. We need to display these alerts in the terminal output without adding latency to the weather display, and without disrupting the existing weather output if the alert API fails or returns unexpected data.

Key requirements:
- Display color-coded alert lines for US locations only
- **Zero latency impact** — alerts must not delay weather display
- **Graceful degradation** — alert failures never block weather
- Alert colors are fixed (not themeable via the existing theme system)
- No new crate dependencies

## Decision

### 1. NWS Alerts via `api.weather.gov/alerts/active`

Use the NWS API `https://api.weather.gov/alerts/active?point={lat},{lon}` for active alerts. It's free, requires no API key, and returns an empty `features` array for non-US locations — providing natural geographic filtering.

### 2. New module `src/alerts.rs` with display-ready types

Domain types live in a dedicated module, following the pattern of `weather.rs`, `forecast.rs`, and `geolocation.rs`. The `Alert` struct is display-ready: it contains the formatted `expires_time: Option<String>` (not raw ISO 8601), so rendering code has no formatting knowledge.

### 3. `AlertSeverity` enum with `Ord` derive

Severity ordering (`Advisory < Watch < Warning`) is defined via `Ord` derive on the enum, making it trivial to sort alerts by descending severity and pick the most severe for display. The spec requires showing the highest-severity alert only.

### 4. `Client::get_alerts(lat, lon)` on the existing HTTP client

The method follows the identical HTTP pattern as every other API method in `api.rs`:
URL construction → `User-Agent` header → send → status check → text → parse → return. Reuses the existing 10-second timeout. Returns `Result<Vec<Alert>, AppError>` — caller handles error suppression.

### 5. Error suppression at the integration layer

In `main.rs`, a `fetch_alerts` helper wraps `client.get_alerts()` and converts `Result<Vec<Alert>>` → `Option<Alert>`. Any network error, HTTP error, or parse error becomes `None`. This keeps the API client clean (it always returns meaningful errors) while the integration layer chooses to drop them silently.

### 6. Concurrent fetch via `tokio::join!`

Weather and alerts are fetched in a single `tokio::join!` call. The alerts future short-circuits to `Ok(vec![])` when `--no-alerts` is set, avoiding the network call entirely. This means the NWS API adds zero latency to the response time.

### 7. Fixed alert colors (not themeable)

Alert colors are hardcoded RGB values as specified:
- **Warning**: bold red `(255, 59, 48)`
- **Watch**: yellow `(255, 204, 0)`
- **Advisory**: dim yellow `(204, 170, 0)`

These are intentionally separate from the existing theme system (which handles weather display colors). The spec notes alert colors are "not themeable" — this keeps the implementation scope minimal and matches the existing behavior of `render_error` which uses raw `Color::Red`.

### 8. Ambient mode: warning indicator suffix

In ambient mode, alerts are indicated by appending a suffix to the single-line weather output:
- Warning severity: `⚠`
- Watch/Advisory: `⚡`

The spec says "Warning indicator" for ambient mode, but to differentiate severity without color in a single-line output, we use distinct indicators. Cache hits skip alert fetching entirely (stale weather data).

## Alternatives Considered

### Alerts on the Open-Meteo API
- **Pros:** Single weather API, already fetched
- **Cons:** Open-Meteo does not provide NWS-grade severe weather alerts
- **Rejected:** Open-Meteo has basic weather codes, not CAP alert data

### Cache alerts alongside weather
- **Pros:** Reduces NWS API load, faster ambient mode
- **Cons:** Stale alerts could show for hours; alerts have different TTL semantics than weather
- **Deferred:** Can be added in a future iteration. Current implementation re-fetches on every invocation.

### Alert colors via the existing theme system
- **Pros:** Consistent with weather display colors
- **Cons:** Adds complexity to the theme engine; the spec explicitly calls for fixed colors
- **Rejected:** Matches the spec requirement. Can be revisited if users request configurable alert colors.

### Return `Vec<Alert>` to the renderer and let it choose the most severe
- **Pros:** Renderer has full flexibility
- **Cons:** Renderer now needs alert domain logic (sorting, filtering)
- **Rejected:** `Client::get_alerts` returns alerts sorted by descending severity; main.rs picks the first. The renderer just renders what it receives.

### Sequential fetch (weather first, then alerts)
- **Pros:** Simpler to implement; alerts don't block weather
- **Cons:** Alerts add latency equal to the NWS round-trip time
- **Rejected:** Explicitly contradicts the spec requirement. Concurrent fetch is required.

## Consequences

- `src/alerts.rs` is a new domain module (follows existing pattern)
- Alert display is limited to current weather and ambient mode; `termcast forecast` does not show alerts
- `--no-alerts` flag allows users to opt out if they have slow networks or don't want alert data
- NWS API failures are completely silent — no stderr, no error messages, no logs
- Adding new NWS alert fields (e.g., `headline`, `instruction`) would require modifying `AlertProperties` and `Alert`
- The 2 pre-existing test failures (`test_config_path_fallback_to_home`, `test_cache_path_fallback_to_home`) in WSL environments are unrelated to this feature and predate it