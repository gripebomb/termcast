//! Data types and logic for multi-day and hourly weather forecasts.
//!
//! Uses the Open-Meteo `/v1/forecast` endpoint with daily and hourly parameters.

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, Timelike, Weekday};
use serde::Deserialize;

/// Response from the Open-Meteo forecast endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ForecastResponse {
    /// Daily forecast data.
    #[serde(rename = "daily")]
    pub daily: DailyForecast,

    /// Hourly forecast data (only present when hourly params requested).
    #[serde(rename = "hourly", default)]
    pub hourly: Option<HourlyForecast>,
}

/// Daily forecast data from Open-Meteo.
#[derive(Debug, Clone, Deserialize)]
pub struct DailyForecast {
    /// Dates for each forecast day (e.g. "2024-01-15").
    pub time: Vec<String>,

    /// WMO weather codes for each day.
    #[serde(rename = "weather_code", default)]
    pub weather_code: Vec<u32>,

    /// Maximum temperatures for each day.
    #[serde(rename = "temperature_2m_max")]
    pub temperature_max: Vec<f64>,

    /// Minimum temperatures for each day.
    #[serde(rename = "temperature_2m_min")]
    pub temperature_min: Vec<f64>,

    /// Maximum precipitation probability for each day (0-100%).
    #[serde(rename = "precipitation_probability_max", default)]
    pub precipitation_probability: Vec<u32>,
}

/// Hourly forecast data from Open-Meteo.
#[derive(Debug, Clone, Deserialize)]
pub struct HourlyForecast {
    /// Timestamps for each hour (e.g. "2024-01-15T00:00").
    pub time: Vec<String>,

    /// Temperature for each hour.
    #[serde(rename = "temperature_2m")]
    pub temperature: Vec<f64>,

    /// Precipitation probability for each hour (0-100%).
    #[serde(rename = "precipitation_probability", default)]
    pub precipitation_probability: Vec<u32>,

    /// WMO weather codes for each hour.
    #[serde(rename = "weather_code", default)]
    pub weather_code: Vec<u32>,
}

/// Display-ready daily forecast row.
#[derive(Debug, Clone)]
pub struct DailyRow {
    /// Day name: "Today", "Tomorrow", or short weekday name ("Mon", "Tue").
    pub day_name: String,

    /// WMO weather code for icon mapping.
    pub weather_code: u32,

    /// Day's high temperature.
    pub temp_high: f64,

    /// Day's low temperature.
    pub temp_low: f64,

    /// Maximum precipitation probability (0-100%).
    pub precip_chance: u32,
}

/// Display-ready hourly entry after notable-change filtering.
#[derive(Debug, Clone)]
pub struct HourlyEntry {
    /// Formatted time string (e.g. "2pm").
    pub time: String,

    /// WMO weather code for icon mapping.
    pub weather_code: u32,

    /// Temperature at this hour.
    pub temperature: f64,

    /// Precipitation probability (0-100%).
    pub precip_chance: u32,

    /// Optional annotation for change descriptions (e.g. "Rain starting").
    pub annotation: Option<String>,
}

/// Complete forecast data ready for rendering.
#[derive(Debug, Clone)]
pub struct ForecastDisplay {
    /// Location name for display.
    pub location: String,

    /// Whether to display temperatures in Fahrenheit.
    pub use_fahrenheit: bool,

    /// Daily forecast rows.
    pub daily: Vec<DailyRow>,

    /// Filtered hourly entries (empty if not requested).
    pub hourly: Vec<HourlyEntry>,
}

impl DailyForecast {
    /// Converts raw daily data to display-ready rows with computed day names.
    ///
    /// Day names: "Today" for the first entry, "Tomorrow" for the second,
    /// then short weekday names ("Mon", "Tue", etc.) for subsequent days.
    pub fn to_daily_rows(&self) -> Vec<DailyRow> {
        let today = Local::now().date_naive();

        self.time
            .iter()
            .enumerate()
            .map(|(i, date_str)| {
                let day_name = compute_day_name(date_str, i, today);
                DailyRow {
                    day_name,
                    weather_code: self.weather_code.get(i).copied().unwrap_or(0),
                    temp_high: self.temperature_max.get(i).copied().unwrap_or(0.0),
                    temp_low: self.temperature_min.get(i).copied().unwrap_or(0.0),
                    precip_chance: self.precipitation_probability.get(i).copied().unwrap_or(0),
                }
            })
            .collect()
    }
}

