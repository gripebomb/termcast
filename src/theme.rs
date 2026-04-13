//! Color theme engine for terminal weather output.
//!
//! Provides named color palettes that replace hardcoded terminal colors.
//! Users select a theme via config or CLI flags.

use crossterm::style::Color;

/// The 6 semantic color slots used by the renderer.
///
/// Each slot maps to a specific visual role in the weather output.
/// All built-in themes define values for every slot.
pub struct ThemeColors {
    pub text: Color,
    pub dimmed: Color,
    pub temp_high: Color,
    pub temp_low: Color,
    pub precip_high: Color,
    pub precip_medium: Color,
}

/// A named color theme with description and color palette.
pub struct Theme {
    pub name: &'static str,
    pub description: &'static str,
    pub aliases: &'static [&'static str],
    pub colors: ThemeColors,
}

/// Returns the default colors matching the original hardcoded values.
pub fn default_colors() -> ThemeColors {
    ThemeColors {
        text: Color::Rgb { r: 255, g: 255, b: 255 },
        dimmed: Color::Rgb { r: 169, g: 169, b: 169 },
        temp_high: Color::Rgb { r: 0, g: 255, b: 255 },
        temp_low: Color::Rgb { r: 255, g: 0, b: 255 },
        precip_high: Color::Rgb { r: 255, g: 0, b: 0 },
        precip_medium: Color::Rgb { r: 255, g: 255, b: 0 },
    }
}

