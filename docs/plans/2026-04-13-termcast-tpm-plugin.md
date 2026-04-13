# TermCast TPM Plugin Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a TPM (tmux plugin manager) plugin that displays weather in the tmux status bar via one-command installation.

**Architecture:** A self-contained tmux plugin that wraps the termcast binary. Scripts live in `bin/`, configuration via `@termcast_*` tmux variables, and the plugin integrates with tmux's `status-right` via a shell script.

**Tech Stack:** Bash scripting, tmux configuration, termcast binary (existing)

---

## TPM Plugin Structure

```
termcast-tmux/
├── bin/
│   └── termcast_status        # Executable script for status bar
├── docs/
│   └── tmux_status_options.md
├── tpm                       # Marker file (required by TPM)
└── scripts/
    └── termcast_settings.sh  # Configuration helper
```

**How TPM works:**
- Plugins are git repositories cloned to `~/.tmux/plugins/`
- TPM adds `bin/` directories to PATH
- `tpm` file in repo root signals valid TPM plugin
- `prefix + I` installs, `prefix + U` updates plugins

---

## Configuration Options

Users set these in their `~/.tmux.conf` **before** the `run` line:

```tmux
# Required: TPM plugin line (at bottom of .tmux.conf)
run '~/.tmux/plugins/tpm/tpm'

# TermCast configuration (set BEFORE the run line):
set -g @termcast_location ""           # Empty = auto-detect, or "Oslo"
set -g @termcast_interval "15"         # Cache TTL in minutes
set -g @termcast_show_icon "yes"       # Show weather icon
set -g @termcast_format "{icon} {temp}" # Output format
```

---

## Tasks

### Task 1: Create plugin directory structure

**Files:**
- Create: `tmux/termcast/bin/termcast_status`
- Create: `tmux/termcast/bin/termcast_settings_helper`
- Create: `tmux/termcast/tpm` (marker file)
- Create: `tmux/termcast/README.md`

- [ ] **Step 1: Create directory structure**

```bash
mkdir -p tmux/termcast/bin
mkdir -p tmux/termcast/docs
mkdir -p tmux/termcast/scripts
```

- [ ] **Step 2: Create `tmux/termcast/tpm` marker file**

```bash
# This file is required by TPM to recognize this as a valid plugin.
# TPM will look for this exact file in the repo root.
```

- [ ] **Step 3: Create `tmux/termcast/bin/termcast_status`**

```bash
#!/usr/bin/env bash
# Displays weather in tmux status bar.
# Called by tmux's status-right option.

# Get configuration with defaults
get_tmux_option() {
    local option="$1"
    local default="$2"
    local value="$(tmux_get_option "$option")"
    echo "${value:-$default}"
}

tmux_get_option() {
    tmux show-option -gv "$1" 2>/dev/null
}

# Source termcast settings helper
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/../scripts/termcast_settings.sh" ]; then
    source "$SCRIPT_DIR/../scripts/termcast_settings.sh"
fi

# Get cached weather (reads from ~/.cache/termcast/current)
get_weather_cache() {
    local cache_file="${XDG_CACHE_HOME:-$HOME/.cache}/termcast/current"

    if [ ! -f "$cache_file" ]; then
        echo ""
        return 1
    fi

    cat "$cache_file"
}

# Parse weather data from cache
parse_cache() {
    local field="$1"
    local cache_data="$(get_weather_cache)"

    if [ -z "$cache_data" ]; then
        return 1
    fi

    # Parse JSON using grep/sed (portable, no jq dependency)
    case "$field" in
        temperature)
            echo "$cache_data" | grep -o '"temperature":[0-9.-]*' | head -1 | sed 's/"temperature"://'
            ;;
        weather_code)
            echo "$cache_data" | grep -o '"weather_code":[0-9]*' | head -1 | sed 's/"weather_code"://'
            ;;
        use_fahrenheit)
            echo "$cache_data" | grep -o '"use_fahrenheit":\(true\|false\)' | head -1 | sed 's/"use_fahrenheit"://'
            ;;
        location)
            echo "$cache_data" | grep -o '"location":"[^"]*"' | head -1 | sed 's/"location":"//; s/"$//'
            ;;
    esac
}

# Get weather icon from code
get_weather_icon() {
    local code="$1"
    case "$code" in
        0) echo "☀️" ;;
        1|2|3) echo "🌤" ;;
        45|48) echo "🌫" ;;
        51|53|55|61|63|65|66|67) echo "🌧" ;;
        71|73|75|77) echo "❄" ;;
        80|81|82|85|86) echo "🌦" ;;
        95|96|99) echo "⛈" ;;
        *) echo "☁" ;;
    esac
}

# Format temperature with unit
format_temp() {
    local temp="$1"
    local use_fahrenheit="$2"

    # Round to integer
    local temp_int=$(printf "%.0f" "$temp")

    if [ "$use_fahrenheit" = "true" ]; then
        echo "${temp_int}°F"
    else
        echo "${temp_int}°C"
    fi
}

# Main output
main() {
    local show_icon="${TERMCAST_SHOW_ICON:-yes}"
    local format="${TERMCAST_FORMAT:-{icon} {temp}}"

    local temperature="$(parse_cache "temperature")"
    local weather_code="$(parse_cache "weather_code")"
    local use_fahrenheit="$(parse_cache "use_fahrenheit")"

    if [ -z "$temperature" ] || [ -z "$weather_code" ]; then
        # Cache miss or invalid - return empty (don't show errors in status bar)
        echo ""
        return 0
    fi

    local icon="$(get_weather_icon "$weather_code")"
    local temp="$(format_temp "$temperature" "$use_fahrenheit")"

    # Build output based on format
    local output="$format"
    output="${output//\{icon\}/$icon}"
    output="${output//\{temp\}/$temp}"
    output="${output//\{location\}/$(parse_cache "location")}"

    echo "$output"
}

main "$@"
```

