use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::constants::{API_BASE_URL, CHAIN_ID, DEFAULT_RPC_URL, WS_URL};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    #[serde(default = "default_rpc_url")]
    pub rpc_url: String,
    #[serde(default = "default_api_url")]
    pub api_url: String,
    #[serde(default = "default_ws_url")]
    pub ws_url: String,
}

fn default_chain_id() -> u64 {
    CHAIN_ID
}
fn default_rpc_url() -> String {
    DEFAULT_RPC_URL.to_string()
}
fn default_api_url() -> String {
    API_BASE_URL.to_string()
}
fn default_ws_url() -> String {
    WS_URL.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            private_key: None,
            chain_id: CHAIN_ID,
            rpc_url: DEFAULT_RPC_URL.to_string(),
            api_url: API_BASE_URL.to_string(),
            ws_url: WS_URL.to_string(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("limitless")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config() -> Option<Config> {
    let path = config_path();
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_config(config: &Config) -> Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir).context("Failed to create config directory")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o700);
        fs::set_permissions(&dir, perms).ok();
    }

    let path = config_path();
    let data = serde_json::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&path, data).context("Failed to write config file")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms).ok();
    }

    Ok(())
}