impl HourlyForecast {
    /// Filters hourly data to show only notable weather changes.
    ///
    /// Rules:
    /// 1. Always include first and last hour of today for context
    /// 2. Include hours where precip probability changes by > 20pp from previous included
    /// 3. Include hours where temperature changes by > 5 degrees from previous included
    /// 4. Include the hour with the day's maximum precipitation
    /// 5. Merge entries < 2 hours apart (keep the more notable one)
    pub fn filter_notable(&self) -> Vec<HourlyEntry> {
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        // Filter to today's hours only
        let today_indices: Vec<usize> = self
            .time
            .iter()
            .enumerate()
            .filter(|(_, t)| t.starts_with(&today_str))
            .map(|(i, _)| i)
            .collect();

        if today_indices.is_empty() {
            return vec![];
        }

        let first_idx = today_indices[0];
        let last_idx = *today_indices.last().unwrap();

        // Find the index of max precipitation
        let max_precip_idx = today_indices
            .iter()
            .max_by_key(|&&i| self.precipitation_probability.get(i).copied().unwrap_or(0))
            .copied()
            .unwrap_or(first_idx);

        // Always include first, last, and max-precip hours
        let mut notable_indices = vec![first_idx];
        if last_idx != first_idx {
            notable_indices.push(last_idx);
        }
        if max_precip_idx != first_idx && max_precip_idx != last_idx {
            notable_indices.push(max_precip_idx);
        }

        // Scan for notable changes from the previous included entry
        let mut prev_temp = self.temperature.get(first_idx).copied().unwrap_or(0.0);
        let mut prev_precip = self
            .precipitation_probability
            .get(first_idx)
            .copied()
            .unwrap_or(0);

        for &idx in &today_indices {
            let temp = self.temperature.get(idx).copied().unwrap_or(0.0);
            let precip = self
                .precipitation_probability
                .get(idx)
                .copied()
                .unwrap_or(0);

            let temp_delta = (temp - prev_temp).abs();
            let precip_delta = (precip as i32 - prev_precip as i32).unsigned_abs();

            if temp_delta > 5.0 || precip_delta > 20 {
                if !notable_indices.contains(&idx) {
                    notable_indices.push(idx);
                }
                prev_temp = temp;
                prev_precip = precip;
            }
        }

        // Sort by index (chronological order)
        notable_indices.sort();

        // Build entries with annotations
        let mut entries: Vec<HourlyEntry> = notable_indices
            .iter()
            .map(|&idx| {
                let time_str = self.time.get(idx).map(|t| t.as_str()).unwrap_or("");
                let formatted = format_hourly_time(time_str);
                let temp = self.temperature.get(idx).copied().unwrap_or(0.0);
                let precip = self
                    .precipitation_probability
                    .get(idx)
                    .copied()
                    .unwrap_or(0);

                HourlyEntry {
                    time: formatted,
                    weather_code: self.weather_code.get(idx).copied().unwrap_or(0),
                    temperature: temp,
                    precip_chance: precip,
                    annotation: None,
                }
            })
            .collect();

        // Add annotations for notable changes between consecutive entries
        for i in 1..entries.len() {
            let prev_idx = notable_indices[i - 1];
            let curr_idx = notable_indices[i];

            let prev_precip = self
                .precipitation_probability
                .get(prev_idx)
                .copied()
                .unwrap_or(0);
            let curr_precip = self
                .precipitation_probability
                .get(curr_idx)
                .copied()
                .unwrap_or(0);
            let prev_code = self.weather_code.get(prev_idx).copied().unwrap_or(0);
            let curr_code = self.weather_code.get(curr_idx).copied().unwrap_or(0);

            let precip_delta = curr_precip as i32 - prev_precip as i32;
            let is_rain = |code: u32| matches!(code, 51..=67 | 80..=82 | 95..=99);
            let was_raining = is_rain(prev_code);
            let is_raining_now = is_rain(curr_code);

            let annotation = if !was_raining && is_raining_now {
                Some("Rain starting".to_string())
            } else if was_raining && !is_raining_now {
                Some("Rain clearing".to_string())
            } else if precip_delta > 20 {
                Some("Rain increasing".to_string())
            } else if precip_delta < -20 {
                Some("Rain decreasing".to_string())
            } else {
                None
            };

            entries[i].annotation = annotation;
        }

        // Merge entries < 2 hours apart
        entries = merge_close_entries(entries, &notable_indices);

        entries
    }
}