static BUILTIN_THEMES: [Theme; 11] = [
    Theme {
        name: "default",
        description: "Current hardcoded colors (white, cyan, magenta)",
        aliases: &[],
        colors: ThemeColors {
            text: Color::Rgb { r: 255, g: 255, b: 255 },
            dimmed: Color::Rgb { r: 169, g: 169, b: 169 },
            temp_high: Color::Rgb { r: 0, g: 255, b: 255 },
            temp_low: Color::Rgb { r: 255, g: 0, b: 255 },
            precip_high: Color::Rgb { r: 255, g: 0, b: 0 },
            precip_medium: Color::Rgb { r: 255, g: 255, b: 0 },
        },
    },
    Theme {
        name: "catppuccin",
        description: "Catppuccin Mocha - warm dark pastels",
        aliases: &["catppuccin-mocha"],
        colors: ThemeColors {
            text: Color::Rgb { r: 205, g: 214, b: 244 },
            dimmed: Color::Rgb { r: 147, g: 153, b: 178 },
            temp_high: Color::Rgb { r: 137, g: 180, b: 250 },
            temp_low: Color::Rgb { r: 203, g: 166, b: 247 },
            precip_high: Color::Rgb { r: 243, g: 139, b: 168 },
            precip_medium: Color::Rgb { r: 249, g: 226, b: 175 },
        },
    },
    Theme {
        name: "catppuccin-latte",
        description: "Catppuccin Latte - warm light pastels",
        aliases: &[],
        colors: ThemeColors {
            text: Color::Rgb { r: 76, g: 79, b: 105 },
            dimmed: Color::Rgb { r: 150, g: 153, b: 166 },
            temp_high: Color::Rgb { r: 30, g: 102, b: 245 },
            temp_low: Color::Rgb { r: 136, g: 57, b: 239 },
            precip_high: Color::Rgb { r: 210, g: 15, b: 57 },
            precip_medium: Color::Rgb { r: 223, g: 186, b: 52 },
        },
    },
    Theme {
        name: "dracula",
        description: "Dracula - dark with vivid accents",
        aliases: &[],
        colors: ThemeColors {
            text: Color::Rgb { r: 248, g: 248, b: 242 },
            dimmed: Color::Rgb { r: 98, g: 114, b: 164 },
            temp_high: Color::Rgb { r: 189, g: 147, b: 249 },
            temp_low: Color::Rgb { r: 255, g: 121, b: 198 },
            precip_high: Color::Rgb { r: 255, g: 85, b: 85 },
            precip_medium: Color::Rgb { r: 241, g: 250, b: 140 },
        },
    },
    Theme {
        name: "nord",
        description: "Nord - arctic blue-gray palette",
        aliases: &[],
        colors: ThemeColors {
            text: Color::Rgb { r: 216, g: 222, b: 233 },
            dimmed: Color::Rgb { r: 76, g: 86, b: 106 },
            temp_high: Color::Rgb { r: 136, g: 192, b: 208 },
            temp_low: Color::Rgb { r: 180, g: 142, b: 173 },
            precip_high: Color::Rgb { r: 191, g: 97, b: 106 },
            precip_medium: Color::Rgb { r: 235, g: 203, b: 139 },
        },
    },
    Theme {
        name: "solarized",
        description: "Solarized Dark - warm earth tones",
        aliases: &["solarized-dark"],
        colors: ThemeColors {
            text: Color::Rgb { r: 147, g: 161, b: 161 },
            dimmed: Color::Rgb { r: 88, g: 110, b: 117 },
            temp_high: Color::Rgb { r: 42, g: 161, b: 152 },
            temp_low: Color::Rgb { r: 108, g: 113, b: 196 },
            precip_high: Color::Rgb { r: 220, g: 50, b: 47 },
            precip_medium: Color::Rgb { r: 181, g: 137, b: 0 },
        },
    },
    Theme {
        name: "solarized-light",
        description: "Solarized Light - warm cream with accents",
        aliases: &[],
        colors: ThemeColors {
            text: Color::Rgb { r: 101, g: 123, b: 131 },
            dimmed: Color::Rgb { r: 147, g: 161, b: 161 },
            temp_high: Color::Rgb { r: 42, g: 161, b: 152 },
            temp_low: Color::Rgb { r: 108, g: 113, b: 196 },
            precip_high: Color::Rgb { r: 220, g: 50, b: 47 },
            precip_medium: Color::Rgb { r: 181, g: 137, b: 0 },
        },
    },
    Theme {
        name: "tokyo-night",
        description: "Tokyo Night - deep blue city nights",
        aliases: &[],
        colors: ThemeColors {
            text: Color::Rgb { r: 169, g: 177, b: 214 },
            dimmed: Color::Rgb { r: 86, g: 95, b: 137 },
            temp_high: Color::Rgb { r: 122, g: 162, b: 247 },
            temp_low: Color::Rgb { r: 187, g: 154, b: 247 },
            precip_high: Color::Rgb { r: 247, g: 118, b: 142 },
            precip_medium: Color::Rgb { r: 224, g: 175, b: 104 },
        },
    },
    Theme {
        name: "tokyo-night-light",
        description: "Tokyo Night Light - cool day variant",
        aliases: &[],
        colors: ThemeColors {
            text: Color::Rgb { r: 52, g: 59, b: 85 },
            dimmed: Color::Rgb { r: 150, g: 153, b: 166 },
            temp_high: Color::Rgb { r: 50, g: 107, b: 215 },
            temp_low: Color::Rgb { r: 116, g: 75, b: 207 },
            precip_high: Color::Rgb { r: 225, g: 65, b: 80 },
            precip_medium: Color::Rgb { r: 188, g: 144, b: 40 },
        },
    },
    Theme {
        name: "gruvbox",
        description: "Gruvbox - warm retro earth tones",
        aliases: &["gruvbox-dark"],
        colors: ThemeColors {
            text: Color::Rgb { r: 235, g: 219, b: 178 },
            dimmed: Color::Rgb { r: 146, g: 131, b: 116 },
            temp_high: Color::Rgb { r: 104, g: 157, b: 106 },
            temp_low: Color::Rgb { r: 215, g: 153, b: 33 },
            precip_high: Color::Rgb { r: 204, g: 36, b: 29 },
            precip_medium: Color::Rgb { r: 214, g: 93, b: 14 },
        },
    },
    Theme {
        name: "gruvbox-light",
        description: "Gruvbox Light - warm light retro tones",
        aliases: &[],
        colors: ThemeColors {
            text: Color::Rgb { r: 60, g: 56, b: 54 },
            dimmed: Color::Rgb { r: 124, g: 111, b: 100 },
            temp_high: Color::Rgb { r: 50, g: 130, b: 52 },
            temp_low: Color::Rgb { r: 188, g: 134, b: 24 },
            precip_high: Color::Rgb { r: 157, g: 0, b: 6 },
            precip_medium: Color::Rgb { r: 214, g: 93, b: 14 },
        },
    },
];

/// Returns all built-in themes as a static slice.
pub fn builtin_themes() -> &'static [Theme] {
    &BUILTIN_THEMES
}

impl ThemeColors {
    /// Converts RGB colors to ANSI 256-color values for terminals
    /// without true-color support.
    pub fn to_ansi256(&self) -> ThemeColors {
        ThemeColors {
            text: rgb_to_ansi256(self.text),
            dimmed: rgb_to_ansi256(self.dimmed),
            temp_high: rgb_to_ansi256(self.temp_high),
            temp_low: rgb_to_ansi256(self.temp_low),
            precip_high: rgb_to_ansi256(self.precip_high),
            precip_medium: rgb_to_ansi256(self.precip_medium),
        }
    }
}

