use anyhow::{bail, Result};

use crate::config::load_config;

pub fn resolve_api_key(cli_flag: Option<&str>) -> Result<String> {
    if let Some(key) = cli_flag {
        return Ok(key.to_string());
    }
    if let Ok(key) = std::env::var("LIMITLESS_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    if let Some(config) = load_config() {
        if let Some(key) = config.api_key {
            return Ok(key);
        }
    }
    bail!("No API key found. Set via --api-key, LIMITLESS_API_KEY env var, or run `limitless setup`.")
}

pub fn resolve_private_key(cli_flag: Option<&str>) -> Result<String> {
    if let Some(key) = cli_flag {
        return Ok(key.to_string());
    }
    if let Ok(key) = std::env::var("LIMITLESS_PRIVATE_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    if let Some(config) = load_config() {
        if let Some(key) = config.private_key {
            return Ok(key);
        }
    }
    bail!("No private key found. Set via --private-key, LIMITLESS_PRIVATE_KEY env var, or run `limitless setup`.")
}
