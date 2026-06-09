//! NWS severe weather alert data types, parsing, and formatting.
//!
//! Uses the NWS API `/alerts/active?point={lat},{lon}` endpoint which returns
//! GeoJSON FeatureCollection with alert features.

use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use serde::Deserialize;

/// Response from the NWS alerts endpoint (GeoJSON FeatureCollection).
#[derive(Debug, Clone, Deserialize)]
pub struct NwsAlertsResponse {
    #[serde(default)]
    pub features: Vec<AlertFeature>,
}

/// A single alert feature from the NWS GeoJSON response.
#[derive(Debug, Clone, Deserialize)]
pub struct AlertFeature {
    #[serde(rename = "properties")]
    pub properties: AlertProperties,
}

/// Properties of a weather alert.
#[derive(Debug, Clone, Deserialize)]
pub struct AlertProperties {
    /// Alert event type (e.g., "Tornado Warning", "Flash Flood Watch").
    pub event: String,

    /// Severity level from NWS: "Extreme", "Severe", "Moderate", "Minor".
    pub severity: String,

    /// ISO 8601 expiration timestamp.
    #[serde(rename = "expires")]
    #[serde(default)]
    pub expires: Option<String>,
}

/// Severity level for an alert, ordered from lowest to highest.
/// Used to display the most severe active alert when multiple exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Advisory = 0,
    Watch = 1,
    Warning = 2,
}

/// A display-ready weather alert.
#[derive(Debug, Clone)]
pub struct Alert {
    /// Severity level for color/style decisions.
    pub severity: AlertSeverity,

    /// Alert event name (e.g., "Tornado Warning").
    pub event: String,

    /// Formatted expiration time string (e.g., "6:00 PM"), or None if unavailable.
    pub expires_time: Option<String>,
}

/// Maps an NWS severity string to our AlertSeverity enum.
///
/// NWS values: "Extreme" → Warning, "Severe" → Warning,
/// "Moderate" → Watch, "Minor" → Advisory, unknown → Advisory.
pub fn map_severity(nws_severity: &str) -> AlertSeverity {
    match nws_severity {
        "Extreme" | "Severe" => AlertSeverity::Warning,
        "Moderate" => AlertSeverity::Watch,
        _ => AlertSeverity::Advisory, // Minor and unknown fall here
    }
}

/// Formats an ISO 8601 timestamp for display as "H:MM AM/PM".
///
/// Returns None for malformed or empty strings.
pub fn format_expiry(expires: &Option<String>) -> Option<String> {
    let s = expires.as_ref()?;

    // Parse ISO 8601. NWS uses formats like "2024-01-15T18:00:00-06:00"
    // First try with timezone offset ( DateTime<Utc>)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return format_time(dt.naive_local());
    }

    // Try without timezone (NaiveDateTime)
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return format_time(dt);
    }

    // Try date-only format
    if let Ok(_date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(format!(
            "{}:00 {}",
            12, // noon as fallback
            "PM"
        ));
    }

    None
}

/// Formats a NaiveDateTime as "H:MM AM/PM".
fn format_time(dt: NaiveDateTime) -> Option<String> {
    let hour24 = dt.hour();
    let minute = dt.minute();

    let (hour12, ampm) = match hour24 {
        0 => (12, "AM"),
        1..=11 => (hour24, "AM"),
        12 => (12, "PM"),
        13..=23 => (hour24 - 12, "PM"),
        _ => return None,
    };

    Some(format!("{}:{:02} {}", hour12, minute, ampm))
}

