use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};

use crate::config::{config_path, load_config, save_config};

pub async fn execute() -> Result<()> {
    println!();
    println!("  {}",  "Limitless CLI — Setup Wizard".cyan().bold());
    println!("  {}", "─".repeat(35).dimmed());
    println!();

    let mut config = load_config().unwrap_or_default();
    let is_update = config.api_key.is_some() || config.private_key.is_some();

    if is_update {
        println!("  Existing config found at {}", config_path().display().to_string().dimmed());
        println!("  Press Enter to keep current value, or type a new one.");
        println!();
    } else {
        println!("  Get your API key from {}", "https://limitless.exchange".underline());
        println!("  {}", "(Profile menu → Api keys)".dimmed());
        println!();
    }

    let current_api = config
        .api_key
        .as_deref()
        .map(mask_key)
        .unwrap_or_else(|| "(not set)".to_string());

    let api_key_input = prompt(&format!("API key [{}]", current_api.dimmed()))?;
    if !api_key_input.is_empty() {
        config.api_key = Some(api_key_input);
    }

    let current_pk = config
        .private_key
        .as_deref()
        .map(mask_key)
        .unwrap_or_else(|| "(not set)".to_string());

    let pk_input = prompt(&format!("Private key [{}]", current_pk.dimmed()))?;
    if !pk_input.is_empty() {
        let pk = pk_input.trim().to_string();
        if !pk.starts_with("0x") || pk.len() != 66 {
            println!();
            println!("  {}", "⚠ Warning: private key should be 0x-prefixed, 64 hex chars (66 total).".yellow());
            let confirm = prompt(&format!("  Save anyway? {}", "(y/N)".dimmed()))?;
            if confirm.to_lowercase() != "y" {
                println!("  {}", "Skipped private key.".dimmed());
            } else {
                config.private_key = Some(pk);
            }
        } else {
            config.private_key = Some(pk);
        }
    }

    save_config(&config).context("Failed to save config")?;

    println!();
    println!("  {} Config saved to {}", "✓".green(), config_path().display().to_string().dimmed());
    println!();

    let has_api = config.api_key.is_some();
    let has_pk = config.private_key.is_some();

    println!("  {}  {}", "API key:".cyan(), if has_api { "configured".green().to_string() } else { "not set".red().to_string() });
    println!("  {}  {}", "Private key:".cyan(), if has_pk { "configured".green().to_string() } else { "not set".red().to_string() });
    println!();

    if has_api {
        println!("  You can now run:  {}", "limitless markets list".bold());
        println!("                    {}", "limitless portfolio positions".bold());
    }
    if has_api && has_pk {
        println!("                    {}", "limitless trading create --slug <slug> ...".bold());
    }
    if !has_api {
        println!("  Run {} again to add your API key.", "limitless setup".bold());
    }

    Ok(())
}

fn prompt(label: &str) -> Result<String> {
    print!("  {} ", label);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        "****".to_string()
    } else {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    }
}
