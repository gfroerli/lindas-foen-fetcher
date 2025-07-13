//! LINDAS Hydrodata Fetcher
//!
//! This application fetches water temperature measurements from the FOEN (Swiss
//! Federal Office for the Environment) LINDAS SPARQL endpoint and displays them
//! in the terminal.

mod config;
mod gfroerli;
mod parsing;
mod sparql;

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{debug, error, info};

use crate::{
    config::Config, gfroerli::send_all_measurements, parsing::StationMeasurement,
    sparql::get_station_measurements,
};

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
                    info!("No temperature data found for station {}", station_id);
                } else {
                    for measurement in &measurements {
                        info!(
                            "Station {} ({}): {:.3}°C ({})",
                            measurement.station_id,
                            measurement.station_name,
                            measurement.temperature,
                            measurement.time.format("%Y-%m-%d %H:%M:%S %z"),
                        );
                    }
                    all_measurements.extend(measurements);
                }
            }
            Err(e) => {
                error!("Error fetching data for station {}: {}", station_id, e);
                error_count += 1;
            }
        }
    }

    (all_measurements, error_count)
}

/// Main application entry point
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = Config::load_from_file(&args.config)
        .with_context(|| format!("Failed to load config from '{}'", args.config))?;

    // Initialize tracing with config-based logging level
    let logging_level = config.logging_level();
    let env_filter = tracing_subscriber::EnvFilter::try_new(logging_level)
        .with_context(|| format!("Invalid logging level: '{logging_level}'"))?;

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let station_ids = config.foen_station_ids();

    info!(
        "Fetching water temperature data for {} stations: {:?}",
        station_ids.len(),
        station_ids
    );

    // Initialize HTTP client
    let client = reqwest::Client::new();

    debug!("Starting data fetch for all stations");

    let (all_measurements, error_count) = fetch_all_station_data(&client, &station_ids).await;

    info!("Total records found: {}", all_measurements.len());
    if error_count > 0 {
        error!("{} station(s) had errors during data fetching", error_count);
    }

    // Send measurements to Gfrörli API
    if !all_measurements.is_empty() {
        info!("Sending measurements to Gfrörli API");
        let (success_count, api_error_count) = send_all_measurements(
            &client,
            &config.gfroerli_api,
            &all_measurements,
            |foen_station_id| config.find_gfroerli_sensor_id(foen_station_id),
        )
        .await;

        info!("Gfrörli API Summary - Successfully sent: {}", success_count);
        if api_error_count > 0 {
            error!(
                "Failed to send {} measurements to Gfrörli API",
                api_error_count
            );
        }
    }

    Ok(())
}