/// Converts a Color to ANSI 256-color index. Non-RGB colors pass through.
fn rgb_to_ansi256(color: Color) -> Color {
    let (r, g, b) = match color {
        Color::Rgb { r, g, b } => (r, g, b),
        other => return other,
    };

    // Check if it matches a greyscale ramp (232-255)
    if r == g && g == b && r < 8 {
        return Color::AnsiValue(16);
    }
    if r == g && g == b && r > 248 {
        return Color::AnsiValue(231);
    }
    if r == g && g == b {
        return Color::AnsiValue(232 + ((r - 8) as u16 * 24 / 230) as u8);
    }

    // Map to 6x6x6 color cube (16-231)
    let ri = (r as u16 * 5 / 255) as u8;
    let gi = (g as u16 * 5 / 255) as u8;
    let bi = (b as u16 * 5 / 255) as u8;
    Color::AnsiValue(16 + 36 * ri + 6 * gi + bi)
}

/// Checks if the terminal supports true-color via COLORTERM env var.
pub fn supports_truecolor() -> bool {
    match std::env::var("COLORTERM") {
        Ok(val) => val == "truecolor" || val == "24bit",
        Err(_) => false,
    }
}

/// Normalizes a theme name: lowercase, underscores to hyphens.
fn normalize_name(name: &str) -> String {
    name.to_lowercase().replace('_', "-")
}

/// Resolves a theme name to its colors.
///
/// - Case-insensitive matching
/// - Underscores and hyphens treated equivalently
/// - Unknown names return default colors
pub fn resolve_theme(name: &str) -> &'static ThemeColors {
    let normalized = normalize_name(name);

    for theme in builtin_themes() {
        if normalize_name(theme.name) == normalized {
            return &theme.colors;
        }
        for alias in theme.aliases {
            if normalize_name(alias) == normalized {
                return &theme.colors;
            }
        }
    }

    static DEFAULTS: ThemeColors = ThemeColors {
        text: Color::Rgb { r: 255, g: 255, b: 255 },
        dimmed: Color::Rgb { r: 169, g: 169, b: 169 },
        temp_high: Color::Rgb { r: 0, g: 255, b: 255 },
        temp_low: Color::Rgb { r: 255, g: 0, b: 255 },
        precip_high: Color::Rgb { r: 255, g: 0, b: 0 },
        precip_medium: Color::Rgb { r: 255, g: 255, b: 0 },
    };
    &DEFAULTS
}

