# TermCast Neovim Plugin Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a Neovim Lua plugin that displays weather in the status line, integrating with popular status line plugins (lualine, airline, lightline).

**Architecture:** A Lua plugin that wraps the Rust binary, exposes Vim commands, and provides a status line component. The plugin reads from the same cache file as the Rust binary, ensuring consistent weather data across all integrations.

**Tech Stack:** Lua (Neovim plugin), Rust (existing binary), vim-villa pattern for module structure

---

## File Structure

```
lua/
├── termcast/
│   ├── init.lua           # Plugin entry point, user commands
│   ├── config.lua         # Configuration handling
│   ├── cache.lua          # Cache file reader (Lua re-implementation)
│   ├── ui.lua             # Status line component generators
│   └── health.lua         # :checkhealth support
```

**Existing files used:**
- `src/cache.rs` — Provides `~/.cache/termcast/current` in JSON format
- `src/weather.rs` — Weather icon/description mapping
- `src/lib.rs` — Exposes cache module via library

**Note:** We'll read the cache file directly from Lua using `vim.fn.json_decode` to avoid requiring the user to have a specific Rust version installed. This also keeps the plugin fast (no subprocess overhead for cached reads).

---

## Dependencies & Prerequisites

1. **termcast binary in PATH** — Users install via `cargo install termcast` or from releases
2. **Nvim 0.7+** — Required for `vim.fn.json_decode` and `health#check` (built into Neovim 0.7+)
3. **Optional status line plugins** — Plugin works standalone but integrates with lualine, airline, lightline

---

## Configuration Options

```lua
-- Default configuration
require('termcast').setup({
    -- Update interval in seconds (default: 900 = 15 minutes)
    refresh_interval = 900,

    -- Cache file location (default: auto-detect via stdpath('cache'))
    cache_file = nil,

    -- Location override (nil = auto-detect via IP, string = manual location)
    location = nil,

    -- Unit preference: 'auto' | 'celsius' | 'fahrenheit'
    unit = 'auto',

    -- Status line format string
    -- Available placeholders: {icon}, {temp}, {location}, {condition}
    format = '{icon} {temp}°',

    -- Show location in status line (default: false — too verbose)
    show_location = false,

    -- Error message when weather unavailable
    error_text = 'weather?',
})
```

---

## Tasks

### Task 1: Plugin Structure & Health Check

**Files:**
- Create: `lua/termcast/init.lua`
- Create: `lua/termcast/health.lua`
- Create: `README.md` (plugin docs)
- Test: `tests/termcast/health_spec.lua` (if test framework used)

- [ ] **Step 1: Create plugin directory structure**

```bash
mkdir -p lua/termcast
mkdir -p tests
```

- [ ] **Step 2: Create `lua/termcast/init.lua`**

