# TermCast Landing Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create an immersive single-file HTML landing page showcasing TermCast with visual weather demos and live API integration.

**Architecture:** Single HTML file with embedded CSS and JS. Dark theme with terminal aesthetics. Google Fonts for typography. Open-Meteo API for live weather data.

**Tech Stack:** HTML5, CSS3 (custom properties, flexbox, grid), Vanilla JavaScript, Google Fonts (Inter, JetBrains Mono), Open-Meteo API (free, no key)

---

## File Structure

```
website/
└── index.html    # Single file containing all CSS and JS
```

---

## Task 1: HTML Structure with All Sections

**Files:**
- Create: `website/index.html`

- [ ] **Step 1: Create HTML file with complete structure**

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TermCast — Weather worth opening your terminal for</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <style>
        /* CSS will be added here */
    </style>
</head>
<body>
    <!-- Navigation -->
    <nav>...</nav>

    <!-- Hero Section -->
    <section id="hero">...</section>

    <!-- Features Section -->
    <section id="features">...</section>

    <!-- How It Works Section -->
    <section id="how-it-works">...</section>

    <!-- Demo Section -->
    <section id="demo">...</section>

    <!-- Shell Integration Section -->
    <section id="integration">...</section>

    <!-- Footer -->
    <footer>...</footer>

    <script>
        /* JavaScript will be added here */
    </script>
</body>
</html>
```

- [ ] **Step 2: Create complete CSS with all styling**

Include:
- CSS custom properties for color palette
- Base styles (reset, typography)
- Section-specific styles
- Animations and transitions
- Responsive breakpoints

- [ ] **Step 3: Create complete JavaScript functionality**

Include:
- Smooth scroll navigation
- Tab switching for shell integration
- Copy-to-clipboard functionality
- Live weather API integration
- Weather icon mapping

- [ ] **Step 4: Commit**

```bash
git add website/index.html
git commit -m "feat: create initial landing page structure"
```

---

## Task 2: Hero Section with Terminal Demo

**Files:**
- Modify: `website/index.html`

- [ ] **Step 1: Add hero section with terminal window**

```html
<section id="hero" class="hero">
    <div class="hero-content">
        <h1>Weather worth opening your terminal for</h1>
        <p class="subtitle">Beautiful ANSI weather output. Zero config. Works out of the box.</p>
        <div class="cta-group">
            <button class="btn-primary" id="install-btn">
                <code>cargo install termcast</code>
                <span class="copy-hint">Click to copy</span>
            </button>
        </div>
    </div>
    <div class="terminal-window">
        <div class="terminal-titlebar">
            <div class="traffic-lights">
                <span class="light red"></span>
                <span class="light yellow"></span>
                <span class="light green"></span>
            </div>
            <span class="terminal-title">termcast — zsh</span>
        </div>
        <div class="terminal-content">
            <pre class="weather-output">
     <span class="icon">☀️</span> <span class="temp">14°C</span> <span class="location">Oslo</span>
   <span class="feels">Feels 11°</span>
   <span class="range">High 17° · Low 8°</span>
   <span class="condition">Clear</span>
            </pre>
        </div>
    </div>
</section>
```

- [ ] **Step 2: Add hero CSS**

```css
.hero {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4rem 8%;
    gap: 4rem;
}

.hero-content {
    flex: 1;
    max-width: 600px;
}

.hero h1 {
    font-size: 3.5rem;
    font-weight: 700;
    line-height: 1.1;
    margin-bottom: 1.5rem;
    background: linear-gradient(135deg, var(--cyan), var(--amber));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
}

.subtitle {
    font-size: 1.25rem;
    color: var(--muted);
    margin-bottom: 2rem;
}

.terminal-window {
    flex: 1;
    max-width: 500px;
    background: var(--surface);
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
    animation: float 6s ease-in-out infinite;
}

@keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
}

.terminal-titlebar {
    background: var(--bg);
    padding: 12px 16px;
    display: flex;
    align-items: center;
    gap: 12px;
}

.traffic-lights {
    display: flex;
    gap: 8px;
}

.light {
    width: 12px;
    height: 12px;
    border-radius: 50%;
}

