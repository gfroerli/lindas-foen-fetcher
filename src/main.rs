//! LINDAS FOEN Fetcher
//!
//! This application fetches water temperature measurements from the FOEN (Swiss
//! Federal Office for the Environment) LINDAS SPARQL endpoint and displays them
//! in the terminal.

mod display;
mod parsing;
mod sparql;

use display::{
    print_error_summary, print_measurement_row, print_no_data_message, print_summary,
    print_table_header,
};
use parsing::StationMeasurement;
use sparql::{STATION_IDS, get_station_measurements};

/// Fetches all station data and handles errors appropriately
async fn fetch_all_station_data(client: &reqwest::Client) -> (Vec<StationMeasurement>, usize) {
    let mut all_measurements = Vec::new();
    let mut error_count = 0;

    for &station_id in STATION_IDS {
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
    println!(
        "Fetching water temperature data for {} stations: {:?}...",
        STATION_IDS.len(),
        STATION_IDS
    );

    let client = reqwest::Client::new();

    print_table_header();

    let (all_measurements, error_count) = fetch_all_station_data(&client).await;

    print_summary(all_measurements.len());
    print_error_summary(error_count);

    Ok(())
}
