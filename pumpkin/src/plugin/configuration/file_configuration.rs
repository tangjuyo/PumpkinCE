use crate::plugin::configuration::Configuration;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Manages configuration files for plugins
pub struct FileConfiguration {
    plugin_name: String,
    data_folder: PathBuf,
    embedded_resource: Option<Vec<u8>>,
}

impl FileConfiguration {
    /// Create a new configuration manager for a plugin
    pub fn new(plugin_name: String, data_folder: PathBuf, embedded_resource: Option<Vec<u8>>) -> Self {
        Self {
            plugin_name,
            data_folder,
            embedded_resource,
        }
    }

    /// Load a configuration file, creating it from embedded resources if it doesn't exist
    pub async fn load_config(&self, filename: &str) -> Result<Configuration, String> {
        let config_path = self.data_folder.join(filename);

        // Ensure the data folder exists
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| format!("Failed to create data folder: {}", e))?;
            }
        }

        // Load the configuration from file if it exists, otherwise use embedded default
        let config_content = if config_path.exists() {
            fs::read_to_string(&config_path)
                .await
                .map_err(|e| format!("Failed to read config file: {}", e))?
        } else {
            // Try to load from embedded resource
            if let Some(embedded_content) = &self.embedded_resource {
                String::from_utf8(embedded_content.clone())
                    .map_err(|e| format!("Invalid UTF-8 in embedded config: {}", e))?
            } else {
                return Err(format!("No embedded resource found for {}", filename));
            }
        };

        // Parse the YAML content
        let config: serde_yaml::Value = serde_yaml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;

        // Convert to our Configuration format
        let data = if let serde_yaml::Value::Mapping(map) = config {
            map.into_iter()
                .filter_map(|(key, value)| {
                    key.as_str().map(|k| (k.to_string(), value))
                })
                .collect()
        } else {
            HashMap::new()
        };

        Ok(Configuration { data })
    }

    /// Save the default configuration file if it doesn't exist
    pub async fn save_default_config(&self, filename: &str) -> Result<(), String> {
        let config_path = self.data_folder.join(filename);

                if !config_path.exists() {
            if let Some(embedded_content) = &self.embedded_resource {
                // Ensure the parent directory exists
                if let Some(parent) = config_path.parent() {
                    fs::create_dir_all(parent)
                        .await
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }

                // Write the embedded content to the file
                fs::write(&config_path, embedded_content)
                    .await
                    .map_err(|e| format!("Failed to write config file: {}", e))?;

                log::info!("[{}] Created default config file: {}", self.plugin_name, filename);
            } else {
                return Err(format!("No embedded resource found for {}", filename));
            }
        }

        Ok(())
    }

    /// Save a resource file
    pub async fn save_resource(&self, filename: &str, replace: bool) -> Result<(), String> {
        let resource_path = self.data_folder.join(filename);

                if let Some(embedded_content) = &self.embedded_resource {
            // Ensure the parent directory exists
            if let Some(parent) = resource_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            // Save only if the file doesn't exist or if replace is true
            if replace || !resource_path.exists() {
                fs::write(&resource_path, embedded_content)
                    .await
                    .map_err(|e| format!("Failed to write resource file: {}", e))?;

                log::info!("[{}] Saved resource file: {}", self.plugin_name, filename);
            }
        } else {
            return Err(format!("No embedded resource found for {}", filename));
        }

        Ok(())
    }
}
