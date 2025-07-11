//! SPARQL query building and data fetching

use crate::parsing::{SparqlResponse, StationMeasurement, parse_station_measurements};

/// SPARQL endpoint URL for the LINDAS platform
pub const SPARQL_ENDPOINT: &str = "https://lindas.admin.ch/query";

/// Station IDs to query for water temperature data
pub const STATION_IDS: &[u32] = &[2104, 2176, 2635, 2070];

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

/// Generates a SPARQL query for a specific station ID
pub fn build_sparql_query(station_id: u32) -> String {
    SPARQL_QUERY_TEMPLATE.replace("{STATION_ID}", &station_id.to_string())
}

/// Fetches raw SPARQL response for a specific station
pub async fn fetch_sparql_data(
    client: &reqwest::Client,
    station_id: u32,
) -> Result<SparqlResponse, Box<dyn std::error::Error>> {
    let query = build_sparql_query(station_id);
    let params = [("query", query.as_str())];

    let response = client
        .post(SPARQL_ENDPOINT)
        .header("Accept", "application/sparql-results+json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Network error for station {station_id}: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        return Err(format!(
            "SPARQL query failed for station {station_id}: HTTP {status} - {error_text}"
        )
        .into());
    }

    let sparql_response: SparqlResponse = response
        .json()
        .await
        .map_err(|e| format!("JSON parsing error for station {station_id}: {e}"))?;

    Ok(sparql_response)
}

/// Fetches and parses station measurement data
pub async fn get_station_measurements(
    client: &reqwest::Client,
    station_id: u32,
) -> Result<Vec<StationMeasurement>, Box<dyn std::error::Error>> {
    let sparql_response = fetch_sparql_data(client, station_id).await?;
    Ok(parse_station_measurements(station_id, sparql_response))
}