```lua
---@diagnostic disable: undefined-global
local M = {}

M.config = {
    refresh_interval = 900,
    cache_file = nil,
    location = nil,
    unit = 'auto',
    format = '{icon} {temp}°',
    show_location = false,
    error_text = 'weather?',
}

M._cache = nil
M._cache_time = 0

--- Setup function called by user
---@param opts table|nil
function M.setup(opts)
    M.config = vim.tbl_deep_extend('force', M.config, opts or {})

    -- Register :TermCast command
    vim.api.nvim_create_user_command('TermCast', function(args)
        if args.args == 'refresh' then
            M._cache = nil
        end
        print(M.get_statusline())
    end, { nargs = '?' })

    -- Register :TermCastLocation command
    vim.api.nvim_create_user_command('TermCastLocation', function(args)
        if args.args == '' then
            vim.notify('Current location: ' .. (M.get_location() or 'auto-detect'))
        else
            M.config.location = args.args
            M._cache = nil -- Force refresh
            vim.notify('Location set to: ' .. args.args)
        end
    end, { nargs = '?' })
end

--- Get the current weather data (cached)
---@return table|nil
function M.get_weather()
    local now = os.time()
    local cache_age = now - M._cache_time

    -- Refresh if cache is old or empty
    if M._cache == nil or cache_age > M.config.refresh_interval then
        M._cache = M._read_cache()
        M._cache_time = now
    end

    return M._cache
end

--- Read the cache file directly (Lua implementation)
---@return table|nil
function M._read_cache()
    local cache_path = M.config.cache_file or M._default_cache_path()

    local fd = io.open(cache_path, 'r')
    if not fd then
        return nil
    end

    local content = fd:read('*all')
    fd:close()

    local ok, data = pcall(vim.fn.json_decode, content)
    if not ok then
        return nil
    end

    return data
end

--- Get the default cache path
---@return string
function M._default_cache_path()
    local cache_home = os.getenv('XDG_CACHE_HOME')
    if not cache_home or cache_home == '' then
        local home = os.getenv('HOME') or ''
        cache_home = home .. '/.cache'
    end
    return cache_home .. '/termcast/current'
end

--- Get current location from cache or config
---@return string|nil
function M.get_location()
    local weather = M.get_weather()
    if weather and weather.location then
        return weather.location
    end
    return M.config.location
end

--- Get status line text
---@return string
function M.get_statusline()
    local weather = M.get_weather()

    if not weather then
        return M.config.error_text
    end

    local icon = M._weather_icon(weather.weather_code or 0)
    local temp = math.floor(weather.temperature + 0.5)
    local location = M.config.show_location and weather.location or nil

    local result = M.config.format
    result = result:gsub('{icon}', icon)
    result = result:gsub('{temp}', tostring(temp))

    -- Handle unit conversion
    if weather.use_fahrenheit then
        result = result:gsub('°', '°F')
    else
        result = result:gsub('°', '°C')
    end

    if location then
        result = result .. ' ' .. location
    end

    return result
end

--- Weather code to icon mapping
---@param code number
---@return string
function M._weather_icon(code)
    local icons = {
        [0] = '☀️',  [1] = '🌤',  [2] = '🌤',  [3] = '🌤',
        [45] = '🌫', [48] = '🌫',
        [51] = '🌧', [53] = '🌧', [55] = '🌧',
        [61] = '🌧', [63] = '🌧', [65] = '🌧',
        [66] = '🌧', [67] = '🌧',
        [71] = '❄',  [73] = '❄',  [75] = '❄',  [77] = '❄',
        [80] = '🌦', [81] = '🌦', [82] = '🌦',
        [85] = '🌦', [86] = '🌦',
        [95] = '⛈',  [96] = '⛈',  [99] = '⛈',
    }
    return icons[code] or '☁'
end

--- Register with lualine
---@return table
function M.lualine_component()
    return {
        provider = function()
            return M.get_statusline()
        end,
        hl = {
            fg = '#22d3ee',
        },
    }
end

return M
```

- [ ] **Step 3: Run to verify syntax**

Run: `nvim --headless -c "luafile lua/termcast/init.lua" -c "qa!"`
Expected: No errors

- [ ] **Step 4: Create `lua/termcast/health.lua`**

```lua
local M = {}

function M.check()
    local health = require('health')

    health.start('termcast')

    -- Check if termcast binary is available
    local bin = vim.fn.executable('termcast')
    if bin == 1 then
        health.ok('termcast binary found')
    else
        health.warn('termcast binary not found in PATH')
        health.info('Install with: cargo install termcast')
    end

    -- Check cache file
    local cache_path = os.getenv('XDG_CACHE_HOME') or (os.getenv('HOME') .. '/.cache')
    cache_path = cache_path .. '/termcast/current'

    if vim.fn.filereadable(cache_path) == 1 then
        health.ok('Cache file found at ' .. cache_path)
    else
        health.warn('Cache file not found at ' .. cache_path)
        health.info('Run `termcast` to generate initial cache')
    end

    -- Check Neovim version
    local version = vim.version()
    if version and version.major >= 0 and version.minor >= 7 then
        health.ok('Neovim version ' .. version.major .. '.' .. version.minor .. ' supports required features')
    else
        health.warn('Neovim 0.7+ required for JSON parsing')
    end
end

return M
```

- [ ] **Step 5: Create `plugin/termcast.lua` (auto-load)**

```lua
-- Auto-load the plugin
vim.api.nvim_exec([[
    lua require('termcast')
]], false)
```

- [ ] **Step 6: Create plugin README**

