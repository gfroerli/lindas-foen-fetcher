//! Configuration management for the LINDAS FOEN fetcher

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// List of station configurations
    pub stations: Vec<StationConfig>,
    /// Gfrörli API configuration
    pub gfroerli_api: GfroerliConfig,
}

/// Gfrörli configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct GfroerliConfig {
    /// Gfrörli API base URL
    pub api_url: String,
    /// Gfrörli private API key
    pub api_key: String,
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
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Get all FOEN station IDs
    pub fn foen_station_ids(&self) -> Vec<u32> {
        self.stations
            .iter()
            .map(|station| station.foen_station_id)
            .collect()
    }

    /// Get all Gfrörli sensor IDs
    pub fn gfroerli_sensor_ids(&self) -> Vec<u32> {
        self.stations
            .iter()
            .map(|station| station.gfroerli_sensor_id)
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