.light.red { background: #ff5f57; }
.light.yellow { background: #febc2e; }
.light.green { background: #28c840; }

.terminal-title {
    font-family: var(--font-mono);
    font-size: 0.875rem;
    color: var(--muted);
}

.terminal-content {
    padding: 2rem;
    background: var(--surface);
}

.weather-output {
    font-family: var(--font-mono);
    font-size: 1.1rem;
    line-height: 1.6;
    color: var(--text);
}

.weather-output .icon { color: var(--amber); }
.weather-output .temp { color: var(--cyan); font-weight: 500; }
.weather-output .location { color: var(--green); }
.weather-output .feels { color: var(--muted); display: block; margin-left: 1rem; }
.weather-output .range { color: var(--muted); display: block; margin-left: 1rem; }
.weather-output .condition { color: var(--text); display: block; margin-left: 1rem; }
```

- [ ] **Step 3: Test in browser**

- [ ] **Step 4: Commit**

```bash
git add website/index.html
git commit -m "feat: add hero section with terminal demo"
```

---

## Task 3: Features Section

**Files:**
- Modify: `website/index.html`

- [ ] **Step 1: Add features HTML**

```html
<section id="features" class="features">
    <h2>Why TermCast?</h2>
    <div class="features-grid">
        <div class="feature-card">
            <div class="feature-icon">✨</div>
            <h3>Beautiful Output</h3>
            <p>ANSI-colored weather with icons that render perfectly in iTerm2, Terminal, and kitty. Screenshot-worthy output.</p>
        </div>
        <div class="feature-card">
            <div class="feature-icon">🚀</div>
            <h3>Zero Config</h3>
            <p>Auto-detects your location via IP. Works on first run with zero setup. No API keys required.</p>
        </div>
        <div class="feature-card">
            <div class="feature-icon">⚡</div>
            <h3>Ambient Mode</h3>
            <p>Compact single-line output perfect for shell prompts and tmux status bars. Blazingly fast.</p>
        </div>
    </div>
</section>
```

- [ ] **Step 2: Add features CSS**

```css
.features {
    padding: 6rem 8%;
    background: var(--bg);
}

.features h2 {
    text-align: center;
    font-size: 2.5rem;
    margin-bottom: 3rem;
}

.features-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
    max-width: 1200px;
    margin: 0 auto;
}

.feature-card {
    background: var(--surface);
    padding: 2rem;
    border-radius: 16px;
    border: 1px solid transparent;
    transition: all 0.3s ease;
}

.feature-card:hover {
    border-color: var(--cyan);
    transform: translateY(-4px);
    box-shadow: 0 20px 40px -20px rgba(34, 211, 238, 0.2);
}

.feature-icon {
    font-size: 2.5rem;
    margin-bottom: 1rem;
}

.feature-card h3 {
    font-size: 1.25rem;
    margin-bottom: 0.75rem;
    color: var(--cyan);
}

.feature-card p {
    color: var(--muted);
    line-height: 1.6;
}
```

- [ ] **Step 3: Test in browser**

- [ ] **Step 4: Commit**

```bash
git add website/index.html
git commit -m "feat: add features section"
```

---

## Task 4: How It Works Section

**Files:**
- Modify: `website/index.html`

- [ ] **Step 1: Add how it works HTML**

```html
<section id="how-it-works" class="how-it-works">
    <h2>How It Works</h2>
    <div class="steps">
        <div class="step">
            <div class="step-number">1</div>
            <div class="step-content">
                <h3>Install</h3>
                <p>Add TermCast to your Rust toolchain</p>
                <code class="step-code">cargo install termcast</code>
            </div>
        </div>
        <div class="step-connector"></div>
        <div class="step">
            <div class="step-number">2</div>
            <div class="step-content">
                <h3>Run</h3>
                <p>Execute the command</p>
                <code class="step-code">./termcast</code>
            </div>
        </div>
        <div class="step-connector"></div>
        <div class="step">
            <div class="step-number">3</div>
            <div class="step-content">
                <h3>Enjoy</h3>
                <p>Beautiful weather in your terminal</p>
                <div class="mini-terminal">
                    <pre>☀️ 14° Oslo</pre>
                </div>
            </div>
        </div>
    </div>
</section>
```

- [ ] **Step 2: Add how it works CSS**

```css
.how-it-works {
    padding: 6rem 8%;
    background: var(--surface);
}

.how-it-works h2 {
    text-align: center;
    font-size: 2.5rem;
    margin-bottom: 4rem;
}

.steps {
    display: flex;
    align-items: flex-start;
    justify-content: center;
    gap: 1rem;
    max-width: 1000px;
    margin: 0 auto;
}

.step {
    flex: 1;
    text-align: center;
}

.step-number {
    width: 60px;
    height: 60px;
    background: linear-gradient(135deg, var(--cyan), var(--green));
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0 auto 1.5rem;
}

.step-content h3 {
    font-size: 1.25rem;
    margin-bottom: 0.5rem;
}

.step-content p {
    color: var(--muted);
    margin-bottom: 1rem;
}

.step-code {
    display: inline-block;
    background: var(--bg);
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    font-family: var(--font-mono);
    color: var(--cyan);
}

.step-connector {
    width: 60px;
    height: 2px;
    background: linear-gradient(90deg, var(--cyan), var(--green));
    margin-top: 30px;
    flex-shrink: 0;
}

.mini-terminal {
    background: var(--bg);
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    display: inline-block;
}

.mini-terminal pre {
    font-family: var(--font-mono);
    color: var(--amber);
}
```

- [ ] **Step 3: Test in browser**

- [ ] **Step 4: Commit**

```bash
git add website/index.html
git commit -m "feat: add how it works section"
```

---

## Task 5: Demo Section with Preset Cards and Live Weather

**Files:**
- Modify: `website/index.html`

- [ ] **Step 1: Add demo section HTML**

```html
<section id="demo" class="demo">
    <h2>See It In Action</h2>
    <p class="demo-intro">TermCast renders weather beautifully across different conditions and locations.</p>

    <!-- Preset Weather Cards -->
    <div class="weather-cards">
        <div class="weather-card">
            <div class="weather-card-header">
                <span class="location-name">Oslo, Norway</span>
                <span class="weather-time">Now</span>
            </div>
            <div class="weather-card-body">
                <span class="weather-icon">☀️</span>
                <span class="weather-temp">14°C</span>
            </div>
            <div class="weather-card-footer">
                <span class="weather-condition">Clear sky</span>
            </div>
            <div class="terminal-mini">
                <pre>☀️ 14°C Oslo</pre>
            </div>
        </div>

        <div class="weather-card">
            <div class="weather-card-header">
                <span class="location-name">Miami, USA</span>
                <span class="weather-time">Now</span>
            </div>
            <div class="weather-card-body">
                <span class="weather-icon">🌤</span>
                <span class="weather-temp">28°C</span>
            </div>
            <div class="weather-card-footer">
                <span class="weather-condition">Sunny</span>
            </div>
            <div class="terminal-mini">
                <pre>🌤 28°C Miami</pre>
            </div>
        </div>

        <div class="weather-card">
            <div class="weather-card-header">
                <span class="location-name">Reykjavik, Iceland</span>
                <span class="weather-time">Now</span>
            </div>
            <div class="weather-card-body">
                <span class="weather-icon">❄</span>
                <span class="weather-temp">-2°C</span>
            </div>
            <div class="weather-card-footer">
                <span class="weather-condition">Light snow</span>
            </div>
            <div class="terminal-mini">
                <pre>❄ -2°C Reykjavik</pre>
            </div>
        </div>

        <div class="weather-card">
            <div class="weather-card-header">
                <span class="location-name">Tokyo, Japan</span>
                <span class="weather-time">Now</span>
            </div>
            <div class="weather-card-body">
                <span class="weather-icon">🌧</span>
                <span class="weather-temp">18°C</span>
            </div>
            <div class="weather-card-footer">
                <span class="weather-condition">Light rain</span>
            </div>
            <div class="terminal-mini">
                <pre>🌧 18°C Tokyo</pre>
            </div>
        </div>
    </div>

    <!-- Interactive Demo -->
    <div class="interactive-demo">
        <h3>Try It Live</h3>
        <p>Enter a city name to see ambient mode output in real-time</p>
        <div class="demo-input-group">
            <input type="text" id="city-input" placeholder="Enter city name (e.g., London)" />
            <button id="fetch-weather" class="btn-secondary">Get Weather</button>
        </div>
        <div class="demo-output" id="demo-output">
            <p class="demo-placeholder">Weather output will appear here...</p>
        </div>
    </div>
</section>
```

- [ ] **Step 2: Add demo CSS**

```css
.demo {
    padding: 6rem 8%;
    background: var(--bg);
}

.demo h2 {
    text-align: center;
    font-size: 2.5rem;
    margin-bottom: 1rem;
}

.demo-intro {
    text-align: center;
    color: var(--muted);
    margin-bottom: 3rem;
}

.weather-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 1.5rem;
    max-width: 1200px;
    margin: 0 auto 4rem;
}

.weather-card {
    background: var(--surface);
    border-radius: 16px;
    padding: 1.5rem;
    border: 1px solid transparent;
    transition: all 0.3s ease;
}

.weather-card:hover {
    border-color: var(--amber);
    transform: translateY(-4px);
    box-shadow: 0 20px 40px -20px rgba(251, 191, 36, 0.2);
}

.weather-card-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 1rem;
}

