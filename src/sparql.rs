//! SPARQL query building and data fetching

use anyhow::{Context, Result};
use tracing::debug;

use crate::parsing::{SparqlResponse, StationMeasurement};

/// SPARQL endpoint URL for the LINDAS platform
pub const SPARQL_ENDPOINT: &str = "https://lindas.admin.ch/query";

/// SPARQL query template to fetch station name and latest water temperature
const SPARQL_QUERY_TEMPLATE: &str = r#"
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX station: <https://environment.ld.admin.ch/foen/hydro/station/>
PREFIX riverOberservation: <https://environment.ld.admin.ch/foen/hydro/river/observation/>
PREFIX dimension: <https://environment.ld.admin.ch/foen/hydro/dimension/>

SELECT ?name ?time ?temperature WHERE {
    station:{STATION_ID} <http://schema.org/name> ?name .
    riverOberservation:{STATION_ID}
        dimension:waterTemperature ?temperature ;
        dimension:measurementTime ?time .
}
ORDER BY DESC(?time)
LIMIT 1
"#;

/// Fetches and parses station measurement data
pub async fn fetch_station_measurement(
    client: &reqwest::Client,
    station_id: u32,
) -> Result<Option<StationMeasurement>> {
    // Create query
    let query = SPARQL_QUERY_TEMPLATE.replace("{STATION_ID}", &station_id.to_string());
    let params = [("query", query.as_str())];

    // Send request
    debug!("Sending SPARQL request for station {}", station_id);
    let response = client
        .post(SPARQL_ENDPOINT)
        .header("Accept", "application/sparql-results+json")
        .form(&params)
        .send()
        .await
        .with_context(|| format!("Failed to send SPARQL request for station {station_id}"))?;

    // Handle errors
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        return Err(anyhow::anyhow!(
            "SPARQL query failed for station {station_id}: HTTP {status} - {error_text}"
        ));
    }

    // Parse response
    let sparql_response: SparqlResponse = response.json().await.with_context(|| {
        format!("Failed to parse SPARQL JSON response for station {station_id}")
    })?;
    debug!(
        "Successfully received SPARQL response for station {} with {} bindings",
        station_id,
        sparql_response.results.bindings.len()
    );
    if sparql_response.results.bindings.len() > 1 {
        return Err(anyhow::anyhow!(
            "Expected 1 result for SPARQL query for station {station_id}, but got {}",
            sparql_response.results.bindings.len(),
        ));
    }

    Ok(sparql_response
        .results
        .bindings
        .into_iter()
        .next()
        .map(|binding| StationMeasurement {
            station_id,
            station_name: binding.name,
            time: binding.time,
            temperature: binding.temperature,
        }))
}
