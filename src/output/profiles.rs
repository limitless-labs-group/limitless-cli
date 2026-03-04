use anyhow::Result;
use colored::Colorize;

use crate::output::portfolio::{print_pnl_summary, print_positions_table};

/// Public positions — reuse the same table format
pub fn print_public_positions(data: &serde_json::Value) -> Result<()> {
    print_positions_table(data, "all")
}

/// Public PnL — reuse the same summary format
pub fn print_public_pnl(data: &serde_json::Value) -> Result<()> {
    print_pnl_summary(data)
}

/// Public traded volume summary
pub fn print_public_volume(data: &serde_json::Value) -> Result<()> {
    if data.is_null() || (data.is_object() && data.as_object().map_or(true, |o| o.is_empty())) {
        println!("No volume data.");
        return Ok(());
    }

    println!("{}", "Traded Volume".cyan().bold());
    println!("{}", "─────────────────────────".dimmed());

    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            let val_str = match value {
                v if v.is_string() => v.as_str().unwrap_or("-").to_string(),
                v if v.is_f64() => format!("{:.2}", v.as_f64().unwrap_or(0.0)),
                v if v.is_i64() => v.as_i64().unwrap_or(0).to_string(),
                v if v.is_u64() => v.as_u64().unwrap_or(0).to_string(),
                _ => format!("{}", value),
            };
            let label = camel_to_title(key);
            println!("{:<20}{}", format!("{}:", label), val_str);
        }
    } else {
        println!("Volume: {}", data);
    }

    Ok(())
}

pub fn print_public_data(label: &str, data: &serde_json::Value) -> Result<()> {
    println!("--- {} ---", label);
    crate::output::print_json(data)?;
    Ok(())
}

fn camel_to_title(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push(' ');
        }
        if i == 0 {
            result.extend(c.to_uppercase());
        } else {
            result.push(c);
        }
    }
    result
}
