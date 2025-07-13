//! Configuration management for the LINDAS FOEN fetcher

use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Execution mode for the application
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub enum RunMode {
    /// Run once and exit
    #[default]
    #[serde(rename = "oneshot")]
    Oneshot,
    /// Run continuously in a loop
    #[serde(rename = "loop")]
    Loop,
}

/// Main configuration structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// List of station configurations
    pub stations: Vec<StationConfig>,
    /// Gfrörli API configuration
    pub gfroerli_api: GfroerliConfig,
    /// Logging configuration (optional, defaults to "info")
    pub logging: Option<LoggingConfig>,
    /// Database configuration (optional, defaults to "measurements.db")
    pub database: Option<DatabaseConfig>,
    /// Run configuration (optional, defaults to oneshot mode)
    pub run: Option<RunConfig>,
}

/// Gfrörli configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct GfroerliConfig {
    /// Gfrörli API base URL
    pub api_url: String,
    /// Gfrörli private API key
    pub api_key: String,
}

/// Logging configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Log level filter (using env_logger syntax)
    pub level: String,
}

/// Database configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// Path to SQLite database file
    pub path: String,
}

/// Run configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct RunConfig {
    /// Interval between runs in minutes (only used in loop mode, defaults to 5 minutes)
    pub interval_minutes: u32,
    /// Execution mode: oneshot (default) or loop
    pub mode: Option<RunMode>,
}

/// Station configuration with FOEN station ID and Gfrörli sensor ID mapping
#[derive(Debug, Deserialize, Serialize)]
pub struct StationConfig {
    /// FOEN hydrological station ID
    pub foen_station_id: u32,
    /// Gfrörli sensor ID
    pub gfroerli_sensor_id: u32,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        debug!("Loading configuration from '{}'", path_ref.display());

        let content = fs::read_to_string(path_ref)
            .with_context(|| format!("Failed to read config file '{}'", path_ref.display()))?;
        let config: Config = toml::from_str(&content).with_context(|| {
            format!("Failed to parse TOML config file '{}'", path_ref.display())
        })?;

        debug!(
            "Successfully loaded configuration with {} stations",
            config.stations.len()
        );
        Ok(config)
    }

    /// Get the logging level, with fallback to "info" if not configured
    pub fn logging_level(&self) -> &str {
        self.logging
            .as_ref()
            .map(|l| l.level.as_str())
            .unwrap_or("info")
    }

    /// Get the database path, with fallback to "measurements.db" if not configured
    pub fn database_path(&self) -> &str {
        self.database
            .as_ref()
            .map(|d| d.path.as_str())
            .unwrap_or("measurements.db")
    }

    /// Get the run interval in minutes, with fallback to 5 minutes if not configured
    pub fn run_interval_minutes(&self) -> u32 {
        self.run.as_ref().map(|r| r.interval_minutes).unwrap_or(5)
    }

    /// Get the run mode, with fallback to oneshot if not configured
    pub fn run_mode(&self) -> RunMode {
        self.run
            .as_ref()
            .and_then(|r| r.mode.clone())
            .unwrap_or_default()
    }

    /// Get all FOEN station IDs
    pub fn foen_station_ids(&self) -> Vec<u32> {
        self.stations
            .iter()
            .map(|station| station.foen_station_id)
            .collect()
    }

    /// Find Gfrörli sensor ID for a given FOEN station ID
    pub fn find_gfroerli_sensor_id(&self, foen_station_id: u32) -> Option<u32> {
        self.stations
            .iter()
            .find(|station| station.foen_station_id == foen_station_id)
            .map(|station| station.gfroerli_sensor_id)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config {
            stations: vec![
                StationConfig {
                    foen_station_id: 2104,
                    gfroerli_sensor_id: 1,
                },
                StationConfig {
                    foen_station_id: 2176,
                    gfroerli_sensor_id: 2,
                },
            ],
            gfroerli_api: GfroerliConfig {
                api_url: "http://localhost:3000/api/".to_string(),
                api_key: "test-api-key".to_string(),
            },
            logging: Some(LoggingConfig {
                level: "info".to_string(),
            }),
            database: Some(DatabaseConfig {
                path: "test.db".to_string(),
            }),
            run: Some(RunConfig {
                interval_minutes: 10,
                mode: Some(RunMode::Oneshot),
            }),
        };
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.stations.len(), deserialized.stations.len());
        assert_eq!(
            config.stations[0].foen_station_id,
            deserialized.stations[0].foen_station_id
        );
        assert_eq!(
            config.stations[0].gfroerli_sensor_id,
            deserialized.stations[0].gfroerli_sensor_id
        );
    }

    #[test]
    fn test_config_file_operations() {
        let test_file = PathBuf::from("test_config.toml");
        let test_config = Config {
            stations: vec![
                StationConfig {
                    foen_station_id: 2104,
                    gfroerli_sensor_id: 1,
                },
                StationConfig {
                    foen_station_id: 2176,
                    gfroerli_sensor_id: 2,
                },
            ],
            gfroerli_api: GfroerliConfig {
                api_url: "http://localhost:3000/api/".to_string(),
                api_key: "test-api-key".to_string(),
            },
            logging: Some(LoggingConfig {
                level: "info".to_string(),
            }),
            database: Some(DatabaseConfig {
                path: "test.db".to_string(),
            }),
            run: Some(RunConfig {
                interval_minutes: 10,
                mode: Some(RunMode::Loop),
            }),
        };

        // Clean up any existing test file
        let _ = fs::remove_file(&test_file);

        // Create test config file
        let toml_content = toml::to_string_pretty(&test_config).unwrap();
        fs::write(&test_file, toml_content).unwrap();

        // Load config from file
        let loaded_config = Config::load_from_file(&test_file).unwrap();
        assert_eq!(loaded_config.stations.len(), test_config.stations.len());
        assert_eq!(
            loaded_config.stations[0].foen_station_id,
            test_config.stations[0].foen_station_id
        );

        // Clean up
        fs::remove_file(&test_file).unwrap();
    }
}
