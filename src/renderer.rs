//! Terminal renderer for weather output.
//!
//! Uses crossterm for cross-platform terminal styling and ANSI output.

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
    stdout.queue(Print(format!(
        "{}{}\r\n",
        " ".repeat(title_padding),
        title
    )))?;
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
pub fn render_forecast_hourly(
    display: &ForecastDisplay,
    colors: &ThemeColors,
) -> io::Result<()> {
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
    stdout.queue(Print(format!(
        "{}{}\r\n",
        " ".repeat(title_padding),
        title
    )))?;
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
}