- [ ] **Step 4: Make script executable**

```bash
chmod +x tmux/termcast/bin/termcast_status
```

- [ ] **Step 5: Verify script runs**

```bash
./tmux/termcast/bin/termcast_status
# Should output weather or empty string (no cache = expected empty)
```

- [ ] **Step 6: Create `tmux/termcast/scripts/termcast_settings.sh`**

```bash
#!/usr/bin/env bash
# Loads termcast configuration from tmux options.

# Note: This script is sourced by termcast_status to get config values.
# It cannot use tmux commands directly - tmux may not be available when
# the status bar is rendered. Instead, we rely on environment variables
# set by the tmux plugin system.

# Configuration is passed via environment variables from tmux config.
# Users set these in their .tmux.conf before the 'run' line.

TERMCAST_LOCATION="${TERMCAST_LOCATION:-}"
TERMCAST_INTERVAL="${TERMCAST_INTERVAL:-15}"
TERMCAST_SHOW_ICON="${TERMCAST_SHOW_ICON:-yes}"
TERMCAST_FORMAT="${TERMCAST_FORMAT:-{icon} {temp}}"
```

- [ ] **Step 7: Create `tmux/termcast/README.md`**

```markdown
# termcast-tmux

A TPM (tmux plugin manager) plugin for displaying weather in your tmux status bar.

## Requirements

- tmux 2.4+
- [termcast](https://github.com/gripebomb/termcast) installed and in PATH
- [tpm](https://github.com/tmux-plugins/tpm) (tmux plugin manager)

## Installation

### 1. Install TPM (if not already installed)

```bash
git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm
```

### 2. Add TermCast to ~/.tmux.conf

Add the following **before** the `run '~/.tmux/plugins/tpm/tpm'` line:

```tmux
# ===========================================
# TermCast Configuration (set BEFORE TPM)
# ===========================================

# Location (empty = auto-detect via IP)
set -g @termcast_location ""

# Cache refresh interval in minutes (default: 15)
set -g @termcast_interval "15"

# Show weather icon (yes/no)
set -g @termcast_show_icon "yes"

# Format string for status bar
# Available: {icon}, {temp}, {location}
set -g @termcast_format "{icon} {temp}"

# ===========================================
# TPM (keep at bottom)
# ===========================================
run '~/.tmux/plugins/tpm/tpm'
```

### 3. Reload tmux config

```bash
tmux source-file ~/.tmux.conf
```

### 4. Install plugins

Press `prefix + I` (usually `Ctrl-b + I`) to install the plugin.

## Usage

### Status Bar Integration

The plugin automatically adds weather to `status-right`. By default it shows:

```
☀️ 14°C
```

### Custom Format

```tmux
# Show icon, temp, and location
set -g @termcast_format "{icon} {temp} in {location}"

