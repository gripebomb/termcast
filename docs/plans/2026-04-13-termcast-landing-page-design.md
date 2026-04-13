# TermCast Landing Page — Design

## Overview

An immersive single-file HTML landing page showcasing TermCast product info and visual output demonstrations. Theme: weather-based with terminal aesthetics.

## Visual Identity

- **Color palette**:
  - Background: Deep charcoal `#0d0d0d`
  - Surface: Dark gray `#1a1a1a`
  - Primary accent: Cyan `#22d3ee`
  - Secondary accent: Amber `#fbbf24`
  - Success accent: Green `#34d399`
  - Text: Off-white `#f5f5f5`
  - Muted text: Gray `#888888`
- **Typography**:
  - Body: Inter (Google Fonts)
  - Code/Terminal: JetBrains Mono (Google Fonts)
- **Style**: Sophisticated dark theme, subtle gradients, soft glows, smooth animations

## Sections

### 1. Hero
- **Headline**: "Weather worth opening your terminal for"
- **Sub-headline**: "Beautiful ANSI weather output. Zero config. Works out of the box."
- **Terminal demo**: Floating window showing TermCast output
- **CTA button**: `cargo install termcast` with copy functionality

### 2. Features (3-column grid)
| Feature | Description |
|---------|-------------|
| Beautiful Output | ANSI-colored weather with icons that render perfectly in iTerm2, Terminal, and kitty |
| Zero Config | Auto-detects your location via IP. Works on first run. |
| Ambient Mode | Compact single-line output perfect for shell prompts and tmux status bars |

### 3. How It Works (3-step visual)
1. Install: `cargo install termcast`
2. Run: `./termcast`
3. Enjoy: Beautiful weather in your terminal

### 4. Demo Section
- **Preset cards** (4 locations):
  - Oslo, Norway — Clear, 14°C
  - Miami, USA — Sunny, 28°C
  - Reykjavik, Iceland — Snow, -2°C
  - Tokyo, Japan — Rainy, 18°C
- Each card displays mini terminal output with location-specific weather
- **Interactive demo**: Input field for city name → fetches live weather via Open-Meteo API → displays ambient mode output `☀️ 14°`
- API call includes geocoding (city → lat/lon) then weather fetch

### 5. Shell Integration
- **Tabbed interface**: Bash | Zsh | tmux
- Syntax-highlighted code blocks with terminal color scheme
- Copy-to-clipboard button on each snippet

### 6. Footer
- GitHub link
- MIT license badge
- "Built with Rust" badge

## Interactions

- Smooth scroll navigation
- Floating animation on hero terminal window (subtle up/down)
- Hover lift effect on feature cards and demo cards
- Copy-to-clipboard with visual feedback on CTA and code snippets
- Tab switching on shell integration section
- Form submission on demo input (fetch live weather)

## Technical Implementation

- **Single HTML file** with embedded CSS and JS
- **No external images** — all graphics via CSS/SVG/emoji
- **External resources**: Google Fonts only (Inter, JetBrains Mono)
- **Live weather API**: Open-Meteo (free, no key required)
  - Geocoding: `https://geocoding-api.open-meteo.com/v1/search`
  - Weather: `https://api.open-meteo.com/v1/forecast`
- **Responsive**: Desktop-first, works on mobile (stacked layouts)

## Component Details

### Terminal Window
- Title bar with traffic lights (macOS style: red/yellow/green circles)
- Window title: "termcast — zsh"
- Content: ANSI-colored TermCast output
- Box shadow for depth
- Subtle floating animation

### Weather Cards
- Dark surface background
- Location name + country
- Weather icon (emoji) + temperature
- Condition text
- Subtle border glow on hover

### Code Blocks
- Dark background with syntax highlighting
- Language label
- Copy button (top-right corner)
- JetBrains Mono font

## Weather Icons (Emoji)

| Code | Icon | Description |
|------|------|-------------|
| 0 | ☀️ | Clear |
| 1-3 | 🌤 | Partly Cloudy |
| 45-48 | 🌫 | Fog |
| 51-67 | 🌧 | Rain/Drizzle |
| 71-77 | ❄ | Snow |
| 80-82 | 🌦 | Showers |
| 95-99 | ⛈ | Thunderstorm |
| default | ☁ | Cloudy |

## Temperature Display

- US locale → Fahrenheit (°F)
- All other locales → Celsius (°C)
- Detect via browser `navigator.language`

## File Structure

```
website/
└── index.html    # Single file with embedded CSS and JS
```
