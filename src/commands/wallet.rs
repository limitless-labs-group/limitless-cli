use anyhow::{Context, Result};
use clap::Subcommand;

use crate::auth::resolve_private_key;
use crate::config::{load_config, save_config, Config};
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum WalletCommand {
    /// Create a new random wallet
    Create,
    /// Import an existing private key
    Import {
        /// Private key (hex, 0x-prefixed)
        private_key: String,
    },
    /// Show wallet address from stored key
    Show,
    /// Show wallet address only
    Address,
    /// Reset stored wallet configuration
    Reset,
}

pub async fn execute(
    command: &WalletCommand,
    output: &OutputFormat,
    _api_key_flag: Option<&str>,
    private_key_flag: Option<&str>,
) -> Result<()> {
    match command {
        WalletCommand::Create => {
            let key = alloy::signers::local::PrivateKeySigner::random();
            let address = key.address();
            let pk_hex = format!("0x{}", hex::encode(key.credential().to_bytes()));

            // Save to config
            let mut config = load_config().unwrap_or_default();
            config.private_key = Some(pk_hex.clone());
            save_config(&config)?;

            match output {
                OutputFormat::Json => {
                    crate::output::print_json(&serde_json::json!({
                        "address": format!("{}", address),
                        "private_key": pk_hex,
                    }))?;
                }
                OutputFormat::Table => {
                    println!("New wallet created!");
                    println!("Address:     {}", address);
                    println!("Private Key: {}", pk_hex);
                    println!();
                    println!("WARNING: Save your private key securely. It will not be shown again.");
                    println!("Key saved to config file.");
                }
            }
        }
        WalletCommand::Import { private_key } => {
            // Validate the key
            let pk = private_key
                .strip_prefix("0x")
                .unwrap_or(private_key);
            let bytes = hex::decode(pk).context("Invalid hex in private key")?;
            let signer = alloy::signers::local::PrivateKeySigner::from_slice(&bytes)
                .context("Invalid private key")?;
            let address = signer.address();

            let mut config = load_config().unwrap_or_default();
            config.private_key = Some(if private_key.starts_with("0x") {
                private_key.clone()
            } else {
                format!("0x{}", private_key)
            });
            save_config(&config)?;

            match output {
                OutputFormat::Json => {
                    crate::output::print_json(&serde_json::json!({
                        "address": format!("{}", address),
                        "imported": true,
                    }))?;
                }
                OutputFormat::Table => {
                    println!("Wallet imported!");
                    println!("Address: {}", address);
                    println!("Key saved to config file.");
                }
            }
        }
        WalletCommand::Show | WalletCommand::Address => {
            let pk_str = resolve_private_key(private_key_flag)?;
            let pk = pk_str.strip_prefix("0x").unwrap_or(&pk_str);
            let bytes = hex::decode(pk).context("Invalid hex in private key")?;
            let signer = alloy::signers::local::PrivateKeySigner::from_slice(&bytes)
                .context("Invalid private key")?;
            let address = signer.address();

            match output {
                OutputFormat::Json => {
                    let mut data = serde_json::json!({"address": format!("{}", address)});
                    if matches!(command, WalletCommand::Show) {
                        if let Some(config) = load_config() {
                            data["config_path"] =
                                serde_json::Value::String(crate::config::config_path().display().to_string());
                            data["has_api_key"] =
                                serde_json::Value::Bool(config.api_key.is_some());
                        }
                    }
                    crate::output::print_json(&data)?;
                }
                OutputFormat::Table => {
                    println!("Address: {}", address);
                    if matches!(command, WalletCommand::Show) {
                        if let Some(config) = load_config() {
                            println!(
                                "Config:  {}",
                                crate::config::config_path().display()
                            );
                            println!(
                                "API Key: {}",
                                if config.api_key.is_some() {
                                    "configured"
                                } else {
                                    "not set"
                                }
                            );
                        }
                    }
                }
            }
        }
        WalletCommand::Reset => {
            let config = Config::default();
            save_config(&config)?;
            match output {
                OutputFormat::Json => {
                    crate::output::print_json(&serde_json::json!({"reset": true}))?;
                }
                OutputFormat::Table => {
                    println!("Wallet configuration reset.");
                }
            }
        }
    }

    Ok(())
}
