//! Configuration management for the LINDAS FOEN fetcher

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Station configuration
    pub stations: StationConfig,
}

/// Station configuration section
#[derive(Debug, Deserialize, Serialize)]
pub struct StationConfig {
    /// List of station IDs to query for water temperature data
    pub ids: Vec<u32>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
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
            stations: StationConfig {
                ids: vec![2104, 2176],
            },
        };
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.stations.ids, deserialized.stations.ids);
    }

    #[test]
    fn test_config_file_operations() {
        let test_file = PathBuf::from("test_config.toml");
        let test_config = Config {
            stations: StationConfig {
                ids: vec![2104, 2176],
            },
        };

        // Clean up any existing test file
        let _ = fs::remove_file(&test_file);

        // Create test config file
        let toml_content = toml::to_string_pretty(&test_config).unwrap();
        fs::write(&test_file, toml_content).unwrap();

        // Load config from file
        let loaded_config = Config::load_from_file(&test_file).unwrap();
        assert_eq!(loaded_config.stations.ids, test_config.stations.ids);

        // Clean up
        fs::remove_file(&test_file).unwrap();
    }
}
