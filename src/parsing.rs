//! Data parsing and structures for SPARQL responses

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Response structure for SPARQL JSON results format
#[derive(Debug, Deserialize)]
pub struct SparqlResponse {
    pub results: Results,
}

/// Container for SPARQL query result bindings
#[derive(Debug, Deserialize)]
pub struct Results {
    pub bindings: Vec<SparqlBinding>,
}

/// SPARQL binding structure for station temperature queries
#[derive(Debug, Deserialize)]
pub struct SparqlBinding {
    #[serde(deserialize_with = "deserialize_sparql_value")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_sparql_datetime")]
    pub time: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_sparql_temperature")]
    pub temperature: f32,
}

/// Custom deserializer to extract the "value" field from SPARQL binding objects
fn deserialize_sparql_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_binding_value(deserializer)
}

/// Helper function to extract the value string from a SPARQL binding
fn deserialize_binding_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct BindingValue {
        value: String,
    }

    let binding_value = BindingValue::deserialize(deserializer)?;
    Ok(binding_value.value)
}

/// Custom deserializer to extract and parse DateTime from SPARQL binding objects
fn deserialize_sparql_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = deserialize_binding_value(deserializer)?;
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| serde::de::Error::custom(format!("Invalid datetime format: {e}")))
}

/// Custom deserializer to extract and parse temperature from SPARQL binding objects
fn deserialize_sparql_temperature<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = deserialize_binding_value(deserializer)?;
    value
        .parse::<f32>()
        .map_err(|e| serde::de::Error::custom(format!("Invalid temperature format: {e}")))
}

/// Represents a water temperature measurement from a monitoring station
#[derive(Debug)]
pub struct StationMeasurement {
    pub station_id: u32,
    pub station_name: String,
    pub time: DateTime<Utc>,
    pub temperature: f32,
}
