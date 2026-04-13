# Locale-Aware Temperature Units

## Problem Statement
*How might we make temperature display respect user locale — US users see °F, everyone else sees °C — as originally specified in the SPEC?*

## Recommended Direction
Implement true locale-based temperature detection: the app determines the user's country from IP geolocation, and requests Fahrenheit for US locations, Celsius for everywhere else. The degree symbol in rendered output is dynamic.

**How it works:**
1. Add `country_code` to `GeoResponse` (ipapi.co returns ISO 2-letter code)
2. Thread `use_fahrenheit: bool` from geolocation → weather fetch → renderer
3. Open-Meteo API call gets `&temperature_unit=fahrenheit` or `&temperature_unit=celsius`
4. Renderer uses dynamic `°F` / `°C` symbol in all output lines
5. SPEC.md updated to document the behavior

## Key Assumptions to Validate
- [x] ipapi.co returns `country_code` reliably — verified, it's a required field
- [x] Open-Meteo accepts `temperature_unit` parameter — verified by API docs
- [x] Country code "US" is the right sentinel — verified, ipapi.co uses ISO 3166-1 alpha-2

## MVP Scope
**In:**
- `country_code` field added to `GeoResponse`
- `use_fahrenheit` detection based on `country_code == "US"`
- API call parameterized with `temperature_unit`
- Degree symbol (`°F` / `°C`) dynamic in all renderer output
- `WeatherDisplay` struct carries the chosen unit
- `SPEC.md` updated

**Not Doing:**
- `--unit` CLI flag to override — adds complexity not in scope
- Fahrenheit-only mode — locale detection is the goal
- Temperature conversion in code — let Open-Meteo return the right units
- Caching the unit — it can change if location changes, always fetch fresh

## Open Questions
- Should `country_code` be added to the cache entry? (No — cache stores weather data at the temperature it was fetched, so unit is baked in)
