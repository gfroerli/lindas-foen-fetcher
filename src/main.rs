//! LINDAS Hydrodata Fetcher
//!
//! This application fetches water temperature measurements from the FOEN (Swiss
//! Federal Office for the Environment) LINDAS SPARQL endpoint and displays them
//! in the terminal.

mod config;
mod display;
mod gfroerli;
mod parsing;
mod sparql;

use clap::Parser;

use config::Config;
use display::{
    print_error_summary, print_measurement_row, print_no_data_message, print_summary,
    print_table_header,
};
use gfroerli::send_all_measurements;
use parsing::StationMeasurement;
use sparql::get_station_measurements;

/// Command line arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

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
    let args = Args::parse();

    // Load configuration
    let config = Config::load_from_file(&args.config)
        .map_err(|e| format!("Failed to load config at {}: {e}", args.config))?;

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

    // Send measurements to Gfrörli API
    if !all_measurements.is_empty() {
        println!("\nSending measurements to Gfrörli API...");
        let (success_count, api_error_count) = send_all_measurements(
            &client,
            &config.gfroerli_api,
            &all_measurements,
            |foen_station_id| config.find_gfroerli_sensor_id(foen_station_id),
        )
        .await;

        println!("\nGfrörli API Summary:");
        println!("Successfully sent: {success_count}");
        if api_error_count > 0 {
            println!("Failed to send: {api_error_count}");
        }
    }

    Ok(())
}