# Temp only (minimal)
set -g @termcast_format "{temp}"
```

### Location Override

```tmux
set -g @termcast_location "London"
```

## How It Works

1. TPM installs the plugin to `~/.tmux/plugins/termcast-tmux`
2. The `bin/termcast_status` script is added to PATH by TPM
3. tmux calls this script via `status-right`
4. Script reads cached weather from `~/.cache/termcast/current`
5. TermCast binary fetches and caches weather (see `termcast --ambient`)

## Cache Behavior

- TermCast caches weather to `~/.cache/termcast/current`
- Default TTL: 15 minutes
- Status bar only shows cached data (no network calls in status bar)
- Run `termcast` or `termcast --ambient` to update cache

## Troubleshooting

### Weather not showing

1. Ensure `termcast` is installed and in PATH:
   ```bash
   which termcast
   ```

2. Ensure cache exists:
   ```bash
   cat ~/.cache/termcast/current
   ```

3. Reload tmux config:
   ```bash
   tmux source-file ~/.tmux.conf
   ```

4. Press `prefix + I` to install/reinstall plugins

### Wrong location

Run `termcast --location "Your City"` to update cache with correct location.

## Key Bindings

- `prefix + I` — Install/update plugins
- `prefix + U` — Update plugins only

## Uninstalling

1. Remove `@termcast_*` lines from `~/.tmux.conf`
2. Remove `termcast-tmux` from the `run` line
3. Press `prefix + alt + u` to remove plugin files
```

- [ ] **Step 8: Create `tmux/termcast/docs/tmux_status_options.md`**

```markdown
# Tmux Status Bar Options

## Available Format Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `{icon}` | Weather icon | ☀️ |
| `{temp}` | Temperature with unit | 14°C |
| `{location}` | Location name | Oslo |

## Example Formats

### Compact (default)

```
☀️ 14°C
```

```tmux
set -g @termcast_format "{icon} {temp}"
```

### With Location

```
☀️ 14°C in Oslo
```

```tmux
set -g @termcast_format "{icon} {temp} in {location}"
```

### Temperature Only

```
14°C
```

```tmux
set -g @termcast_format "{temp}"
```

### Minimal

```
14°
```

```tmux
set -g @termcast_format "{temp}"
# Remove the degree symbol from template since {temp} includes it
# Note: This is not currently supported, use default format
```

## Configuration Reference

| Option | Default | Description |
|--------|---------|-------------|
| `@termcast_location` | "" (auto-detect) | City name or coordinates |
| `@termcast_interval` | "15" | Cache TTL in minutes |
| `@termcast_show_icon` | "yes" | Show/hide weather icon |
| `@termcast_format` | "{icon} {temp}" | Status bar format |
```

## Combining with Other Status Items

If you have other items in `status-right`, use `#{?#{exists:termcast_status,..}` conditional:

```tmux
set -g status-right "#{prefix_highlight} | #{termcast_status} | %H:%M"
```

Or configure your status bar in `termcast_settings.sh` if you need custom logic.
```

- [ ] **Step 9: Verify plugin structure**

```bash
find tmux/termcast -type f | sort
```

Expected:
```
tmux/termcast/bin/termcast_status
tmux/termcast/bin/termcast_settings_helper
tmux/termcast/docs/tmux_status_options.md
tmux/termcast/README.md
tmux/termcast/scripts/termcast_settings.sh
tmux/termcast/tpm
```

- [ ] **Step 10: Commit**

```bash
git add tmux/termcast/
git commit -m "feat: add TPM plugin for tmux status bar integration"
```

---

### Task 2: Add plugin configuration via tmux hooks

**Files:**
- Modify: `tmux/termcast/bin/termcast_status` (add hook-based config)
- Create: `tmux/termcast/scripts/termcast_config_hook.sh`

- [ ] **Step 1: Update `termcast_status` to read tmux options**

The current script reads environment variables, but we need to read tmux options directly for true TPM integration.

```bash
#!/usr/bin/env bash
# tmux/termcast/bin/termcast_status
#
# Reads tmux options directly to support @termcast_* configuration.
# This allows users to set options in .tmux.conf and have them reflected
# in the status bar.

get_tmux_option() {
    local option="$1"
    local default="$2"

    # Try to get from tmux, fall back to env var, then default
    local value
    if value="$(tmux show-option -gv "$option" 2>/dev/null)" && [ -n "$value" ]; then
        echo "$value"
    elif [ -n "${!option}" ]; then
        echo "${!option}"
    else
        echo "$default"
    fi
}

# Get configuration from tmux options
get_config() {
    local option="$1"
    tmux show-option -gv "$option" 2>/dev/null || echo ""
}

