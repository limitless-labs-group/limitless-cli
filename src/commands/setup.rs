use anyhow::{Context, Result};
use std::io::{self, Write};

use crate::config::{config_path, load_config, save_config};

pub async fn execute() -> Result<()> {
    println!("┌─────────────────────────────────────────┐");
    println!("│  Limitless CLI — Setup Wizard            │");
    println!("└─────────────────────────────────────────┘");
    println!();

    // Load existing config or start fresh
    let mut config = load_config().unwrap_or_default();
    let is_update = config.api_key.is_some() || config.private_key.is_some();

    if is_update {
        println!("  Existing config found at {}", config_path().display());
        println!("  Press Enter to keep current value, or type a new one.");
        println!();
    } else {
        println!("  Get your API key from https://limitless.exchange");
        println!("  (Profile menu → Api keys)");
        println!();
    }

    // ── API Key ──────────────────────────────────────────────────────

    let current_api = config
        .api_key
        .as_deref()
        .map(mask_key)
        .unwrap_or_else(|| "(not set)".to_string());

    let api_key_input = prompt(&format!("API key [{}]", current_api))?;
    if !api_key_input.is_empty() {
        config.api_key = Some(api_key_input);
    }

    // ── Private Key ──────────────────────────────────────────────────

    let current_pk = config
        .private_key
        .as_deref()
        .map(mask_key)
        .unwrap_or_else(|| "(not set)".to_string());

    let pk_input = prompt(&format!("Private key [{}]", current_pk))?;
    if !pk_input.is_empty() {
        let pk = pk_input.trim().to_string();
        // Basic validation: should be 0x-prefixed hex, 66 chars
        if !pk.starts_with("0x") || pk.len() != 66 {
            println!();
            println!("  ⚠ Warning: private key should be 0x-prefixed, 64 hex chars (66 total).");
            let confirm = prompt("  Save anyway? (y/N)")?;
            if confirm.to_lowercase() != "y" {
                println!("  Skipped private key.");
            } else {
                config.private_key = Some(pk);
            }
        } else {
            config.private_key = Some(pk);
        }
    }

    // ── Save ─────────────────────────────────────────────────────────

    save_config(&config).context("Failed to save config")?;

    println!();
    println!("  ✓ Config saved to {}", config_path().display());
    println!();

    // Show summary
    let has_api = config.api_key.is_some();
    let has_pk = config.private_key.is_some();

    println!("  API key:     {}", if has_api { "configured" } else { "not set" });
    println!("  Private key: {}", if has_pk { "configured" } else { "not set" });
    println!();

    if has_api {
        println!("  You can now run:  limitless markets list");
        println!("                    limitless portfolio positions");
    }
    if has_api && has_pk {
        println!("                    limitless trading create --slug <slug> ...");
    }
    if !has_api {
        println!("  Run `limitless setup` again to add your API key.");
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
