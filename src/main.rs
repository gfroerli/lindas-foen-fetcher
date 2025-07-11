//! LINDAS FOEN Fetcher
//!
//! This application fetches water temperature measurements from the FOEN (Swiss
//! Federal Office for the Environment) LINDAS SPARQL endpoint and displays them
//! in the terminal.

use std::collections::HashMap;

use reqwest;
use serde::Deserialize;

/// Response structure for SPARQL JSON results format
#[derive(Debug, Deserialize)]
struct SparqlResponse {
    results: Results,
}

/// Container for SPARQL query result bindings
#[derive(Debug, Deserialize)]
struct Results {
    bindings: Vec<HashMap<String, BindingValue>>,
}

/// Individual value in a SPARQL result binding
#[derive(Debug, Deserialize)]
struct BindingValue {
    value: String,
}

/// SPARQL endpoint URL for the LINDAS platform
const SPARQL_ENDPOINT: &str = "https://lindas.admin.ch/query";

/// Station IDs to query for water temperature data
const STATION_IDS: &[u32] = &[2104, 2176, 2635, 2070];

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
fn generate_sparql_query(station_id: u32) -> String {
    SPARQL_QUERY_TEMPLATE.replace("{STATION_ID}", &station_id.to_string())
}

/// Fetches water temperature data for a specific station
async fn fetch_station_data(
    client: &reqwest::Client,
    station_id: u32,
) -> Result<SparqlResponse, Box<dyn std::error::Error>> {
    let query = generate_sparql_query(station_id);
    let params = [("query", query.as_str())];

    let response = client
        .post(SPARQL_ENDPOINT)
        .header("Accept", "application/sparql-results+json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to SPARQL endpoint: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        return Err(format!(
            "SPARQL query failed for station {}: HTTP {} - {}",
            station_id, status, error_text
        )
        .into());
    }

    let sparql_response: SparqlResponse = response.json().await.map_err(|e| {
        format!(
            "Failed to parse JSON response for station {}: {}",
            station_id, e
        )
    })?;

    Ok(sparql_response)
}

/// Fetches and displays water temperature data from the LINDAS SPARQL endpoint

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Fetching water temperature data for stations: {:?}...",
        STATION_IDS
    );

    let client = reqwest::Client::new();
    let mut total_records_all = 0;

    // Print header
    println!("\nResults:");
    println!(
        "{:<10} {:<30} {:<25} {:<15}",
        "Station ID", "Station Name", "Time", "Temperature (°C)"
    );
    println!("{}", "-".repeat(85));

    for &station_id in STATION_IDS {
        match fetch_station_data(&client, station_id).await {
            Ok(sparql_response) => {
                let total_records = sparql_response.results.bindings.len();
                total_records_all += total_records;

                if total_records == 0 {
                    println!("{:<10} No temperature data found", station_id);
                } else {
                    for binding in sparql_response.results.bindings {
                        let name = binding.get("name").map_or("N/A", |v| v.value.as_str());
                        let time = binding.get("time").map_or("N/A", |v| v.value.as_str());
                        let temperature = binding
                            .get("temperature")
                            .map_or("N/A", |v| v.value.as_str());

                        println!(
                            "{:<10} {:<30} {:<25} {:<15}",
                            station_id, name, time, temperature
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("Error fetching data for station {}: {}", station_id, e);
            }
        }
    }

    println!("\nTotal records found: {}", total_records_all);

    Ok(())
}