.location-name {
    font-weight: 600;
}

.weather-time {
    color: var(--muted);
    font-size: 0.875rem;
}

.weather-card-body {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
}

.weather-icon {
    font-size: 3rem;
}

.weather-temp {
    font-size: 2rem;
    font-weight: 700;
    color: var(--cyan);
}

.weather-card-footer {
    margin-bottom: 1rem;
}

.weather-condition {
    color: var(--muted);
    font-size: 0.9rem;
}

.terminal-mini {
    background: var(--bg);
    padding: 0.75rem 1rem;
    border-radius: 8px;
    border-left: 3px solid var(--cyan);
}

.terminal-mini pre {
    font-family: var(--font-mono);
    font-size: 0.95rem;
    color: var(--text);
}

/* Interactive Demo */
.interactive-demo {
    max-width: 600px;
    margin: 0 auto;
    text-align: center;
    background: var(--surface);
    padding: 2.5rem;
    border-radius: 16px;
}

.interactive-demo h3 {
    font-size: 1.5rem;
    margin-bottom: 0.5rem;
}

.interactive-demo p {
    color: var(--muted);
    margin-bottom: 1.5rem;
}

.demo-input-group {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
}

#city-input {
    flex: 1;
    background: var(--bg);
    border: 1px solid var(--muted);
    padding: 0.875rem 1rem;
    border-radius: 8px;
    color: var(--text);
    font-size: 1rem;
}

