//! LINDAS Hydrodata Fetcher
//!
//! This application fetches water temperature measurements from the FOEN (Swiss
//! Federal Office for the Environment) LINDAS SPARQL endpoint and sends them
//! to the Gfrörli API.

mod config;
mod database;
mod gfroerli;
mod parsing;
mod sparql;

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use rusqlite::Connection;
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};

use crate::{
    config::{Config, RunMode},
    database::{init_database, is_measurement_sent, record_measurement_sent},
    gfroerli::send_measurement,
    sparql::fetch_station_measurement,
};

/// Command line arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    /// Dry run mode - fetch data but don't send to API or record in database
    #[arg(long)]
    dry_run: bool,
}

/// Processes a single station: Fetches data and sends to API
async fn process_station(
    client: &reqwest::Client,
    config: &Config,
    db_conn: &Connection,
    station_id: u32,
    dry_run: bool,
) -> Result<()> {
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

    // Check if this measurement was already sent
    if is_measurement_sent(db_conn, sensor_id, &measurement.time)? {
        warn!(
            "Station {} ({}) measurement at {} already sent, skipping",
            measurement.station_id,
            measurement.station_name,
            measurement.time.format("%Y-%m-%d %H:%M:%S %z")
        );
        return Ok(());
    }

    if dry_run {
        info!(
            "Station {} ({}) would be sent to API (sensor {}) [DRY RUN]",
            measurement.station_id, measurement.station_name, sensor_id,
        );
        return Ok(());
    }

    // Send to API
    match send_measurement(client, &config.gfroerli_api, &measurement, sensor_id).await {
        Ok(()) => {
            // Record that we successfully sent this measurement
            record_measurement_sent(db_conn, sensor_id, &measurement.time)?;
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

    // Initialize database
    let db_conn =
        init_database(config.database_path()).with_context(|| "Failed to initialize database")?;

    // Initialize HTTP client
    let client = reqwest::Client::new();

    if args.dry_run {
        info!("Running in DRY RUN mode - no data will be sent to API or recorded in database");
    }

    let interval_minutes = config.run_interval_minutes();
    let mode = config.run_mode();

    match mode {
        RunMode::Oneshot => debug!("Running in oneshot mode"),
        RunMode::Loop => info!(
            "Running in loop mode with {} minute intervals",
            interval_minutes
        ),
    }

    loop {
        debug!("Starting station processing cycle");

        let mut total_success = 0;
        let mut total_errors = 0;

        for &station_id in &station_ids {
            if let Err(e) =
                process_station(&client, &config, &db_conn, station_id, args.dry_run).await
            {
                error!("Failed to process station {}: {}", station_id, e);
                total_errors += 1;
            } else {
                total_success += 1;
            }
        }

        match mode {
            RunMode::Oneshot => {
                info!(
                    "Successfully sent {} measurements to Gfrörli API",
                    total_success
                );
                if total_errors > 0 {
                    error!("Total errors encountered: {}", total_errors);
                }
                break;
            }
            RunMode::Loop => {
                info!(
                    "Cycle complete - Successfully sent {} measurements to Gfrörli API",
                    total_success
                );
                if total_errors > 0 {
                    error!(
                        "Cycle complete - Total errors encountered: {}",
                        total_errors
                    );
                }

                let sleep_duration = Duration::from_secs(interval_minutes as u64 * 60);
                info!("Sleeping for {} minutes until next cycle", interval_minutes);
                sleep(sleep_duration).await;
            }
        }
    }

    Ok(())
}
