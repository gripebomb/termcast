//! HTTP client for weather and geolocation APIs.
//!
//! Uses Open-Meteo for weather data and ipapi.co for IP-based geolocation.
//! Both APIs are free and don't require API keys.

use crate::errors::AppError;
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
    /// Returns a tuple of (latitude, longitude, city_name).
    pub async fn get_location(&self) -> Result<(f64, f64, String), AppError> {
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

        Ok((geo.latitude, geo.longitude, geo.location_name()))
    }

    /// Fetches weather data from Open-Meteo API.
    ///
    /// Returns a WeatherDisplay with all data needed for rendering.
    pub async fn get_weather(
        &self,
        latitude: f64,
        longitude: f64,
        location: &str,
    ) -> Result<WeatherDisplay, AppError> {
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto",
            latitude, longitude
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

        Ok(WeatherDisplay::from_response(&weather, location))
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
}
