//! Data types for IP geolocation API responses.
//!
//! Uses the ipapi.co API which returns location data based on client IP.

use serde::Deserialize;

/// Response from the ipapi.co geolocation API.
#[derive(Debug, Clone, Deserialize)]
pub struct GeoResponse {
    /// Latitude coordinate.
    #[serde(rename = "latitude")]
    pub latitude: f64,

    /// Longitude coordinate.
    #[serde(rename = "longitude")]
    pub longitude: f64,

    /// City name.
    #[serde(rename = "city", default)]
    pub city: String,

    /// Country name.
    #[serde(rename = "country_name", default)]
    pub country: String,

    /// ISO 3166-1 alpha-2 country code (e.g., "US", "GB", "NO").
    #[serde(rename = "country_code", default)]
    pub country_code: String,
}

impl GeoResponse {
    /// Returns a formatted location string.
    pub fn location_name(&self) -> String {
        if self.city.is_empty() {
            self.country.clone()
        } else if self.country.is_empty() {
            self.city.clone()
        } else {
            format!("{}, {}", self.city, self.country)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_geolocation_response() {
        let json = r#"{"latitude": 59.9139, "longitude": 10.7522, "city": "Oslo", "country_name": "Norway"}"#;
        let response: GeoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.latitude, 59.9139);
        assert_eq!(response.longitude, 10.7522);
        assert_eq!(response.city, "Oslo");
        assert_eq!(response.country, "Norway");
    }

    #[test]
    fn test_location_name_with_city() {
        let response = GeoResponse {
            latitude: 40.7128,
            longitude: -74.0060,
            city: "New York".to_string(),
            country: "United States".to_string(),
            country_code: "US".to_string(),
        };
        assert_eq!(response.location_name(), "New York, United States");
    }

    #[test]
    fn test_location_name_city_only() {
        let response = GeoResponse {
            latitude: 0.0,
            longitude: 0.0,
            city: "Unknown".to_string(),
            country: "".to_string(),
            country_code: "".to_string(),
        };
        assert_eq!(response.location_name(), "Unknown");
    }
}
