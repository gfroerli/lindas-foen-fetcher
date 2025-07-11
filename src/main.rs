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

/// SPARQL query to fetch station name and latest water temperature for station 2104
const SPARQL_QUERY: &str = r#"
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX station: <https://environment.ld.admin.ch/foen/hydro/station/>
PREFIX riverOberservation: <https://environment.ld.admin.ch/foen/hydro/river/observation/>
PREFIX dimension: <https://environment.ld.admin.ch/foen/hydro/dimension/>

SELECT ?name ?time ?temperature WHERE {
    station:2104 <http://schema.org/name> ?name .
    riverOberservation:2104
        dimension:waterTemperature ?temperature ;
        dimension:measurementTime ?time .
}
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching water temperature data for station 2104...");

    let client = reqwest::Client::new();

    // Prepare the form data for the SPARQL query
    let params = [("query", SPARQL_QUERY)];

    // Fetch request
    let response = client
        .post(SPARQL_ENDPOINT)
        .header("Accept", "application/sparql-results+json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to SPARQL endpoint: {}", e))?;
    if !response.status().is_success() {
        let status = response.status();
        eprintln!("Error: HTTP {}", status);
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        eprintln!("Response: {}", error_text);
        return Err(format!("SPARQL query failed with status: {}", status).into());
    }

    // Parse response
    let sparql_response: SparqlResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    let total_records = sparql_response.results.bindings.len();

    // Print results
    println!("\nResults:");
    println!(
        "{:<30} {:<25} {:<15}",
        "Station Name", "Time", "Temperature (°C)"
    );
    println!("{}", "-".repeat(75));

    if total_records == 0 {
        println!("No temperature data found for station 2104.");
        return Ok(());
    }

    for binding in sparql_response.results.bindings {
        let name = binding.get("name").map_or("N/A", |v| v.value.as_str());
        let time = binding.get("time").map_or("N/A", |v| v.value.as_str());

        let temperature = binding
            .get("temperature")
            .map_or("N/A", |v| v.value.as_str());

        println!("{:<30} {:<25} {:<15}", name, time, temperature);
    }

    println!("\nTotal records found: {}", total_records);

    Ok(())
}
