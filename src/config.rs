use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub model: String,
    pub timeout: u64,
    pub temperature: f32,
    pub max_tokens: Option<i32>,
}

// Partial config struct for loading from file with missing fields
#[derive(Debug, Deserialize)]
struct PartialConfig {
    endpoint: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    timeout: Option<u64>,
    temperature: Option<f32>,
    max_tokens: Option<Option<i32>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: "https://api.deepseek.com/v1".to_string(),
            api_key: None,
            model: "deepseek-chat".to_string(),
            timeout: 30,
            temperature: 1.3,
            max_tokens: Some(2000),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

            // Try to parse as partial config first, then merge with defaults
            let partial: PartialConfig =
                toml::from_str(&content).context("Failed to parse config file")?;

            let default = Config::default();
            let config = Config {
                endpoint: partial.endpoint.unwrap_or(default.endpoint),
                api_key: partial.api_key.or(default.api_key),
                model: partial.model.unwrap_or(default.model),
                timeout: partial.timeout.unwrap_or(default.timeout),
                temperature: partial.temperature.unwrap_or(default.temperature),
                max_tokens: partial.max_tokens.unwrap_or(default.max_tokens),
            };

            // Save the merged config to ensure all fields are present in the file
            config.save()?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("tzh");
        Ok(config_dir.join("config.toml"))
    }

    // Getters
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn timeout(&self) -> u64 {
        self.timeout
    }

    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    pub fn max_tokens(&self) -> Option<i32> {
        self.max_tokens
    }

    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some() && !self.api_key.as_ref().unwrap().is_empty()
    }

    // Setters
    pub fn set_endpoint(&mut self, endpoint: &str) {
        self.endpoint = endpoint.to_string();
    }

    pub fn set_api_key(&mut self, api_key: &str) {
        self.api_key = Some(api_key.to_string());
    }

    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = temperature;
    }

    pub fn set_max_tokens(&mut self, max_tokens: Option<i32>) {
        self.max_tokens = max_tokens;
    }
}
