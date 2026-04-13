//! Terminal renderer for weather output.
//!
//! Uses crossterm for cross-platform terminal styling and ANSI output.

use crate::weather::WeatherDisplay;
use crossterm::{
    style::{Color, Print, SetForegroundColor},
    QueueableCommand,
};
use std::io::{self, Write};

/// Renders weather data to the terminal with styled output.
///
/// Output format (80-column centered):
/// ```text
///      * 14C Oslo
///    Feels 11C
///    High 17C . Low 8C
///    Clear until evening
/// ```
pub fn render_weather(weather: &WeatherDisplay, description: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    let terminal_width: usize = 80;

    // Calculate padding for centering
    let icon = crate::weather::weather_icon(weather.weather_code);
    let main_line = format!(
        "{} {}°C {}",
        icon, weather.temperature as i32, weather.location
    );
    let padding = (terminal_width.saturating_sub(main_line.len())) / 2;
    let main_indent = " ".repeat(padding);

    // Line 1: Icon, temperature, location (bold white)
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(Print(format!("{}{}", main_indent, main_line)))?;
    stdout.queue(Print("\r\n"))?;

    // Line 2: Feels like (dim)
    let feels_line = format!("  Feels {}°", weather.apparent_temperature as i32);
    let feels_padding = (terminal_width.saturating_sub(feels_line.len())) / 2;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!(
        "{}{}",
        " ".repeat(feels_padding),
        feels_line
    )))?;
    stdout.queue(Print("\r\n"))?;

    // Line 3: High/Low (cyan for high, magenta for low)
    let high_low_line = format!(
        "High {}° · Low {}°",
        weather.temp_max as i32, weather.temp_min as i32
    );
    let hl_padding = (terminal_width.saturating_sub(high_low_line.len())) / 2;

    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("{}{}", " ".repeat(hl_padding), "High ")))?;
    stdout.queue(Print(format!("{}°", weather.temp_max as i32)))?;
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(Print(" · "))?;
    stdout.queue(SetForegroundColor(Color::Magenta))?;
    stdout.queue(Print("Low "))?;
    stdout.queue(Print(format!("{}°", weather.temp_min as i32)))?;
    stdout.queue(Print("\r\n"))?;

    // Line 4: Condition description (normal)
    let desc_padding = (terminal_width.saturating_sub(description.len())) / 2;
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(Print(format!(
        "{}{}",
        " ".repeat(desc_padding),
        description
    )))?;
    stdout.queue(Print("\r\n"))?;

    stdout.flush()
}

/// Renders a styled error message.
pub fn render_error(message: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(SetForegroundColor(Color::Red))?;
    stdout.queue(Print("Error: "))?;
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(Print(message))?;
    stdout.queue(Print("\r\n"))?;
    stdout.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::WeatherDisplay;

    #[test]
    fn test_weather_display_construction() {
        let weather = WeatherDisplay {
            temperature: 14.0,
            apparent_temperature: 11.0,
            temp_max: 17.0,
            temp_min: 8.0,
            weather_code: 0,
            location: "Oslo".to_string(),
        };
        // Verify basic data structure works
        assert_eq!(weather.temperature, 14.0);
    }

    #[test]
    fn test_render_weather_output() {
        let weather = WeatherDisplay {
            temperature: 14.0,
            apparent_temperature: 11.0,
            temp_max: 17.0,
            temp_min: 8.0,
            weather_code: 0,
            location: "Test City".to_string(),
        };

        // Just verify it doesn't panic and writes to stdout
        let result = render_weather(&weather, "Clear");
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_error() {
        let result = render_error("Network error");
        assert!(result.is_ok());
    }
}