#city-input:focus {
    outline: none;
    border-color: var(--cyan);
}

.btn-secondary {
    background: var(--cyan);
    color: var(--bg);
    border: none;
    padding: 0.875rem 1.5rem;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
}

.btn-secondary:hover {
    background: var(--green);
    transform: translateY(-2px);
}

.demo-output {
    background: var(--bg);
    padding: 1.5rem;
    border-radius: 8px;
    min-height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.demo-output pre {
    font-family: var(--font-mono);
    font-size: 1.25rem;
}

.demo-placeholder {
    color: var(--muted);
}

.demo-error {
    color: #ff6b6b;
}
```

- [ ] **Step 3: Add weather fetching JavaScript**

```javascript
// Weather icon mapping based on WMO code
function getWeatherIcon(code) {
    if (code === 0) return '☀️';
    if (code >= 1 && code <= 3) return '🌤';
    if (code >= 45 && code <= 48) return '🌫';
    if (code >= 51 && code <= 67) return '🌧';
    if (code >= 71 && code <= 77) return '❄';
    if (code >= 80 && code <= 82) return '🌦';
    if (code >= 95) return '⛈';
    return '☁';
}

// Get temperature unit based on locale
function getTempUnit() {
    const locale = navigator.language || navigator.userLanguage;
    return locale.startsWith('en-US') ? '°F' : '°C';
}

// Fetch weather for a city
async function fetchWeather(city) {
    try {
        // Geocoding: convert city name to coordinates
        const geoResponse = await fetch(
            `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(city)}&count=1`
        );
        const geoData = await geoResponse.json();

        if (!geoData.results || geoData.results.length === 0) {
            return { error: `City "${city}" not found` };
        }

        const { latitude, longitude, name, country } = geoData.results[0];

        // Weather fetch
        const unit = getTempUnit() === '°F' ? 'fahrenheit' : 'celsius';
        const weatherResponse = await fetch(
            `https://api.open-meteo.com/v1/forecast?latitude=${latitude}&longitude=${longitude}&current=temperature_2m,weather_code&temperature_unit=${unit}`
        );
        const weatherData = await weatherResponse.json();

        const temp = Math.round(weatherData.current.temperature_2m);
        const icon = getWeatherIcon(weatherData.current.weather_code);
        const symbol = getTempUnit();

        return {
            icon,
            temp,
            symbol,
            name,
            country,
            output: `${icon} ${temp}${symbol} ${name}`
        };
    } catch (error) {
        return { error: 'Failed to fetch weather data' };
    }
}

// Handle demo form submission
document.getElementById('fetch-weather').addEventListener('click', async () => {
    const input = document.getElementById('city-input');
    const output = document.getElementById('demo-output');
    const city = input.value.trim();

    if (!city) {
        output.innerHTML = '<p class="demo-error">Please enter a city name</p>';
        return;
    }

    output.innerHTML = '<p style="color: var(--muted);">Fetching...</p>';

    const result = await fetchWeather(city);

    if (result.error) {
        output.innerHTML = `<p class="demo-error">${result.error}</p>`;
    } else {
        output.innerHTML = `<pre>${result.output}</pre>`;
    }
});
```

- [ ] **Step 4: Test in browser**

- [ ] **Step 5: Commit**

```bash
git add website/index.html
git commit -m "feat: add demo section with preset cards and live weather"
```

---

## Task 6: Shell Integration Section with Tabs

**Files:**
- Modify: `website/index.html`

- [ ] **Step 1: Add shell integration HTML**

```html
<section id="integration" class="integration">
    <h2>Shell Integration</h2>
    <p class="integration-intro">Add weather to your prompt with these simple snippets.</p>

    <div class="tabs-container">
        <div class="tab-buttons">
            <button class="tab-btn active" data-tab="bash">Bash</button>
            <button class="tab-btn" data-tab="zsh">Zsh</button>
            <button class="tab-btn" data-tab="tmux">tmux</button>
        </div>

        <div class="tab-content active" id="bash">
            <div class="code-block">
                <div class="code-header">
                    <span class="code-lang">Bash</span>
                    <button class="copy-btn" data-copy="bash-code">
                        <span class="copy-icon">📋</span>
                        <span class="copy-text">Copy</span>
                    </button>
                </div>
                <pre><code id="bash-code">termcast_prompt() {
    termcast --ambient
}
export PS1='\u@\h \w $(termcast_prompt)$ '</code></pre>
            </div>
        </div>

        <div class="tab-content" id="zsh">
            <div class="code-block">
                <div class="code-header">
                    <span class="code-lang">Zsh</span>
                    <button class="copy-btn" data-copy="zsh-code">
                        <span class="copy-icon">📋</span>
                        <span class="copy-text">Copy</span>
                    </button>
                </div>
                <pre><code id="zsh-code">termcast_prompt() {
    termcast --ambient
}
PROMPT='%n@%m %~ $(termcast_prompt)% '</code></pre>
            </div>
        </div>

        <div class="tab-content" id="tmux">
            <div class="code-block">
                <div class="code-header">
                    <span class="code-lang">tmux</span>
                    <button class="copy-btn" data-copy="tmux-code">
                        <span class="copy-icon">📋</span>
                        <span class="copy-text">Copy</span>
                    </button>
                </div>
                <pre><code id="tmux-code">set -g status-right '#(termcast --ambient)'</code></pre>
            </div>
        </div>
    </div>
</section>
```

- [ ] **Step 2: Add integration CSS**

```css
.integration {
    padding: 6rem 8%;
    background: var(--surface);
}

.integration h2 {
    text-align: center;
    font-size: 2.5rem;
    margin-bottom: 1rem;
}

.integration-intro {
    text-align: center;
    color: var(--muted);
    margin-bottom: 3rem;
}

.tabs-container {
    max-width: 700px;
    margin: 0 auto;
}

.tab-buttons {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
    justify-content: center;
}

.tab-btn {
    background: transparent;
    border: 1px solid var(--muted);
    color: var(--muted);
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.3s ease;
}

.tab-btn:hover {
    border-color: var(--cyan);
    color: var(--cyan);
}

.tab-btn.active {
    background: var(--cyan);
    border-color: var(--cyan);
    color: var(--bg);
}

.tab-content {
    display: none;
}

.tab-content.active {
    display: block;
}

.code-block {
    background: var(--bg);
    border-radius: 12px;
    overflow: hidden;
}

.code-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: rgba(0, 0, 0, 0.3);
}