impl Alert {
    /// Creates an Alert from NWS AlertFeature, filtering out expired alerts.
    ///
    /// Returns None if the alert is already expired.
    pub fn from_feature(feature: &AlertFeature, now: DateTime<Utc>) -> Option<Self> {
        // Check if alert has expired
        if let Some(ref expires_str) = feature.properties.expires {
            if let Ok(expires) = DateTime::parse_from_rfc3339(expires_str) {
                if expires.with_timezone(&Utc) <= now {
                    return None; // Already expired
                }
            }
        }

        let severity = map_severity(&feature.properties.severity);
        let expires_time = format_expiry(&feature.properties.expires);
        let event = feature.properties.event.clone();

        Some(Self {
            severity,
            event,
            expires_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Severity mapping tests ---

    #[test]
    fn test_map_severity_extreme() {
        assert_eq!(map_severity("Extreme"), AlertSeverity::Warning);
    }

    #[test]
    fn test_map_severity_severe() {
        assert_eq!(map_severity("Severe"), AlertSeverity::Warning);
    }

    #[test]
    fn test_map_severity_moderate() {
        assert_eq!(map_severity("Moderate"), AlertSeverity::Watch);
    }

    #[test]
    fn test_map_severity_minor() {
        assert_eq!(map_severity("Minor"), AlertSeverity::Advisory);
    }

    #[test]
    fn test_map_severity_unknown() {
        assert_eq!(map_severity("Unknown"), AlertSeverity::Advisory);
        assert_eq!(map_severity(""), AlertSeverity::Advisory);
    }

    // --- Severity ordering tests ---

    #[test]
    fn test_severity_ordering() {
        assert!(AlertSeverity::Advisory < AlertSeverity::Watch);
        assert!(AlertSeverity::Watch < AlertSeverity::Warning);
        assert!(AlertSeverity::Advisory < AlertSeverity::Warning);
    }

    // --- Time formatting tests ---

    #[test]
    fn test_format_expiry_rfc3339() {
        let expires = Some("2024-01-15T18:00:00-06:00".to_string());
        let result = format_expiry(&expires);
        assert_eq!(result, Some("6:00 PM".to_string()));
    }

    #[test]
    fn test_format_expiry_midnight() {
        let expires = Some("2024-01-15T00:30:00-05:00".to_string());
        let result = format_expiry(&expires);
        assert_eq!(result, Some("12:30 AM".to_string()));
    }

    #[test]
    fn test_format_expiry_noon() {
        let expires = Some("2024-01-15T12:00:00-05:00".to_string());
        let result = format_expiry(&expires);
        assert_eq!(result, Some("12:00 PM".to_string()));
    }

    #[test]
    fn test_format_expiry_malformed() {
        let expires = Some("not-a-timestamp".to_string());
        assert!(format_expiry(&expires).is_none());
    }

    #[test]
    fn test_format_expiry_empty() {
        let expires: Option<String> = None;
        assert!(format_expiry(&expires).is_none());
    }

    #[test]
    fn test_format_expiry_empty_string() {
        let expires = Some("".to_string());
        assert!(format_expiry(&expires).is_none());
    }

    // --- Deserialization tests ---

    #[test]
    fn test_parse_alerts_response() {
        let json = r#"{
            "@context": [],
            "features": [
                {
                    "type": "Feature",
                    "properties": {
                        "event": "Tornado Warning",
                        "severity": "Extreme",
                        "expires": "2024-01-15T18:00:00-06:00"
                    }
                }
            ]
        }"#;
        let response: NwsAlertsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.features.len(), 1);
        assert_eq!(response.features[0].properties.event, "Tornado Warning");
        assert_eq!(response.features[0].properties.severity, "Extreme");
    }

    #[test]
    fn test_parse_empty_response() {
        let json = r#"{"@context": [], "features": []}"#;
        let response: NwsAlertsResponse = serde_json::from_str(json).unwrap();
        assert!(response.features.is_empty());
    }

    #[test]
    fn test_parse_multiple_alerts() {
        let json = r#"{
            "features": [
                {
                    "properties": {
                        "event": "Flash Flood Warning",
                        "severity": "Severe",
                        "expires": "2024-01-15T20:00:00-05:00"
                    }
                },
                {
                    "properties": {
                        "event": "Wind Advisory",
                        "severity": "Minor",
                        "expires": "2024-01-16T12:00:00-05:00"
                    }
                }
            ]
        }"#;
        let response: NwsAlertsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.features.len(), 2);
    }

    // --- Alert creation tests ---

    #[test]
    fn test_alert_from_feature() {
        let json = r#"{
            "properties": {
                "event": "Tornado Warning",
                "severity": "Extreme",
                "expires": "2024-01-15T18:00:00-06:00"
            }
        }"#;
        let feature: AlertFeature = serde_json::from_str(json).unwrap();

        let now = DateTime::parse_from_rfc3339("2024-01-15T10:00:00-06:00")
            .unwrap()
            .with_timezone(&Utc);
        let alert = Alert::from_feature(&feature, now).unwrap();

        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.event, "Tornado Warning");
        assert_eq!(alert.expires_time, Some("6:00 PM".to_string()));
    }

    #[test]
    fn test_alert_from_feature_no_expiry() {
        let json = r#"{
            "properties": {
                "event": "Special Weather Statement",
                "severity": "Minor"
            }
        }"#;
        let feature: AlertFeature = serde_json::from_str(json).unwrap();

        let now = Utc::now();
        let alert = Alert::from_feature(&feature, now).unwrap();

        assert_eq!(alert.severity, AlertSeverity::Advisory);
        assert_eq!(alert.event, "Special Weather Statement");
        assert!(alert.expires_time.is_none());
    }

    #[test]
    fn test_alert_from_feature_expired() {
        let json = r#"{
            "properties": {
                "event": "Old Warning",
                "severity": "Extreme",
                "expires": "2024-01-01T12:00:00-06:00"
            }
        }"#;
        let feature: AlertFeature = serde_json::from_str(json).unwrap();

        let now = DateTime::parse_from_rfc3339("2024-02-01T10:00:00-06:00")
            .unwrap()
            .with_timezone(&Utc);
        assert!(Alert::from_feature(&feature, now).is_none());
    }

    // --- Edge cases ---

    #[test]
    fn test_alert_feature_missing_expires() {
        let json = r#"{
            "properties": {
                "event": "Dense Fog Advisory",
                "severity": "Moderate"
            }
        }"#;
        let feature: AlertFeature = serde_json::from_str(json).unwrap();
        assert!(feature.properties.expires.is_none());
    }

    #[test]
    fn test_format_expiry_9am() {
        let expires = Some("2024-01-15T09:30:00-05:00".to_string());
        assert_eq!(format_expiry(&expires), Some("9:30 AM".to_string()));
    }

    #[test]
    fn test_format_expiry_11pm() {
        let expires = Some("2024-01-15T23:45:00-05:00".to_string());
        assert_eq!(format_expiry(&expires), Some("11:45 PM".to_string()));
    }
}