/// Merges hourly entries that are less than 2 hours apart.
/// Keeps the more notable entry (higher precip chance).
fn merge_close_entries(entries: Vec<HourlyEntry>, indices: &[usize]) -> Vec<HourlyEntry> {
    if entries.len() <= 1 {
        return entries;
    }

    let mut result: Vec<HourlyEntry> = vec![entries[0].clone()];
    let mut result_indices: Vec<usize> = vec![indices[0]];

    for i in 1..entries.len() {
        let prev_idx = *result_indices.last().unwrap();
        let curr_idx = indices[i];

        // Check if less than 2 hours apart (indices differ by < 2)
        if curr_idx.saturating_sub(prev_idx) < 2 {
            let prev_precip = result.last().unwrap().precip_chance;
            let curr_precip = entries[i].precip_chance;

            if curr_precip >= prev_precip {
                let last = result.last_mut().unwrap();
                *last = entries[i].clone();
                *result_indices.last_mut().unwrap() = curr_idx;
            }
        } else {
            result.push(entries[i].clone());
            result_indices.push(curr_idx);
        }
    }

    result
}

/// Computes a display day name from a date string and its position.
fn compute_day_name(date_str: &str, index: usize, today: NaiveDate) -> String {
    let parsed = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");

    match parsed {
        Ok(date) if date == today => "Today".to_string(),
        Ok(date) if date == today + chrono::Duration::days(1) => "Tomorrow".to_string(),
        Ok(date) => format_weekday_short(date.weekday()),
        Err(_) => format!("Day {}", index + 1),
    }
}

