//! Plugin configuration system for Pumpkin
//!
//! This module provides functionality for plugins to manage their configuration files,
//! similar to Paper's saveDefaultConfig() and getConfig() system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod file_configuration;

pub use file_configuration::FileConfiguration;

/// Macro to include a resource file at compile time
#[macro_export]
macro_rules! include_plugin_resource {
    ($filename:expr) => {
        include_bytes!(concat!("resources/", $filename))
    };
}

/// Represents a plugin configuration loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    /// The raw configuration data as a HashMap
    pub data: HashMap<String, serde_yaml::Value>,
}

impl Configuration {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Get a string value from the configuration
    pub fn get_string(&self, path: &str) -> Option<String> {
        self.get_value(path).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    /// Get a string value with a default
    pub fn get_string_or(&self, path: &str, default: &str) -> String {
        self.get_string(path).unwrap_or_else(|| default.to_string())
    }

    /// Get an integer value from the configuration
    pub fn get_int(&self, path: &str) -> Option<i64> {
        self.get_value(path).and_then(|v| v.as_i64())
    }

    /// Get an integer value with a default
    pub fn get_int_or(&self, path: &str, default: i64) -> i64 {
        self.get_int(path).unwrap_or(default)
    }

    /// Get a boolean value from the configuration
    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get_value(path).and_then(|v| v.as_bool())
    }

    /// Get a boolean value with a default
    pub fn get_bool_or(&self, path: &str, default: bool) -> bool {
        self.get_bool(path).unwrap_or(default)
    }

    /// Get a list of strings from the configuration
    pub fn get_string_list(&self, path: &str) -> Option<Vec<String>> {
        self.get_value(path).and_then(|v| {
            v.as_sequence().map(|seq| {
                seq.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
        })
    }

    /// Get a nested configuration section
    pub fn get_section(&self, path: &str) -> Option<Configuration> {
        self.get_value(path).and_then(|v| {
            v.as_mapping().map(|map| {
                let mut data = HashMap::new();
                for (key, value) in map {
                    if let Some(key_str) = key.as_str() {
                        data.insert(key_str.to_string(), value.clone());
                    }
                }
                Configuration { data }
            })
        })
    }

                /// Get a raw value from the configuration
    fn get_value(&self, path: &str) -> Option<&serde_yaml::Value> {
        let parts: Vec<&str> = path.split('.').collect();

        // For now, just handle top-level keys
        if parts.len() == 1 {
            return self.data.get(parts[0]);
        }

        // For nested paths, we'll need a more complex implementation
        // For now, return None for nested paths
        None
    }

        /// Set a value in the configuration
    pub fn set(&mut self, path: &str, value: serde_yaml::Value) {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return;
        }

        let current_map = &mut self.data;

        // For now, just set at the top level
        if let Some(last_part) = parts.last() {
            current_map.insert(last_part.to_string(), value);
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for plugins that need configuration management
#[async_trait::async_trait]
pub trait ConfigurablePlugin {
    /// Get the plugin's data folder path
    fn get_data_folder(&self) -> PathBuf;

    /// Get the plugin's name
    fn get_plugin_name(&self) -> &str;

    /// Get an embedded resource from the plugin
    fn get_embedded_resource(&self, filename: &str) -> Option<Vec<u8>>;

    /// Save the default configuration file if it doesn't exist
    async fn save_default_config(&self, filename: &str) -> Result<(), String> {
        let config_manager = FileConfiguration::new(
            self.get_plugin_name().to_string(),
            self.get_data_folder(),
            self.get_embedded_resource(filename),
        );

        config_manager.save_default_config(filename)
            .await
            .map_err(|e| e.to_string())
    }

    /// Save a resource file if it doesn't exist
    async fn save_resource(&self, filename: &str, replace: bool) -> Result<(), String> {
        let config_manager = FileConfiguration::new(
            self.get_plugin_name().to_string(),
            self.get_data_folder(),
            self.get_embedded_resource(filename),
        );

        config_manager.save_resource(filename, replace)
            .await
            .map_err(|e| e.to_string())
    }

    /// Load a configuration file
    async fn load_config(&self, filename: &str) -> Result<Configuration, String> {
        let config_manager = FileConfiguration::new(
            self.get_plugin_name().to_string(),
            self.get_data_folder(),
            self.get_embedded_resource(filename),
        );

        config_manager.load_config(filename)
            .await
            .map_err(|e| e.to_string())
    }

    /// Get the main configuration (config.yml)
    async fn get_config(&self) -> Result<Configuration, String> {
        self.load_config("config.yml").await
    }
}
