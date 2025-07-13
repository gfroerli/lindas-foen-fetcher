//! LINDAS Hydrodata Fetcher
//!
//! This application fetches water temperature measurements from the FOEN (Swiss
//! Federal Office for the Environment) LINDAS SPARQL endpoint and sends them
//! to the Gfrörli API.

mod config;
mod gfroerli;
mod parsing;
mod sparql;

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use tracing::{debug, error, info};

use crate::{config::Config, gfroerli::send_measurement, sparql::fetch_station_measurement};

/// Command line arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

/// Processes a single station: Fetches data and sends to API
async fn process_station(client: &reqwest::Client, config: &Config, station_id: u32) -> Result<()> {
    // Query latest measurement from LINDAS
    let measurement = fetch_station_measurement(client, station_id)
        .await
        .with_context(|| format!("Error fetching data for station {station_id}"))?
        .ok_or_else(|| anyhow!("No temperature data found for station {}", station_id))?;
    info!(
        "Station {} ({}) fetched: {:.3}°C (at {})",
        measurement.station_id,
        measurement.station_name,
        measurement.temperature,
        measurement.time.format("%Y-%m-%d %H:%M:%S %z"),
    );

    // Get Gfrörli sensor ID from config
    let sensor_id = config
        .find_gfroerli_sensor_id(measurement.station_id)
        .ok_or_else(|| {
            anyhow!(
                "No sensor mapping found for station {}",
                measurement.station_id
            )
        })?;

    // Send to API
    match send_measurement(client, &config.gfroerli_api, &measurement, sensor_id).await {
        Ok(()) => {
            info!(
                "Station {} ({}) sent to API (sensor {})",
                measurement.station_id, measurement.station_name, sensor_id,
            );
            Ok(())
        }
        Err(e) => Err(anyhow!(
            "Failed to send measurement for station {} (sensor {}): {}",
            measurement.station_id,
            sensor_id,
            e
        )),
    }
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

    debug!("Starting station processing");

    let mut total_success = 0;
    let mut total_errors = 0;

    for &station_id in &station_ids {
        if let Err(e) = process_station(&client, &config, station_id).await {
            error!("Failed to process station {}: {}", station_id, e);
            total_errors += 1;
        } else {
            total_success += 1;
        }
    }

    info!(
        "Successfully sent {} measurements to Gfrörli API",
        total_success
    );
    if total_errors > 0 {
        error!("Total errors encountered: {}", total_errors);
    }
    Ok(())
}
