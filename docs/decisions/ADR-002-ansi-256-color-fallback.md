# ADR-002: ANSI 256-color fallback via COLORTERM detection

## Status
Accepted

## Date
2026-04-13

## Context
TermCast's theme system uses RGB (true-color) values for all color slots. However, not all terminals support 24-bit color. When RGB escape codes are sent to a terminal that only supports 256 colors, the output may render incorrectly or show no color at all.

Terminals that lack true-color support include:
- Screen/tmux sessions (unless configured for true-color)
- Older terminal emulators (xterm, rxvt)
- CI environments and basic consoles

## Decision
Detect true-color support via the `COLORTERM` environment variable and fall back to ANSI 256-color approximations when true-color is unavailable.

### Detection
```rust
pub fn supports_truecolor() -> bool {
    match std::env::var("COLORTERM") {
        Ok(val) => val == "truecolor" || val == "24bit",
        Err(_) => false,
    }
}
```

The `COLORTERM` variable is the standard way terminals advertise true-color support. Modern terminals (iTerm2, kitty, Alacritty, Windows Terminal, Ghostty) set `COLORTERM=truecolor`.

### Color Conversion
Map RGB values to the ANSI 256-color space using the 6x6x6 color cube (indices 16-231):

```rust
fn rgb_to_ansi256(color: Color) -> Color {
    // Greyscale ramp (232-255) for R==G==B
    // Color cube (16-231) for all other colors:
    //   index = 16 + 36*R + 6*G + B
    //   where R, G, B are in [0..5] mapped from [0..255]
}
```

### Adaptation Point
Colors are adapted once in `main.rs` before being passed to the renderer, not on every render call:

```rust
fn adapt_colors_for_terminal(colors: &ThemeColors) -> ThemeColors {
    if supports_truecolor() { colors.clone() } else { colors.to_ansi256() }
}
```

## Alternatives Considered

### Always use ANSI 256 colors
- **Pros:** Maximum compatibility; no detection needed
- **Cons:** Color accuracy suffers significantly on true-color terminals (6x6x6 cube has only 216 distinct colors vs 16.7M RGB)
- **Rejected:** The whole point of themes is precise color matching to terminal schemes. Reducing to 216 colors defeats this.

### Use `TERM` variable instead of `COLORTERM`
- **Pros:** `TERM` is universally set
- **Cons:** `TERM` doesn't directly indicate color depth; would need a lookup table of terminal types; fragile and incomplete
- **Rejected:** `COLORTERM` is the standard and reliable signal for true-color support.

### Query terminal capabilities via terminfo/termcap
- **Pros:** Most accurate detection
- **Cons:** Adds a dependency (`terminfo` crate); not all platforms have terminfo; crossterm already handles low-level terminal I/O
- **Rejected:** Over-engineering. `COLORTERM` is sufficient and widely supported.

### No fallback — just send RGB and hope for the best
- **Pros:** Simplest code
- **Cons:** Broken colors on 256-color terminals; users on those terminals see degraded output
- **Rejected:** TermCast's core value is aesthetic quality. Broken colors undermine that.

## Consequences
- True-color terminals get exact theme colors (no quality loss)
- 256-color terminals get best-effort approximations via the color cube
- The detection is simple and standard (no platform-specific code)
- Some edge cases may not be perfectly handled (e.g., `COLORTERM` unset but terminal supports true-color), but this is rare and acceptable
- The conversion is lossy — some theme colors won't be perfectly represented on 256-color terminals. The 6x6x6 cube provides reasonable approximations for the palettes we ship.
