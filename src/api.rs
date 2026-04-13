//! HTTP client for weather and geolocation APIs.
//!
//! Uses Open-Meteo for weather data and ipapi.co for IP-based geolocation.
//! Both APIs are free and don't require API keys.

use crate::errors::AppError;
use crate::forecast::{ForecastDisplay, ForecastResponse};
use crate::geolocation::GeoResponse;
use crate::weather::{WeatherDisplay, WeatherResponse};
use serde::Deserialize;

/// Client for making API requests with a 10-second timeout.
#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    /// Creates a new API client with a 10-second timeout.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        Self { inner: client }
    }

    /// Fetches location based on client IP using ipapi.co.
    ///
    /// Returns a tuple of (latitude, longitude, city_name, use_fahrenheit).
    pub async fn get_location(&self) -> Result<(f64, f64, String, bool), AppError> {
        const URL: &str = "https://ipapi.co/json/";

        let response = self
            .inner
            .get(URL)
            .header("User-Agent", "termcast/0.1.0")
            .send()
            .await
            .map_err(|e| AppError::network(URL, e))?;

        if !response.status().is_success() {
            return Err(AppError::weather(
                URL,
                &format!("Status: {}", response.status()),
            ));
        }

        let text = response
            .text()
            .await
            .map_err(|e| AppError::network(URL, e))?;
        let geo: GeoResponse =
            serde_json::from_str(&text).map_err(|e| AppError::parse(URL, "geolocation", e))?;

        let use_fahrenheit = geo.country_code == "US";
        Ok((
            geo.latitude,
            geo.longitude,
            geo.location_name(),
            use_fahrenheit,
        ))
    }

    /// Fetches weather data from Open-Meteo API.
    ///
    /// Returns a WeatherDisplay with all data needed for rendering.
    pub async fn get_weather(
        &self,
        latitude: f64,
        longitude: f64,
        location: &str,
        use_fahrenheit: bool,
    ) -> Result<WeatherDisplay, AppError> {
        let unit = if use_fahrenheit {
            "fahrenheit"
        } else {
            "celsius"
        };
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true&daily=weather_code,temperature_2m_max,temperature_2m_min&temperature_unit={}&timezone=auto",
            latitude, longitude, unit
        );

        let response = self
            .inner
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::network(&url, e))?;

        if !response.status().is_success() {
            return Err(AppError::weather(
                &url,
                &format!("Status: {}", response.status()),
            ));
        }

        let text = response
            .text()
            .await
            .map_err(|e| AppError::network(&url, e))?;
        let weather: WeatherResponse =
            serde_json::from_str(&text).map_err(|e| AppError::parse(&url, "weather", e))?;

        Ok(WeatherDisplay::from_response(
            &weather,
            location,
            use_fahrenheit,
        ))
    }

    /// Fetches multi-day forecast data from Open-Meteo API.
    ///
    /// Returns a ForecastDisplay with daily rows and optionally filtered hourly entries.
    pub async fn get_forecast(
        &self,
        latitude: f64,
        longitude: f64,
        location: &str,
        use_fahrenheit: bool,
        days: u32,
        include_hourly: bool,
    ) -> Result<ForecastDisplay, AppError> {
        let unit = if use_fahrenheit {
            "fahrenheit"
        } else {
            "celsius"
        };

        let mut url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max&temperature_unit={}&timezone=auto&forecast_days={}",
            latitude, longitude, unit, days
        );

        if include_hourly {
            url.push_str("&hourly=temperature_2m,precipitation_probability,weather_code");
        }

        let response = self
            .inner
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::network(&url, e))?;

        if !response.status().is_success() {
            return Err(AppError::weather(
                &url,
                &format!("Status: {}", response.status()),
            ));
        }

        let text = response
            .text()
            .await
            .map_err(|e| AppError::network(&url, e))?;
        let forecast: ForecastResponse =
            serde_json::from_str(&text).map_err(|e| AppError::parse(&url, "forecast", e))?;

        let daily = forecast.daily.to_daily_rows();
        let hourly = forecast
            .hourly
            .map(|h| h.filter_notable())
            .unwrap_or_default();

        Ok(ForecastDisplay {
            location: location.to_string(),
            use_fahrenheit,
            daily,
            hourly,
        })
    }

    /// Geocodes a location name to coordinates using Open-Meteo geocoding.
    ///
    /// Returns a tuple of (latitude, longitude, location_name).
    pub async fn geocode_location(&self, location: &str) -> Result<(f64, f64, String), AppError> {
        let url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
            urlencoding::encode(location)
        );

        let response = self
            .inner
            .get(&url)
            .header("User-Agent", "termcast/0.1.0")
            .send()
            .await
            .map_err(|e| AppError::network(&url, e))?;

        if !response.status().is_success() {
            return Err(AppError::weather(
                &url,
                &format!("Status: {}", response.status()),
            ));
        }

        let text = response
            .text()
            .await
            .map_err(|e| AppError::network(&url, e))?;

        #[derive(Debug, Deserialize)]
        struct GeoResult {
            results: Option<Vec<GeoResultItem>>,
        }

        #[derive(Debug, Deserialize)]
        struct GeoResultItem {
            latitude: f64,
            longitude: f64,
            name: String,
            country: Option<String>,
            admin1: Option<String>,
        }

        let geo_result: GeoResult =
            serde_json::from_str(&text).map_err(|e| AppError::parse(&url, "geocode", e))?;

        let item = geo_result
            .results
            .and_then(|mut v| v.pop())
            .ok_or_else(|| AppError::geolocation(format!("Location '{}' not found", location)))?;

        let location_name = match (&item.admin1, &item.country) {
            (Some(admin), Some(country)) => format!("{}, {}, {}", item.name, admin, country),
            (Some(admin), None) => format!("{}, {}", item.name, admin),
            (None, Some(country)) => format!("{}, {}", item.name, country),
            (None, None) => item.name.clone(),
        };

        Ok((item.latitude, item.longitude, location_name))
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = Client::new();
    }

    #[tokio::test]
    async fn test_get_forecast_daily_only() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let tomorrow = (chrono::Local::now() + chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        let response_body = serde_json::json!({
            "daily": {
                "time": [today, tomorrow],
                "weather_code": [0, 3],
                "temperature_2m_max": [17.0, 15.0],
                "temperature_2m_min": [8.0, 7.0],
                "precipitation_probability_max": [5, 10]
            }
        });

        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        // Override the base URL by constructing the request manually for testing
        let url = format!(
            "{}/v1/forecast?latitude=59.91&longitude=10.75&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max&temperature_unit=celsius&timezone=auto&forecast_days=2",
            mock_server.uri()
        );

        let response = client.inner.get(&url).send().await.unwrap();
        let text = response.text().await.unwrap();
        let forecast: ForecastResponse = serde_json::from_str(&text).unwrap();
        assert_eq!(forecast.daily.time.len(), 2);
        assert_eq!(forecast.daily.temperature_max, vec![17.0, 15.0]);

        let display = ForecastDisplay {
            location: "Oslo".to_string(),
            use_fahrenheit: false,
            daily: forecast.daily.to_daily_rows(),
            hourly: vec![],
        };

        assert_eq!(display.daily.len(), 2);
        assert_eq!(display.daily[0].day_name, "Today");
        assert_eq!(display.daily[1].day_name, "Tomorrow");
    }

    #[tokio::test]
    async fn test_get_forecast_with_hourly() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let response_body = serde_json::json!({
            "daily": {
                "time": [today],
                "weather_code": [0],
                "temperature_2m_max": [17.0],
                "temperature_2m_min": [8.0],
                "precipitation_probability_max": [5]
            },
            "hourly": {
                "time": [format!("{}T00:00", today), format!("{}T12:00", today)],
                "temperature_2m": [10.0, 15.0],
                "precipitation_probability": [5, 10],
                "weather_code": [0, 1]
            }
        });

        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let url = format!(
            "{}/v1/forecast?latitude=59.91&longitude=10.75&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max&hourly=temperature_2m,precipitation_probability,weather_code&temperature_unit=celsius&timezone=auto&forecast_days=1",
            mock_server.uri()
        );

        let response = client.inner.get(&url).send().await.unwrap();
        let text = response.text().await.unwrap();
        let forecast: ForecastResponse = serde_json::from_str(&text).unwrap();

        assert!(forecast.hourly.is_some());
        let hourly = forecast.hourly.unwrap();
        assert_eq!(hourly.temperature, vec![10.0, 15.0]);
    }
}
