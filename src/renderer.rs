//! Terminal renderer for weather output.
//!
//! Uses crossterm for cross-platform terminal styling and ANSI output.

use crate::alerts::Alert;
use crate::forecast::ForecastDisplay;
use crate::theme::ThemeColors;
use crate::weather::WeatherDisplay;
use crossterm::{
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    QueueableCommand,
};
use std::io::{self, Write};

/// Renders weather data to the terminal with styled output.
///
/// Output format (80-column centered):
/// ```text
///      * 14°C Oslo
///    Feels 11°
///    High 17° . Low 8°
///    Clear until evening
/// ```
pub fn render_weather(
    weather: &WeatherDisplay,
    description: &str,
    colors: &ThemeColors,
) -> io::Result<()> {
    let mut stdout = io::stdout();
    let terminal_width: usize = 80;

    let unit = if weather.use_fahrenheit { "°F" } else { "°C" };

    // Calculate padding for centering
    let icon = crate::weather::weather_icon(weather.weather_code);
    let main_line = format!(
        "{} {}{} {}",
        icon, weather.temperature as i32, unit, weather.location
    );
    let padding = (terminal_width.saturating_sub(main_line.len())) / 2;
    let main_indent = " ".repeat(padding);

    // Line 1: Icon, temperature, location
    stdout.queue(SetForegroundColor(colors.text))?;
    stdout.queue(Print(format!("{}{}", main_indent, main_line)))?;
    stdout.queue(Print("\r\n"))?;

    // Line 2: Feels like (dimmed)
    let feels_line = format!("  Feels {}{}", weather.apparent_temperature as i32, unit);
    let feels_padding = (terminal_width.saturating_sub(feels_line.len())) / 2;
    stdout.queue(SetForegroundColor(colors.dimmed))?;
    stdout.queue(Print(format!(
        "{}{}",
        " ".repeat(feels_padding),
        feels_line
    )))?;
    stdout.queue(Print("\r\n"))?;

    // Line 3: High/Low
    let high_low_line = format!(
        "High {}{} · Low {}{}",
        weather.temp_max as i32, unit, weather.temp_min as i32, unit
    );
    let hl_padding = (terminal_width.saturating_sub(high_low_line.len())) / 2;

    stdout.queue(SetForegroundColor(colors.temp_high))?;
    stdout.queue(Print(format!("{}{}", " ".repeat(hl_padding), "High ")))?;
    stdout.queue(Print(format!("{}{}", weather.temp_max as i32, unit)))?;
    stdout.queue(SetForegroundColor(colors.text))?;
    stdout.queue(Print(" · "))?;
    stdout.queue(SetForegroundColor(colors.temp_low))?;
    stdout.queue(Print("Low "))?;
    stdout.queue(Print(format!("{}{}", weather.temp_min as i32, unit)))?;
    stdout.queue(Print("\r\n"))?;

    // Line 4: Condition description
    let desc_padding = (terminal_width.saturating_sub(description.len())) / 2;
    stdout.queue(SetForegroundColor(colors.text))?;
    stdout.queue(Print(format!(
        "{}{}",
        " ".repeat(desc_padding),
        description
    )))?;
    stdout.queue(Print("\r\n"))?;

    stdout.flush()
}

/// Renders a styled error message to stderr.
///
/// Uses red for the "termcast: error: " prefix and white for the message body
/// to match the aesthetic of the rest of the renderer. Writes to stderr so
/// the message stays out of the data stream and is appropriate for shell
/// integration.
pub fn render_error(message: &str) -> io::Result<()> {
    let mut stderr = io::stderr();
    stderr.queue(SetForegroundColor(Color::Red))?;
    stderr.queue(Print("termcast: error: "))?;
    stderr.queue(SetForegroundColor(Color::White))?;
    stderr.queue(Print(message))?;
    stderr.queue(Print("\r\n"))?;
    stderr.flush()
}