.code-lang {
    font-family: var(--font-mono);
    font-size: 0.875rem;
    color: var(--green);
}

.copy-btn {
    background: transparent;
    border: none;
    color: var(--muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    transition: all 0.3s ease;
}

.copy-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
}

.copy-btn.copied {
    color: var(--green);
}

.code-block pre {
    padding: 1.5rem;
    overflow-x: auto;
}

.code-block code {
    font-family: var(--font-mono);
    font-size: 0.9rem;
    line-height: 1.6;
    color: var(--text);
}
```

- [ ] **Step 3: Add tab switching and copy JavaScript**

```javascript
// Tab switching
document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        const tabName = btn.dataset.tab;

        // Update buttons
        document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');

        // Update content
        document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
        document.getElementById(tabName).classList.add('active');
    });
});

// Copy to clipboard
document.querySelectorAll('.copy-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        const codeId = btn.dataset.copy;
        const code = document.getElementById(codeId).textContent;

        navigator.clipboard.writeText(code).then(() => {
            btn.classList.add('copied');
            const textEl = btn.querySelector('.copy-text');
            const originalText = textEl.textContent;
            textEl.textContent = 'Copied!';

            setTimeout(() => {
                btn.classList.remove('copied');
                textEl.textContent = originalText;
            }, 2000);
        });
    });
});
```

- [ ] **Step 4: Test in browser**

- [ ] **Step 5: Commit**

```bash
git add website/index.html
git commit -m "feat: add shell integration section with tabs"
```

---

## Task 7: Navigation, Footer, and Polish

**Files:**
- Modify: `website/index.html`

- [ ] **Step 1: Add navigation HTML and CSS**

```html
<nav class="nav">
    <div class="nav-brand">
        <span class="nav-logo">☀️</span>
        <span class="nav-name">TermCast</span>
    </div>
    <div class="nav-links">
        <a href="#features">Features</a>
        <a href="#demo">Demo</a>
        <a href="#integration">Integration</a>
        <a href="https://github.com/yourusername/termcast" target="_blank" class="nav-github">GitHub</a>
    </div>
