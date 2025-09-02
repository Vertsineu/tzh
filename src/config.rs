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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: "https://api.deepseek.com/v1".to_string(),
            api_key: None,
            model: "deepseek-chat".to_string(),
            timeout: 30,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
            let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
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
}
