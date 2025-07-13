//! Gfrörli API integration for sending measurement data

use anyhow::{Context, Result};
use tracing::{debug, error};

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::config::GfroerliConfig;
use crate::parsing::StationMeasurement;

/// Request payload for Gfrörli measurements API
#[derive(Debug, Serialize)]
struct MeasurementRequest {
    sensor_id: u32,
    temperature: f32,
    created_at: DateTime<Utc>,
}

/// Helper function to build API endpoint URL
fn build_api_url(base_url: &str, endpoint: &str) -> String {
    let base = base_url.trim_end_matches('/');
    format!("{base}/{endpoint}")
}

/// Sends a measurement to the Gfrörli API
pub async fn send_measurement(
    client: &reqwest::Client,
    config: &GfroerliConfig,
    measurement: &StationMeasurement,
    sensor_id: u32,
) -> Result<()> {
    let url = build_api_url(&config.api_url, "measurements");

    let payload = MeasurementRequest {
        sensor_id,
        temperature: measurement.temperature,
        created_at: measurement.time,
    };

    debug!(
        "Sending measurement to Gfrörli API for station {} (sensor {}): {}°C at {}",
        measurement.station_id, sensor_id, measurement.temperature, measurement.time
    );

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&payload)
        .send()
        .await
        .with_context(|| format!("Failed to send measurement to Gfrörli API at {url}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        return Err(anyhow::anyhow!(
            "Gfrörli API request failed: HTTP {status} - {error_text}"
        ));
    }

    Ok(())
}

/// Sends all measurements to the Gfrörli API
pub async fn send_all_measurements(
    client: &reqwest::Client,
    config: &GfroerliConfig,
    measurements: &[StationMeasurement],
    find_sensor_id: impl Fn(u32) -> Option<u32>,
) -> (usize, usize) {
    let mut success_count = 0;
    let mut error_count = 0;

    for measurement in measurements {
        match find_sensor_id(measurement.station_id) {
            Some(sensor_id) => {
                match send_measurement(client, config, measurement, sensor_id).await {
                    Ok(()) => {
                        debug!(
                            "Sent measurement for station {} (sensor {}) to Gfrörli",
                            measurement.station_id, sensor_id
                        );
                        success_count += 1;
                    }
                    Err(e) => {
                        error!(
                            "Failed to send measurement for station {} (sensor {}): {}",
                            measurement.station_id, sensor_id, e
                        );
                        error_count += 1;
                    }
                }
            }
            None => {
                error!(
                    "No sensor mapping found for station {}",
                    measurement.station_id
                );
                error_count += 1;
            }
        }
    }

    (success_count, error_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_build_api_url_with_trailing_slash() {
        let url = build_api_url("http://localhost:3000/api/", "measurements");
        assert_eq!(url, "http://localhost:3000/api/measurements");
    }

    #[test]
    fn test_build_api_url_without_trailing_slash() {
        let url = build_api_url("http://localhost:3000/api", "measurements");
        assert_eq!(url, "http://localhost:3000/api/measurements");
    }

    #[test]
    fn test_measurement_request_serialization() {
        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 12, 30, 45).unwrap();
        let request = MeasurementRequest {
            sensor_id: 1,
            temperature: 20.7,
            created_at: timestamp,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"sensor_id\":1"));
        assert!(json.contains("\"temperature\":20.7"));
        assert!(json.contains("\"created_at\":\"2023-01-01T12:30:45Z\""));
    }
}