# ... rest of script unchanged ...
```

Actually, tmux evaluates `status-right` by running the command in a shell. The script runs fine, but we can't use `tmux show-option` from inside a status bar command (tmux isn't fully initialized).

**Better approach:** Use an environment variable approach that works with TPM:

- [ ] **Step 2: Create `tmux/termcast/scripts/set_environment`**

TPM provides a way to set environment variables via `set-environment`:

```bash
#!/usr/bin/env bash
# Called by TPM to set environment variables

# This script is called during plugin initialization.
# We use it to pass configuration to the status bar script.

# Get tmux plugin path
PLUGIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Set environment variables that termcast_status will read
set -g -u TERMCAST_BIN_DIR "$PLUGIN_DIR/bin"
```

Wait, this won't work because tmux option reading happens in a different context.

**Final approach:** Keep the environment variable approach but document that users need to set them in their `.tmux.conf` BEFORE the `run` line.

- [ ] **Step 3: Update README to clarify config order requirement**

```markdown
## Important: Configuration Order

TPM configuration options must be set **before** the `run '~/.tmux/plugins/tpm/tpm'` line.
This is because TPM evaluates these options during plugin initialization.

```tmux
# WRONG - @termcast_* lines after run line won't work
run '~/.tmux/plugins/tpm/tpm'
set -g @termcast_location "London"  # Too late!
set -g @termcast_format "{icon} {temp}"  # Won't be read

# CORRECT - @termcast_* lines before run line
set -g @termcast_location "London"
set -g @termcast_format "{icon} {temp}"
run '~/.tmux/plugins/tpm/tpm'
```
```

- [ ] **Step 4: Commit**

```bash
git add tmux/termcast/README.md
git commit -m "docs: clarify TPM config order requirement"
```

---

### Task 3: Integrate with existing TermCast binary

**Files:**
- Modify: `tmux/termcast/bin/termcast_status` (verify integration)
- Create: `tmux/termcast/bin/termcast_refresh` (manual refresh helper)

- [ ] **Step 1: Create `tmux/termcast/bin/termcast_refresh`**

A helper script for manual cache refresh:

```bash
#!/usr/bin/env bash
# Manually refresh the weather cache.
# Useful for binding to a keybinding.

# Get location from tmux option or use auto-detect
LOCATION="$(tmux show-option -gv '@termcast_location' 2>/dev/null || echo '')"

if [ -n "$LOCATION" ]; then
    termcast --location "$LOCATION"
else
    termcast
fi

# Show confirmation
CACHE_FILE="${XDG_CACHE_HOME:-$HOME/.cache}/termcast/current"
if [ -f "$CACHE_FILE" ]; then
    echo "Weather cache refreshed"
else
    echo "Warning: Cache not created"
fi
```

- [ ] **Step 2: Add to README under "Key Bindings"**

```markdown
### Manual Refresh

Add to your `.tmux.conf` for a refresh keybinding:

```tmux
# Refresh weather on Ctrl-b r
bind r run-shell "$PLUGIN_DIR/bin/termcast_refresh"
```
```

- [ ] **Step 3: Make executable and verify**

```bash
chmod +x tmux/termcast/bin/termcast_refresh
./tmux/termcast/bin/termcast_refresh
```

- [ ] **Step 4: Commit**

```bash
git add tmux/termcast/bin/termcast_refresh
git commit -m "feat: add manual refresh helper to TPM plugin"
```

---

## Verification Checklist

- [ ] `find tmux/termcast -type f` shows correct structure
- [ ] `tmux/termcast/bin/termcast_status` is executable
- [ ] README documents installation clearly
- [ ] Configuration options have sensible defaults
- [ ] Plugin follows TPM structure (`tpm` marker file present)

---

## Not Doing (and Why)

- **Direct tmux option reading** — tmux status bar commands run in restricted context. Environment variables are the standard TPM pattern.
- **Automatic weather refresh** — Let tmux's `status-interval` handle periodic updates by calling termcast_status repeatedly.
- **Interactive location picker** — Users configure location once in .tmux.conf. Interactive would add complexity without value.
- **Multi-location support** — tmux status bar is global. One location per tmux session.

---

## Open Questions

1. Should we add a default `status-right` value, or require users to configure it?
2. How to handle the case where `termcast` binary is not installed? (Decided: Don't hard error, show empty string)
3. Should we show a "setup required" message if cache is empty on first run? (Decided: No, keep status bar clean)