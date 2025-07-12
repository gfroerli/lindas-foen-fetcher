//! LINDAS FOEN Fetcher
//!
//! This application fetches water temperature measurements from the FOEN (Swiss
//! Federal Office for the Environment) LINDAS SPARQL endpoint and displays them
//! in the terminal.

mod config;
mod display;
mod parsing;
mod sparql;

use config::Config;
use display::{
    print_error_summary, print_measurement_row, print_no_data_message, print_summary,
    print_table_header,
};
use parsing::StationMeasurement;
use sparql::get_station_measurements;

/// Fetches all station data and handles errors appropriately
async fn fetch_all_station_data(
    client: &reqwest::Client,
    station_ids: &[u32],
) -> (Vec<StationMeasurement>, usize) {
    let mut all_measurements = Vec::new();
    let mut error_count = 0;

    for &station_id in station_ids {
        match get_station_measurements(client, station_id).await {
            Ok(measurements) => {
                if measurements.is_empty() {
                    print_no_data_message(station_id);
                } else {
                    for measurement in &measurements {
                        print_measurement_row(measurement);
                    }
                    all_measurements.extend(measurements);
                }
            }
            Err(e) => {
                eprintln!("Error fetching data for station {station_id}: {e}");
                error_count += 1;
            }
        }
    }

    (all_measurements, error_count)
}

/// Main application entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load_from_file("config.toml")
        .map_err(|e| format!("Failed to load config.toml: {e}"))?;

    let station_ids = config.foen_station_ids();

    println!(
        "Fetching water temperature data for {} stations: {:?}...",
        station_ids.len(),
        station_ids
    );

    let client = reqwest::Client::new();

    print_table_header();

    let (all_measurements, error_count) = fetch_all_station_data(&client, &station_ids).await;

    print_summary(all_measurements.len());
    print_error_summary(error_count);

    Ok(())
}