/// Formats a weekday as a 3-letter abbreviation.
fn format_weekday_short(weekday: Weekday) -> String {
    match weekday {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
    .to_string()
}

/// Formats an hourly timestamp like "2024-01-15T14:00" into "2pm".
fn format_hourly_time(time_str: &str) -> String {
    NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M")
        .map(|dt| {
            let hour = dt.hour();
            match hour {
                0 => "12am".to_string(),
                1..=11 => format!("{}am", hour),
                12 => "12pm".to_string(),
                13..=23 => format!("{}pm", hour - 12),
                _ => format!("{}:00", hour),
            }
        })
        .unwrap_or_else(|_| time_str.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_forecast_response_daily_only() {
        let json = r#"{
            "daily": {
                "time": ["2024-01-15", "2024-01-16"],
                "weather_code": [0, 3],
                "temperature_2m_max": [17.0, 15.0],
                "temperature_2m_min": [8.0, 7.0],
                "precipitation_probability_max": [5, 10]
            }
        }"#;
        let response: ForecastResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.daily.time.len(), 2);
        assert_eq!(response.daily.weather_code, vec![0, 3]);
        assert_eq!(response.daily.temperature_max, vec![17.0, 15.0]);
        assert_eq!(response.daily.temperature_min, vec![8.0, 7.0]);
        assert_eq!(response.daily.precipitation_probability, vec![5, 10]);
        assert!(response.hourly.is_none());
    }

    #[test]
    fn test_parse_forecast_response_with_hourly() {
        let json = r#"{
            "daily": {
                "time": ["2024-01-15"],
                "weather_code": [0],
                "temperature_2m_max": [17.0],
                "temperature_2m_min": [8.0],
                "precipitation_probability_max": [5]
            },
            "hourly": {
                "time": ["2024-01-15T00:00", "2024-01-15T01:00"],
                "temperature_2m": [10.0, 11.0],
                "precipitation_probability": [5, 10],
                "weather_code": [0, 1]
            }
        }"#;
        let response: ForecastResponse = serde_json::from_str(json).unwrap();
        let hourly = response.hourly.unwrap();
        assert_eq!(hourly.time.len(), 2);
        assert_eq!(hourly.temperature, vec![10.0, 11.0]);
        assert_eq!(hourly.precipitation_probability, vec![5, 10]);
        assert_eq!(hourly.weather_code, vec![0, 1]);
    }

    #[test]
    fn test_to_daily_rows_day_names() {
        let today = Local::now().date_naive();
        let daily = DailyForecast {
            time: vec![
                today.format("%Y-%m-%d").to_string(),
                (today + chrono::Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string(),
                (today + chrono::Duration::days(2))
                    .format("%Y-%m-%d")
                    .to_string(),
            ],
            weather_code: vec![0, 3, 61],
            temperature_max: vec![17.0, 15.0, 12.0],
            temperature_min: vec![8.0, 7.0, 6.0],
            precipitation_probability: vec![5, 10, 75],
        };

        let rows = daily.to_daily_rows();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].day_name, "Today");
        assert_eq!(rows[1].day_name, "Tomorrow");
        // Third day should be a 3-letter weekday abbreviation
        assert_eq!(rows[2].day_name.len(), 3);
        assert_eq!(rows[0].weather_code, 0);
        assert_eq!(rows[1].temp_high, 15.0);
        assert_eq!(rows[2].precip_chance, 75);
    }

    #[test]
    fn test_to_daily_rows_with_missing_precipitation() {
        let today = Local::now().date_naive();
        let daily = DailyForecast {
            time: vec![today.format("%Y-%m-%d").to_string()],
            weather_code: vec![0],
            temperature_max: vec![17.0],
            temperature_min: vec![8.0],
            precipitation_probability: vec![],
        };

        let rows = daily.to_daily_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].precip_chance, 0);
    }

    #[test]
    fn test_format_hourly_time() {
        assert_eq!(format_hourly_time("2024-01-15T00:00"), "12am");
        assert_eq!(format_hourly_time("2024-01-15T06:00"), "6am");
        assert_eq!(format_hourly_time("2024-01-15T09:00"), "9am");
        assert_eq!(format_hourly_time("2024-01-15T11:00"), "11am");
        assert_eq!(format_hourly_time("2024-01-15T12:00"), "12pm");
        assert_eq!(format_hourly_time("2024-01-15T13:00"), "1pm");
        assert_eq!(format_hourly_time("2024-01-15T23:00"), "11pm");
    }

    #[test]
    fn test_format_hourly_time_invalid() {
        assert_eq!(format_hourly_time("invalid"), "invalid");
    }

    #[test]
    fn test_compute_day_name_invalid_date() {
        let today = Local::now().date_naive();
        assert_eq!(compute_day_name("not-a-date", 3, today), "Day 4");
    }

    #[test]
    fn test_filter_notable_no_notable_changes() {
        // All hours have similar temps and precip — only first and last should appear
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let hours: Vec<String> = (0..24)
            .map(|h| format!("{}T{:02}:00", today_str, h))
            .collect();
        let temps = vec![10.0; 24];
        let precip = vec![5u32; 24];
        let codes = vec![0u32; 24];

        let hourly = HourlyForecast {
            time: hours,
            temperature: temps,
            precipitation_probability: precip,
            weather_code: codes,
        };

        let entries = hourly.filter_notable();
        // Should have first (0:00) and last (23:00) at minimum
        assert!(!entries.is_empty());
        assert_eq!(entries.first().unwrap().time, "12am");
        assert_eq!(entries.last().unwrap().time, "11pm");
    }

    #[test]
    fn test_filter_notable_large_precip_change() {
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let hours: Vec<String> = (0..24)
            .map(|h| format!("{}T{:02}:00", today_str, h))
            .collect();
        let temps = vec![15.0; 24];
        let mut precip = vec![5u32; 24];
        precip[12] = 80;
        let mut codes = vec![0u32; 24];
        codes[12] = 61;

        let hourly = HourlyForecast {
            time: hours,
            temperature: temps,
            precipitation_probability: precip,
            weather_code: codes,
        };

        let entries = hourly.filter_notable();
        let has_noon = entries.iter().any(|e| e.time == "12pm");
        assert!(
            has_noon,
            "Should include noon entry with notable precip change"
        );
    }

    #[test]
    fn test_filter_notable_large_temp_change() {
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let hours: Vec<String> = (0..24)
            .map(|h| format!("{}T{:02}:00", today_str, h))
            .collect();
        // Sustained temp rise at hour 14 (hours 14-23 all warm)
        let mut temps = vec![10.0; 14];
        temps.extend(vec![20.0; 10]);
        let precip = vec![5u32; 24];
        let codes = vec![0u32; 24];

        let hourly = HourlyForecast {
            time: hours,
            temperature: temps,
            precipitation_probability: precip,
            weather_code: codes,
        };

        let entries = hourly.filter_notable();
        let has_2pm = entries.iter().any(|e| e.time == "2pm");
        assert!(has_2pm, "Should include 2pm entry with notable temp change");
    }

    #[test]
    fn test_filter_notable_empty_data() {
        let hourly = HourlyForecast {
            time: vec![],
            temperature: vec![],
            precipitation_probability: vec![],
            weather_code: vec![],
        };

        let entries = hourly.filter_notable();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_filter_notable_merge_close_entries() {
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let hours: Vec<String> = (0..24)
            .map(|h| format!("{}T{:02}:00", today_str, h))
            .collect();
        let mut temps = vec![10.0; 24];
        temps[5] = 20.0;
        temps[6] = 30.0;
        let precip = vec![5u32; 24];
        let codes = vec![0u32; 24];

        let hourly = HourlyForecast {
            time: hours,
            temperature: temps,
            precipitation_probability: precip,
            weather_code: codes,
        };

        let entries = hourly.filter_notable();
        let has_5am = entries.iter().any(|e| e.time == "5am");
        let has_6am = entries.iter().any(|e| e.time == "6am");
        assert!(
            !(has_5am && has_6am),
            "Should not have both 5am and 6am — they should be merged"
        );
    }

    #[test]
    fn test_filter_notable_rain_annotations() {
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let hours: Vec<String> = (0..24)
            .map(|h| format!("{}T{:02}:00", today_str, h))
            .collect();
        let temps = vec![15.0; 24];
        let mut precip = vec![5u32; 24];
        precip[12] = 80;
        let mut codes = vec![0u32; 24];
        codes[12] = 61;
        codes[13] = 61;
        codes[18] = 0;

        let hourly = HourlyForecast {
            time: hours,
            temperature: temps,
            precipitation_probability: precip,
            weather_code: codes,
        };

        let entries = hourly.filter_notable();
        let has_rain_annotation = entries
            .iter()
            .any(|e| e.annotation.as_ref().is_some_and(|a| a.contains("Rain")));
        assert!(has_rain_annotation, "Should have a rain-related annotation");
    }

    #[test]
    fn test_format_weekday_short() {
        assert_eq!(format_weekday_short(Weekday::Mon), "Mon");
        assert_eq!(format_weekday_short(Weekday::Tue), "Tue");
        assert_eq!(format_weekday_short(Weekday::Wed), "Wed");
        assert_eq!(format_weekday_short(Weekday::Thu), "Thu");
        assert_eq!(format_weekday_short(Weekday::Fri), "Fri");
        assert_eq!(format_weekday_short(Weekday::Sat), "Sat");
        assert_eq!(format_weekday_short(Weekday::Sun), "Sun");
    }
}