/// Renders a centered alert line with severity-based color and style.
///
/// Output format: `{icon} {event} until {time}` centered in 80 columns.
/// When `expires_time` is None, format is `{icon} {event}` with no "until".
/// Colors: Warning = bold red (255,59,48), Watch = yellow (255,204,0),
/// Advisory = dim yellow (204,170,0).
pub fn render_alert_line(alert: &Alert) -> io::Result<()> {
    let mut stdout = io::stdout();
    let terminal_width: usize = 80;

    // Build the warning icon (⚠) + event + optional time
    let icon = "⚠";
    let event = &alert.event;
    let display_line = if let Some(time) = alert.expires_time.as_ref() {
        format!("{} {} until {}", icon, event, time)
    } else {
        format!("{} {}", icon, event)
    };

    let padding = (terminal_width.saturating_sub(display_line.len())) / 2;

    // Apply severity-based color and style
    match alert.severity {
        crate::alerts::AlertSeverity::Warning => {
            stdout.queue(SetAttribute(Attribute::Bold))?;
            stdout.queue(SetForegroundColor(Color::Rgb {
                r: 255,
                g: 59,
                b: 48,
            }))?;
        }
        crate::alerts::AlertSeverity::Watch => {
            stdout.queue(SetForegroundColor(Color::Rgb {
                r: 255,
                g: 204,
                b: 0,
            }))?;
        }
        crate::alerts::AlertSeverity::Advisory => {
            stdout.queue(SetForegroundColor(Color::Rgb {
                r: 204,
                g: 170,
                b: 0,
            }))?;
        }
    }

    stdout.queue(Print(format!("{}{}", " ".repeat(padding), display_line)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(Print("\r\n"))?;
    stdout.flush()
}

/// Renders the daily forecast table.
///
/// Output format (80-column centered):
/// ```text
///         Forecast for Oslo
///
///   Mon   ☀️  17°/8°    ☂ 5%
///   Tue   🌤 15°/7°    ☂ 10%
/// ```
pub fn render_forecast(display: &ForecastDisplay, colors: &ThemeColors) -> io::Result<()> {
    let mut stdout = io::stdout();
    let terminal_width: usize = 80;

    let title = format!("Forecast for {}", display.location);
    let title_padding = (terminal_width.saturating_sub(title.len())) / 2;

    // Title line
    stdout.queue(SetForegroundColor(colors.text))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("{}{}\r\n", " ".repeat(title_padding), title)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(Print("\r\n"))?;

    let unit = if display.use_fahrenheit { "°F" } else { "°C" };

    for row in &display.daily {
        let icon = crate::weather::weather_icon(row.weather_code);

        // Day name color
        match row.day_name.as_str() {
            "Today" => {
                stdout.queue(SetForegroundColor(colors.temp_high))?;
            }
            "Tomorrow" => {
                stdout.queue(SetForegroundColor(colors.temp_low))?;
            }
            _ => {
                stdout.queue(SetForegroundColor(colors.text))?;
                stdout.queue(SetAttribute(Attribute::Bold))?;
            }
        }
        stdout.queue(Print(format!("  {:<9}", row.day_name)))?;
        stdout.queue(SetAttribute(Attribute::Reset))?;

        // Icon
        stdout.queue(Print(format!("{} ", icon)))?;

        // High temp
        stdout.queue(SetForegroundColor(colors.temp_high))?;
        stdout.queue(Print(format!("{}{}", row.temp_high as i32, unit)))?;

        // Separator
        stdout.queue(SetForegroundColor(colors.text))?;
        stdout.queue(Print("/"))?;

        // Low temp
        stdout.queue(SetForegroundColor(colors.temp_low))?;
        stdout.queue(Print(format!("{}{}", row.temp_low as i32, unit)))?;

        // Precip chance
        let precip_padding = 4; // align precip column
        if row.precip_chance >= 80 {
            stdout.queue(SetForegroundColor(colors.precip_high))?;
        } else if row.precip_chance >= 50 {
            stdout.queue(SetForegroundColor(colors.precip_medium))?;
        } else {
            stdout.queue(SetForegroundColor(colors.dimmed))?;
        }
        stdout.queue(Print(format!(
            "{:>precip_padding$}☂ {}%\r\n",
            "",
            row.precip_chance,
            precip_padding = precip_padding
        )))?;
    }

    stdout.queue(Print("\r\n"))?;
    stdout.flush()
}

/// Renders the hourly forecast breakdown.
///
/// Output format:
/// ```text
///         Today's Forecast — Oslo
///
///   9am   ☀️  11°    ☂ 5%
///   12pm  🌤 14°    ☂ 10%
///   3pm   🌧 13°    ☂ 70%    Rain starting
/// ```
pub fn render_forecast_hourly(display: &ForecastDisplay, colors: &ThemeColors) -> io::Result<()> {
    let mut stdout = io::stdout();
    let terminal_width: usize = 80;

    if display.hourly.is_empty() {
        return Ok(());
    }

    let title = format!("Today's Forecast — {}", display.location);
    let title_padding = (terminal_width.saturating_sub(title.len())) / 2;

    // Title
    stdout.queue(SetForegroundColor(colors.text))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("{}{}\r\n", " ".repeat(title_padding), title)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(Print("\r\n"))?;

    let unit = if display.use_fahrenheit { "°F" } else { "°C" };

    for entry in &display.hourly {
        let icon = crate::weather::weather_icon(entry.weather_code);

        // Time
        stdout.queue(SetForegroundColor(colors.text))?;
        stdout.queue(Print(format!("  {:<6}", entry.time)))?;

        // Icon
        stdout.queue(Print(format!("{} ", icon)))?;

        // Temperature
        stdout.queue(SetForegroundColor(colors.temp_high))?;
        stdout.queue(Print(format!("{}{}", entry.temperature as i32, unit)))?;

        // Precip chance
        if entry.precip_chance >= 80 {
            stdout.queue(SetForegroundColor(colors.precip_high))?;
        } else if entry.precip_chance >= 50 {
            stdout.queue(SetForegroundColor(colors.precip_medium))?;
        } else {
            stdout.queue(SetForegroundColor(colors.dimmed))?;
        }
        stdout.queue(Print(format!("    ☂ {}%", entry.precip_chance)))?;

        // Annotation
        if let Some(ref annotation) = entry.annotation {
            stdout.queue(SetForegroundColor(colors.dimmed))?;
            stdout.queue(Print(format!("    {}", annotation)))?;
        }

        stdout.queue(Print("\r\n"))?;
    }

    stdout.queue(Print("\r\n"))?;
    stdout.flush()
}

/// Renders a static demo weather display using the given theme's colors.
pub fn render_preview_theme(name: &str, colors: &ThemeColors) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(SetForegroundColor(colors.text))?;
    stdout.queue(Print("       \u{2600}\u{FE0F} 14\u{00B0}C Demo City\r\n"))?;
    stdout.queue(SetForegroundColor(colors.dimmed))?;
    stdout.queue(Print("     Feels 11\u{00B0}C\r\n"))?;
    stdout.queue(SetForegroundColor(colors.temp_high))?;
    stdout.queue(Print("     High 17\u{00B0}C"))?;
    stdout.queue(SetForegroundColor(colors.text))?;
    stdout.queue(Print(" \u{00B7} "))?;
    stdout.queue(SetForegroundColor(colors.temp_low))?;
    stdout.queue(Print("Low 8\u{00B0}C\r\n"))?;
    stdout.queue(SetForegroundColor(colors.text))?;
    stdout.queue(Print("     Clear skies and warm\r\n\r\n"))?;

    // Forecast demo
    stdout.queue(SetForegroundColor(colors.text))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print("       Forecast preview\r\n\r\n"))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    stdout.queue(SetForegroundColor(colors.temp_high))?;
    stdout.queue(Print(
        "  Today     \u{2600}\u{FE0F}  17\u{00B0}C/8\u{00B0}C",
    ))?;
    stdout.queue(SetForegroundColor(colors.dimmed))?;
    stdout.queue(Print("    \u{2602} 5%\r\n"))?;

    stdout.queue(SetForegroundColor(colors.temp_low))?;
    stdout.queue(Print("  Tomorrow  \u{26C5} 15\u{00B0}C/7\u{00B0}C"))?;
    stdout.queue(SetForegroundColor(colors.precip_medium))?;
    stdout.queue(Print("    \u{2602} 60%\r\n"))?;

    stdout.queue(SetForegroundColor(colors.text))?;
    stdout.queue(Print(
        "  Wed       \u{1F327}\u{FE0F} 12\u{00B0}C/5\u{00B0}C",
    ))?;
    stdout.queue(SetForegroundColor(colors.precip_high))?;
    stdout.queue(Print("    \u{2602} 85%\r\n"))?;

    stdout.queue(Print("\r\n"))?;
    stdout.queue(SetForegroundColor(colors.dimmed))?;
    stdout.queue(Print(format!("  Theme: {}\r\n", name)))?;
    stdout.queue(Print("\r\n"))?;
    stdout.flush()
}