</nav>
```

```css
.nav {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 8%;
    background: rgba(13, 13, 13, 0.9);
    backdrop-filter: blur(10px);
    z-index: 100;
}

.nav-brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 600;
    font-size: 1.25rem;
}

.nav-logo {
    font-size: 1.5rem;
}

.nav-links {
    display: flex;
    gap: 2rem;
    align-items: center;
}

.nav-links a {
    color: var(--muted);
    text-decoration: none;
    transition: color 0.3s ease;
}

.nav-links a:hover {
    color: var(--cyan);
}

.nav-github {
    background: var(--surface);
    padding: 0.5rem 1rem;
    border-radius: 6px;
}

.nav-github:hover {
    background: var(--cyan);
    color: var(--bg) !important;
}
```

- [ ] **Step 2: Add footer HTML and CSS**

```html
<footer class="footer">
    <div class="footer-content">
        <div class="footer-brand">
            <span class="footer-logo">☀️</span>
            <span>TermCast</span>
        </div>
        <div class="footer-links">
            <a href="https://github.com/yourusername/termcast" target="_blank">GitHub</a>
            <span class="footer-divider">•</span>
            <span>MIT License</span>
            <span class="footer-divider">•</span>
            <span>Built with Rust 🦀</span>
        </div>
    </div>
</footer>
```

```css
.footer {
    padding: 3rem 8%;
    background: var(--bg);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.footer-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 1rem;
}

.footer-brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 600;
}

.footer-logo {
    font-size: 1.25rem;
}

