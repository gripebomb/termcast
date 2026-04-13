//! Data types for Open-Meteo weather API responses.
//!
//! Uses the Open-Meteo API which provides free weather data without API keys.

use serde::Deserialize;

/// Response from the Open-Meteo weather API.
#[derive(Debug, Clone, Deserialize)]
pub struct WeatherResponse {
    /// Current weather conditions.
    #[serde(rename = "current_weather")]
    pub current_weather: CurrentWeather,

    /// Daily forecast data.
    #[serde(rename = "daily", default)]
    pub daily: Option<DailyWeather>,
}

/// Current weather conditions.
#[derive(Debug, Clone, Deserialize)]
pub struct CurrentWeather {
    /// Current temperature in Celsius.
    pub temperature: f64,

    /// Weather code based on WMO standards.
    #[serde(rename = "weathercode")]
    pub weather_code: u32,

    /// Apparent temperature (feels like) in Celsius.
    #[serde(rename = "apparent_temperature", default)]
    pub apparent_temperature: f64,

    /// Time of the current weather reading.
    #[serde(rename = "time", default)]
    #[allow(dead_code)]
    pub time: String,
}

/// Daily forecast data.
#[derive(Debug, Clone, Deserialize)]
pub struct DailyWeather {
    /// Dates for the forecast.
    #[allow(dead_code)]
    pub time: Vec<String>,

    /// Maximum temperatures for each day.
    #[serde(rename = "temperature_2m_max")]
    pub temperature_max: Vec<f64>,

    /// Minimum temperatures for each day.
    #[serde(rename = "temperature_2m_min")]
    pub temperature_min: Vec<f64>,

    /// Weather codes for each day.
    #[serde(rename = "weathercode")]
    #[allow(dead_code)]
    pub weather_code: Vec<u32>,
}

/// Display-ready weather data for rendering.
#[derive(Debug, Clone)]
pub struct WeatherDisplay {
    /// Current temperature in Celsius.
    pub temperature: f64,

    /// Feels-like temperature in Celsius.
    pub apparent_temperature: f64,

    /// Maximum temperature for today.
    pub temp_max: f64,

    /// Minimum temperature for today.
    pub temp_min: f64,

    /// WMO weather code.
    pub weather_code: u32,

    /// Location name.
    pub location: String,
}

impl WeatherDisplay {
    /// Creates a new WeatherDisplay from API response and location name.
    pub fn from_response(response: &WeatherResponse, location: &str) -> Self {
        let temp_max = response
            .daily
            .as_ref()
            .and_then(|d| d.temperature_max.first())
            .copied()
            .unwrap_or(response.current_weather.temperature);

        let temp_min = response
            .daily
            .as_ref()
            .and_then(|d| d.temperature_min.first())
            .copied()
            .unwrap_or(response.current_weather.temperature);

        Self {
            temperature: response.current_weather.temperature,
            apparent_temperature: response.current_weather.apparent_temperature,
            temp_max,
            temp_min,
            weather_code: response.current_weather.weather_code,
            location: location.to_string(),
        }
    }
}

/// Weather code to icon mapping based on WMO standards.
pub fn weather_icon(code: u32) -> &'static str {
    match code {
        0 => "☀️",
        1..=3 => "🌤",
        45..=48 => "🌫",
        51..=67 => "🌧",
        71..=77 => "❄",
        80..=82 => "🌦",
        95..=99 => "⛈",
        _ => "☁",
    }
}

/// Weather code to description mapping based on WMO standards.
pub fn weather_description(code: u32) -> &'static str {
    match code {
        0 => "Clear",
        1 => "Mainly Clear",
        2 => "Partly Cloudy",
        3 => "Overcast",
        45 => "Foggy",
        48 => "Depositing Rime Fog",
        51 => "Light Drizzle",
        53 => "Moderate Drizzle",
        55 => "Dense Drizzle",
        56 => "Light Freezing Drizzle",
        57 => "Dense Freezing Drizzle",
        61 => "Slight Rain",
        63 => "Moderate Rain",
        65 => "Heavy Rain",
        66 => "Light Freezing Rain",
        67 => "Heavy Freezing Rain",
        71 => "Slight Snow",
        73 => "Moderate Snow",
        75 => "Heavy Snow",
        77 => "Snow Grains",
        80 => "Slight Rain Showers",
        81 => "Moderate Rain Showers",
        82 => "Violent Rain Showers",
        85 => "Slight Snow Showers",
        86 => "Heavy Snow Showers",
        95 => "Thunderstorm",
        96 => "Thunderstorm with Light Hail",
        99 => "Thunderstorm with Heavy Hail",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_weather_response() {
        let json = r#"{
            "current_weather": {
                "temperature": 14.0,
                "weathercode": 3,
                "apparent_temperature": 11.0,
                "time": "2024-01-15T12:00"
            },
            "daily": {
                "time": ["2024-01-15"],
                "temperature_2m_max": [17.0],
                "temperature_2m_min": [8.0],
                "weathercode": [2]
            }
        }"#;
        let response: WeatherResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.current_weather.temperature, 14.0);
        assert_eq!(response.current_weather.weather_code, 3);
        assert_eq!(response.current_weather.apparent_temperature, 11.0);
    }

    #[test]
    fn test_weather_display_from_response() {
        let json = r#"{
            "current_weather": {
                "temperature": 14.0,
                "weathercode": 0,
                "apparent_temperature": 11.0
            },
            "daily": {
                "time": ["2024-01-15"],
                "temperature_2m_max": [17.0],
                "temperature_2m_min": [8.0],
                "weathercode": [0]
            }
        }"#;
        let response: WeatherResponse = serde_json::from_str(json).unwrap();
        let display = WeatherDisplay::from_response(&response, "Oslo");
        assert_eq!(display.temperature, 14.0);
        assert_eq!(display.apparent_temperature, 11.0);
        assert_eq!(display.temp_max, 17.0);
        assert_eq!(display.temp_min, 8.0);
        assert_eq!(display.location, "Oslo");
    }

    #[test]
    fn test_weather_icon_mapping() {
        assert_eq!(weather_icon(0), "☀️");
        assert_eq!(weather_icon(2), "🌤");
        assert_eq!(weather_icon(45), "🌫");
        assert_eq!(weather_icon(61), "🌧");
        assert_eq!(weather_icon(73), "❄");
        assert_eq!(weather_icon(81), "🌦");
        assert_eq!(weather_icon(95), "⛈");
        assert_eq!(weather_icon(100), "☁");
    }

    #[test]
    fn test_weather_description() {
        assert_eq!(weather_description(0), "Clear");
        assert_eq!(weather_description(2), "Partly Cloudy");
        assert_eq!(weather_description(61), "Slight Rain");
        assert_eq!(weather_description(95), "Thunderstorm");
    }
}