/// Resolves a theme name, returning `Some(&ThemeColors)` if found
/// or `None` if the name doesn't match any built-in theme.
pub fn resolve_theme_checked(name: &str) -> Option<&'static ThemeColors> {
    let normalized = normalize_name(name);

    for theme in builtin_themes() {
        if normalize_name(theme.name) == normalized {
            return Some(&theme.colors);
        }
        for alias in theme.aliases {
            if normalize_name(alias) == normalized {
                return Some(&theme.colors);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_themes_returns_11() {
        assert_eq!(builtin_themes().len(), 11);
    }

    #[test]
    fn test_resolve_default_returns_hardcoded_colors() {
        let colors = resolve_theme("default");
        assert!(matches!(colors.text, Color::Rgb { r: 255, g: 255, b: 255 }));
        assert!(matches!(colors.dimmed, Color::Rgb { r: 169, g: 169, b: 169 }));
        assert!(matches!(colors.temp_high, Color::Rgb { r: 0, g: 255, b: 255 }));
        assert!(matches!(colors.temp_low, Color::Rgb { r: 255, g: 0, b: 255 }));
        assert!(matches!(colors.precip_high, Color::Rgb { r: 255, g: 0, b: 0 }));
        assert!(matches!(colors.precip_medium, Color::Rgb { r: 255, g: 255, b: 0 }));
    }

    #[test]
    fn test_resolve_nonexistent_returns_default() {
        let colors = resolve_theme("nonexistent");
        assert!(matches!(colors.text, Color::Rgb { r: 255, g: 255, b: 255 }));
    }

    #[test]
    fn test_resolve_case_insensitive() {
        let upper = resolve_theme("CATPPUCCIN");
        assert!(matches!(upper.text, Color::Rgb { r: 205, g: 214, b: 244 }));
    }

    #[test]
    fn test_resolve_underscore_equals_hyphen() {
        let underscore = resolve_theme("catppuccin_mocha");
        let hyphen = resolve_theme("catppuccin-mocha");
        assert!(matches!(underscore.text, Color::Rgb { r: 205, g: 214, b: 244 }));
        assert!(matches!(hyphen.text, Color::Rgb { r: 205, g: 214, b: 244 }));
    }

    #[test]
    fn test_resolve_catppuccin_mocha() {
        let colors = resolve_theme("catppuccin");
        assert!(matches!(colors.text, Color::Rgb { r: 205, g: 214, b: 244 }));
        assert!(matches!(colors.dimmed, Color::Rgb { r: 147, g: 153, b: 178 }));
        assert!(matches!(colors.temp_high, Color::Rgb { r: 137, g: 180, b: 250 }));
        assert!(matches!(colors.temp_low, Color::Rgb { r: 203, g: 166, b: 247 }));
        assert!(matches!(colors.precip_high, Color::Rgb { r: 243, g: 139, b: 168 }));
        assert!(matches!(colors.precip_medium, Color::Rgb { r: 249, g: 226, b: 175 }));
    }

    #[test]
    fn test_resolve_dracula() {
        let colors = resolve_theme("dracula");
        assert!(matches!(colors.text, Color::Rgb { r: 248, g: 248, b: 242 }));
        assert!(matches!(colors.dimmed, Color::Rgb { r: 98, g: 114, b: 164 }));
        assert!(matches!(colors.temp_high, Color::Rgb { r: 189, g: 147, b: 249 }));
    }

    #[test]
    fn test_resolve_nord() {
        let colors = resolve_theme("nord");
        assert!(matches!(colors.text, Color::Rgb { r: 216, g: 222, b: 233 }));
    }

    #[test]
    fn test_resolve_solarized() {
        let colors = resolve_theme("solarized");
        assert!(matches!(colors.text, Color::Rgb { r: 147, g: 161, b: 161 }));
    }

    #[test]
    fn test_resolve_solarized_light() {
        let colors = resolve_theme("solarized-light");
        assert!(matches!(colors.text, Color::Rgb { r: 101, g: 123, b: 131 }));
    }

    #[test]
    fn test_resolve_tokyo_night() {
        let colors = resolve_theme("tokyo-night");
        assert!(matches!(colors.text, Color::Rgb { r: 169, g: 177, b: 214 }));
    }

    #[test]
    fn test_resolve_tokyo_night_light() {
        let colors = resolve_theme("tokyo-night-light");
        assert!(matches!(colors.text, Color::Rgb { r: 52, g: 59, b: 85 }));
    }

    #[test]
    fn test_resolve_gruvbox() {
        let colors = resolve_theme("gruvbox");
        assert!(matches!(colors.text, Color::Rgb { r: 235, g: 219, b: 178 }));
    }

    #[test]
    fn test_resolve_gruvbox_light() {
        let colors = resolve_theme("gruvbox-light");
        assert!(matches!(colors.text, Color::Rgb { r: 60, g: 56, b: 54 }));
    }

    #[test]
    fn test_alias_gruvbox_dark() {
        let colors = resolve_theme("gruvbox-dark");
        assert!(matches!(colors.text, Color::Rgb { r: 235, g: 219, b: 178 }));
    }

    #[test]
    fn test_alias_catppuccin_latte() {
        let colors = resolve_theme("catppuccin-latte");
        assert!(matches!(colors.text, Color::Rgb { r: 76, g: 79, b: 105 }));
    }

    #[test]
    fn test_to_ansi256_converts_rgb() {
        let colors = default_colors();
        let ansi = colors.to_ansi256();
        assert!(matches!(ansi.text, Color::AnsiValue(_)));
        assert!(matches!(ansi.temp_high, Color::AnsiValue(_)));
    }

    #[test]
    fn test_rgb_to_ansi256_white() {
        let result = rgb_to_ansi256(Color::Rgb { r: 255, g: 255, b: 255 });
        assert!(matches!(result, Color::AnsiValue(231)));
    }

    #[test]
    fn test_rgb_to_ansi256_black() {
        let result = rgb_to_ansi256(Color::Rgb { r: 0, g: 0, b: 0 });
        assert!(matches!(result, Color::AnsiValue(16)));
    }

    #[test]
    fn test_rgb_to_ansi256_cyan() {
        let result = rgb_to_ansi256(Color::Rgb { r: 0, g: 255, b: 255 });
        assert!(matches!(result, Color::AnsiValue(51)));
    }

    #[test]
    fn test_rgb_to_ansi256_non_rgb_passthrough() {
        let result = rgb_to_ansi256(Color::DarkGrey);
        assert!(matches!(result, Color::DarkGrey));
    }
}