.footer-links {
    display: flex;
    align-items: center;
    gap: 1rem;
    color: var(--muted);
    font-size: 0.875rem;
}

.footer-links a {
    color: var(--muted);
    text-decoration: none;
}

.footer-links a:hover {
    color: var(--cyan);
}

.footer-divider {
    opacity: 0.5;
}
```

- [ ] **Step 3: Add global styles and polish CSS**

```css
:root {
    --bg: #0d0d0d;
    --surface: #1a1a1a;
    --cyan: #22d3ee;
    --amber: #fbbf24;
    --green: #34d399;
    --text: #f5f5f5;
    --muted: #888888;
    --font-body: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    --font-mono: 'JetBrains Mono', 'Fira Code', monospace;
}

*, *::before, *::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

html {
    scroll-behavior: smooth;
}

body {
    font-family: var(--font-body);
    background: var(--bg);
    color: var(--text);
    line-height: 1.6;
}

a {
    color: var(--cyan);
    text-decoration: none;
}

section {
    scroll-margin-top: 80px;
}

/* Scrollbar styling */
::-webkit-scrollbar {
    width: 8px;
}

::-webkit-scrollbar-track {
    background: var(--bg);
}

::-webkit-scrollbar-thumb {
    background: var(--surface);
    border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
    background: var(--muted);
}

/* Responsive */
@media (max-width: 768px) {
    .hero {
        flex-direction: column;
        text-align: center;
    }

    .hero h1 {
        font-size: 2.5rem;
    }

    .steps {
        flex-direction: column;
        align-items: center;
    }

    .step-connector {
        width: 2px;
        height: 40px;
        margin: 0;
    }

    .nav-links {
        display: none;
    }

    .footer-content {
        flex-direction: column;
        text-align: center;
    }
}
```

- [ ] **Step 4: Add copy functionality for install button**

```javascript
// Install button copy
document.getElementById('install-btn').addEventListener('click', function() {
    navigator.clipboard.writeText('cargo install termcast').then(() => {
        const hint = this.querySelector('.copy-hint');
        hint.textContent = 'Copied!';
        setTimeout(() => {
            hint.textContent = 'Click to copy';
        }, 2000);
    });
});
```

- [ ] **Step 5: Test all interactions in browser**

- [ ] **Step 6: Commit**

```bash
git add website/index.html
git commit -m "feat: add navigation, footer, and global polish"
```

---

## Task 8: Final Verification and Testing

**Files:**
- Modify: `website/index.html`

- [ ] **Step 1: Test all sections render correctly**

- [ ] **Step 2: Test smooth scroll navigation**

- [ ] **Step 3: Test copy buttons work**

- [ ] **Step 4: Test tab switching**

- [ ] **Step 5: Test live weather demo with a city name**

- [ ] **Step 6: Test responsive layout**

- [ ] **Step 7: Final commit**

```bash
git add website/index.html
git commit -m "feat: complete TermCast landing page"
```

---

## Verification Checklist

- [ ] All 6 sections present (Hero, Features, How It Works, Demo, Integration, Footer)
- [ ] Hero terminal window has floating animation
- [ ] Feature cards have hover effects
- [ ] Weather cards show preset locations with mini terminal output
- [ ] Live demo fetches real weather from Open-Meteo
- [ ] Tab switching works for shell integration
- [ ] Copy buttons functional
- [ ] Mobile responsive
- [ ] No external images (all CSS/emoji)
- [ ] Google Fonts load (Inter, JetBrains Mono)
- [ ] Smooth scroll navigation
- [ ] Dark theme with terminal-inspired colors

---

## Implementation Notes

1. **Color scheme**: Deep charcoal background (#0d0d0d), cyan accent (#22d3ee), amber for weather icons (#fbbf24), green for success/location (#34d399)

2. **Weather API**: Open-Meteo is free and requires no API key
   - Geocoding endpoint: `https://geocoding-api.open-meteo.com/v1/search`
   - Weather endpoint: `https://api.open-meteo.com/v1/forecast`

3. **Temperature units**: Detect US locale (en-US) and use Fahrenheit, otherwise Celsius

4. **GitHub link**: Replace `yourusername` with actual GitHub username before deployment
