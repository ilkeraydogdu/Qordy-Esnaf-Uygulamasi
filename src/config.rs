//! QORDY POS - Runtime Configuration System
//! Loads configuration from config.toml file or environment variables.
//! Environment variables take precedence over file values.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// Application configuration loaded from config.toml or environment variables
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_cloud_url")]
    pub cloud_url: String,

    #[serde(default)]
    pub api_key: String,

    #[serde(default)]
    pub tenant_id: String,

    #[serde(default = "default_register_id")]
    pub register_id: String,

    #[serde(default = "default_cashier_name")]
    pub cashier_name: String,

    #[serde(default = "default_cashier_id")]
    pub cashier_id: String,

    #[serde(default = "default_branch_id")]
    pub branch_id: String,

    #[serde(default = "default_branch_name")]
    pub branch_name: String,

    #[serde(default = "default_business_name")]
    pub business_name: String,

    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_cloud_url() -> String {
    "https://qordy.com".to_string()
}

fn default_register_id() -> String {
    "KASA-01".to_string()
}

fn default_cashier_name() -> String {
    "Ahmet Kasiyer".to_string()
}

fn default_cashier_id() -> String {
    "USR_01".to_string()
}

fn default_branch_id() -> String {
    "BR_01".to_string()
}

fn default_branch_name() -> String {
    "Kadıköy Çarşı Şubesi".to_string()
}

fn default_business_name() -> String {
    "QORDY Cadde Esnaf".to_string()
}

fn default_port() -> u16 {
    8090
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cloud_url: default_cloud_url(),
            api_key: String::new(),
            tenant_id: String::new(),
            register_id: default_register_id(),
            cashier_name: default_cashier_name(),
            cashier_id: default_cashier_id(),
            branch_id: default_branch_id(),
            branch_name: default_branch_name(),
            business_name: default_business_name(),
            port: default_port(),
        }
    }
}

impl AppConfig {
    /// Load configuration from config.toml file and/or environment variables.
    /// Environment variables override file values.
    ///
    /// Search order for config.toml:
    /// 1. Current working directory
    /// 2. Application data directory (~/Library/Application Support/QordyPos on macOS)
    pub fn load() -> Result<Self> {
        let config_path = find_config_file();

        let mut config = if let Some(path) = &config_path {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", path.display()))?
        } else {
            tracing::info!("No config.toml found, using defaults + environment variables");
            AppConfig::default()
        };

        // Environment variables override file values
        config.apply_env_overrides();

        // Validate required fields
        config.validate()?;

        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("QORDY_CLOUD_URL") {
            self.cloud_url = val;
        }
        if let Ok(val) = std::env::var("QORDY_API_KEY") {
            self.api_key = val;
        }
        if let Ok(val) = std::env::var("QORDY_TENANT_ID") {
            self.tenant_id = val;
        }
        if let Ok(val) = std::env::var("QORDY_REGISTER_ID") {
            self.register_id = val;
        }
        if let Ok(val) = std::env::var("QORDY_CASHIER_NAME") {
            self.cashier_name = val;
        }
        if let Ok(val) = std::env::var("QORDY_CASHIER_ID") {
            self.cashier_id = val;
        }
        if let Ok(val) = std::env::var("QORDY_BRANCH_ID") {
            self.branch_id = val;
        }
        if let Ok(val) = std::env::var("QORDY_BRANCH_NAME") {
            self.branch_name = val;
        }
        if let Ok(val) = std::env::var("QORDY_BUSINESS_NAME") {
            self.business_name = val;
        }
        if let Ok(val) = std::env::var("QORDY_PORT") {
            if let Ok(port) = val.parse::<u16>() {
                self.port = port;
            }
        }
    }

    /// Validate that required configuration values are present
    fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            anyhow::bail!(
                "API key is not configured. Set it in config.toml or via QORDY_API_KEY environment variable."
            );
        }
        if self.tenant_id.is_empty() {
            anyhow::bail!(
                "Tenant ID is not configured. Set it in config.toml or via QORDY_TENANT_ID environment variable."
            );
        }
        Ok(())
    }
}

/// Find config.toml in standard locations
fn find_config_file() -> Option<PathBuf> {
    // Check current working directory first
    let cwd_config = PathBuf::from("config.toml");
    if cwd_config.exists() {
        return Some(cwd_config);
    }

    // Check application data directory
    if let Some(data_dir) = get_data_dir() {
        let data_config = data_dir.join("config.toml");
        if data_config.exists() {
            return Some(data_config);
        }
    }

    None
}

fn get_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/QordyPos"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("QordyPos"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config/QordyPos"))
    }
}