/// Outputs a compact one-line forecast for shell prompts/status bars.
///
/// Format: `[icon] [high]/[low] ☂ [precip]% [day_name]`
pub fn output_ambient_forecast(display: &ForecastDisplay) -> io::Result<()> {
    // Use tomorrow's data if available, otherwise today's
    let row = display.daily.get(1).or(display.daily.first());
    match row {
        Some(row) => {
            let icon = crate::weather::weather_icon(row.weather_code);
            let unit = if display.use_fahrenheit { "°F" } else { "°C" };
            println!(
                "{} {}{}/{}{} ☂ {}% {}",
                icon,
                row.temp_high as i32,
                unit,
                row.temp_low as i32,
                unit,
                row.precip_chance,
                row.day_name
            );
        }
        None => println!("No forecast data available"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forecast::{DailyRow, HourlyEntry};
    use crate::weather::WeatherDisplay;

    fn default_colors() -> ThemeColors {
        crate::theme::default_colors()
    }

    #[test]
    fn test_weather_display_construction() {
        let weather = WeatherDisplay {
            temperature: 14.0,
            apparent_temperature: 11.0,
            temp_max: 17.0,
            temp_min: 8.0,
            weather_code: 0,
            location: "Oslo".to_string(),
            use_fahrenheit: false,
        };
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
            use_fahrenheit: false,
        };
        let result = render_weather(&weather, "Clear", &default_colors());
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_error() {
        let result = render_error("Network error");
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_forecast() {
        let display = ForecastDisplay {
            location: "Oslo".to_string(),
            use_fahrenheit: false,
            daily: vec![
                DailyRow {
                    day_name: "Today".to_string(),
                    weather_code: 0,
                    temp_high: 17.0,
                    temp_low: 8.0,
                    precip_chance: 5,
                },
                DailyRow {
                    day_name: "Tomorrow".to_string(),
                    weather_code: 3,
                    temp_high: 15.0,
                    temp_low: 7.0,
                    precip_chance: 75,
                },
            ],
            hourly: vec![],
        };
        let result = render_forecast(&display, &default_colors());
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_forecast_hourly() {
        let display = ForecastDisplay {
            location: "Oslo".to_string(),
            use_fahrenheit: false,
            daily: vec![],
            hourly: vec![
                HourlyEntry {
                    time: "9am".to_string(),
                    weather_code: 0,
                    temperature: 11.0,
                    precip_chance: 5,
                    annotation: None,
                },
                HourlyEntry {
                    time: "3pm".to_string(),
                    weather_code: 61,
                    temperature: 13.0,
                    precip_chance: 70,
                    annotation: Some("Rain starting".to_string()),
                },
            ],
        };
        let result = render_forecast_hourly(&display, &default_colors());
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_forecast_hourly_empty() {
        let display = ForecastDisplay {
            location: "Oslo".to_string(),
            use_fahrenheit: false,
            daily: vec![],
            hourly: vec![],
        };
        let result = render_forecast_hourly(&display, &default_colors());
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_ambient_forecast() {
        let display = ForecastDisplay {
            location: "Oslo".to_string(),
            use_fahrenheit: false,
            daily: vec![
                DailyRow {
                    day_name: "Today".to_string(),
                    weather_code: 0,
                    temp_high: 17.0,
                    temp_low: 8.0,
                    precip_chance: 5,
                },
                DailyRow {
                    day_name: "Wed".to_string(),
                    weather_code: 3,
                    temp_high: 15.0,
                    temp_low: 7.0,
                    precip_chance: 70,
                },
            ],
            hourly: vec![],
        };
        let result = output_ambient_forecast(&display);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_ambient_forecast_empty() {
        let display = ForecastDisplay {
            location: "Oslo".to_string(),
            use_fahrenheit: false,
            daily: vec![],
            hourly: vec![],
        };
        let result = output_ambient_forecast(&display);
        assert!(result.is_ok());
    }

    // --- Alert rendering tests ---

    #[test]
    fn test_render_alert_line_warning() {
        let alert = crate::alerts::Alert {
            severity: crate::alerts::AlertSeverity::Warning,
            event: "Tornado Warning".to_string(),
            expires_time: Some("6:00 PM".to_string()),
        };
        let result = render_alert_line(&alert);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_alert_line_watch() {
        let alert = crate::alerts::Alert {
            severity: crate::alerts::AlertSeverity::Watch,
            event: "Flash Flood Watch".to_string(),
            expires_time: Some("8:00 PM".to_string()),
        };
        let result = render_alert_line(&alert);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_alert_line_advisory() {
        let alert = crate::alerts::Alert {
            severity: crate::alerts::AlertSeverity::Advisory,
            event: "Wind Advisory".to_string(),
            expires_time: Some("12:00 AM".to_string()),
        };
        let result = render_alert_line(&alert);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_alert_line_no_expiry() {
        let alert = crate::alerts::Alert {
            severity: crate::alerts::AlertSeverity::Warning,
            event: "Special Weather Statement".to_string(),
            expires_time: None,
        };
        let result = render_alert_line(&alert);
        assert!(result.is_ok());
    }
}