```markdown
# termcast.nvim

A Neovim plugin that displays weather in your status line, powered by the TermCast Rust binary.

## Requirements

- Neovim 0.7+
- [termcast](https://github.com/gripebomb/termcast) installed and in PATH

## Installation

### vim-plug

```vim
Plug 'gripebomb/termcast', { 'rtp': 'lua' }
```

### Packer

```lua
use 'gripebomb/termcast'
```

### lazy.nvim

```lua
{ 'gripebomb/termcast', lazy = false }
```

## Setup

```lua
require('termcast').setup({
    refresh_interval = 900,  -- 15 minutes
    show_location = false,
})
```

## Usage

### Commands

- `:TermCast` — Show weather in command line
- `:TermCastLocation` — Show/set location
- `:TermCastLocation London` — Set location to London

### Lualine Integration

```lua
require('lualine').setup({
    sections = {
        lualine_c = {
            { require('termcast').get_statusline }
        },
    },
})
```

## Status Line Format

Default format: `{icon} {temp}°`

Available placeholders:
- `{icon}` — Weather icon
- `{temp}` — Temperature
- `{location}` — Location name

## Checkhealth

Run `:checkhealth termcast` to verify your setup.
```

- [ ] **Step 7: Commit**

```bash
git add lua/termcast/ plugin/termcast.lua README.md
git commit -m "feat: add neovim plugin structure with health check"
```

---

### Task 2: Lualine Integration

**Files:**
- Modify: `lua/termcast/init.lua` (add lualine provider)
- Create: `tests/lualine_spec.lua` (manual test)
- Update: `README.md`

- [ ] **Step 1: Verify lualine component works**

Create test file `test_lualine.lua`:
```lua
-- Manual test
package.path = package.path .. ';./lua/?.lua;./lua/?/init.lua'
local termcast = require('termcast')

-- Test setup
termcast.setup({
    show_location = false,
    format = '{icon} {temp}°',
})

-- Test get_statusline returns a string
local status = termcast.get_statusline()
print('Status line: ' .. status)
assert(type(status) == 'string', 'get_statusline should return string')
```

Run: `nvim --headless -c "luafile test_lualine.lua" -c "qa!"`
Expected: Prints status line

- [ ] **Step 2: Verify lualine component structure**

```lua
-- test_lualine_component.lua
package.path = package.path .. ';./lua/?.lua;./lua/?/init.lua'
local termcast = require('termcast')

local component = termcast.lualine_component()
print('Provider type: ' .. type(component.provider))
print('HL set: ' .. (component.hl and 'yes' or 'no'))
assert(type(component.provider) == 'function', 'lualine_component should return function provider')
```

Run: `nvim --headless -c "luafile test_lualine_component.lua" -c "qa!"`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add lua/termcast/init.lua
git commit -m "feat: add lualine integration component"
```

---

### Task 3: Documentation & Distribution

**Files:**
- Update: `README.md` (full documentation)
- Create: `lua/termcast/VERSION` (version file synced with Rust binary)

- [ ] **Step 1: Update README with complete docs**

Document:
- Installation (all plugin managers)
- Configuration options with defaults
- Lualine/Airline/Lightline integration examples
- Commands reference
- Troubleshooting

- [ ] **Step 2: Create version file**

```bash
echo "0.1.0" > lua/termcast/VERSION
```

- [ ] **Step 3: Test README syntax**

Run: Check markdown renders correctly
Expected: Valid markdown

- [ ] **Step 4: Commit**

```bash
git add README.md lua/termcast/VERSION
git commit -m "docs: complete neovim plugin documentation"
```

---

## Verification Checklist

- [ ] `nvim --headless` loads the plugin without errors
- [ ] `:TermCast` command prints weather status
- [ ] `:checkhealth termcast` shows health report
- [ ] Lualine component returns correct format
- [ ] README includes all installation options

---

## Not Doing (and Why)

- **Airline/Lightline integration in v1** — Focus on lualine first (most popular). Add others after.
- **Subprocess invocation** — Read cache directly for speed. Subprocess adds 10-50ms latency.
- **Custom weather icons** — User theme should control status line colors, not icon shapes.
- **Interactive location picker** — CLI `--location` flag handles this. Neovim plugin is display-only.