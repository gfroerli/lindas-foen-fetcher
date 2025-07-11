//! Data parsing and structures for SPARQL responses

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
    #[serde(deserialize_with = "deserialize_sparql_value")]
    pub time: String,
    #[serde(deserialize_with = "deserialize_sparql_value")]
    pub temperature: String,
}

/// Custom deserializer to extract the "value" field from SPARQL binding objects
fn deserialize_sparql_value<'de, D>(deserializer: D) -> Result<String, D::Error>
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

/// Represents a water temperature measurement from a monitoring station
#[derive(Debug)]
pub struct StationMeasurement {
    pub station_id: u32,
    pub station_name: String,
    pub time: String,
    pub temperature: String,
}

/// Converts SPARQL response to structured measurement data
pub fn parse_station_measurements(
    station_id: u32,
    sparql_response: SparqlResponse,
) -> Vec<StationMeasurement> {
    sparql_response
        .results
        .bindings
        .into_iter()
        .map(|binding| StationMeasurement {
            station_id,
            station_name: binding.name,
            time: binding.time,
            temperature: binding.temperature,
        })
        .collect()
}
