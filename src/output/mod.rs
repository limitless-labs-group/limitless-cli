pub mod markets;
pub mod orderbook;
pub mod portfolio;
pub mod profiles;
pub mod trading;

use anyhow::Result;
use clap::ValueEnum;
use colored::Colorize;
use rust_decimal::Decimal;
use serde::Serialize;
use tabled::settings::object::Columns;
use tabled::settings::{Modify, Style, Width};
use tabled::{Table, Tabled};

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

pub fn success(msg: &str) -> String {
    format!("{} {}", "✓".green(), msg.green())
}

pub fn warn(msg: &str) -> String {
    format!("{} {}", "⚠".yellow(), msg.yellow())
}

pub fn label(msg: &str) -> String {
    msg.cyan().bold().to_string()
}

pub fn dim(msg: &str) -> String {
    msg.dimmed().to_string()
}

pub fn price_green(msg: &str) -> String {
    msg.green().to_string()
}

pub fn price_red(msg: &str) -> String {
    msg.red().to_string()
}

pub fn pnl_color(value: f64, formatted: &str) -> String {
    if value > 0.0 {
        formatted.green().to_string()
    } else if value < 0.0 {
        formatted.red().to_string()
    } else {
        formatted.dimmed().to_string()
    }
}

pub fn print_json<T: Serialize>(data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}

pub fn print_table<T: Tabled>(rows: &[T]) {
    if rows.is_empty() {
        println!("{}", "No results.".dimmed());
        return;
    }
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{}", table);
}

pub fn print_detail_table(rows: Vec<(&str, String)>) {
    if rows.is_empty() {
        println!("{}", "No data.".dimmed());
        return;
    }

    #[derive(Tabled)]
    struct Row {
        #[tabled(rename = "Field")]
        field: String,
        #[tabled(rename = "Value")]
        value: String,
    }

    let table_rows: Vec<Row> = rows
        .into_iter()
        .map(|(f, v)| Row {
            field: f.to_string(),
            value: v,
        })
        .collect();

    let table = Table::new(&table_rows)
        .with(Style::rounded())
        .with(Modify::new(Columns::single(1)).with(Width::wrap(80).keep_words(true)))
        .to_string();
    println!("{}", table);
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut truncated: String = s.chars().take(max.saturating_sub(1)).collect();
    truncated.push('\u{2026}');
    truncated
}

pub fn format_decimal(value: Decimal) -> String {
    let thousand = Decimal::from(1_000);
    let million = Decimal::from(1_000_000);

    if value >= million {
        format!("{:.1}M", value / million)
    } else if value >= thousand {
        format!("{:.1}K", value / thousand)
    } else {
        format!("{:.2}", value)
    }
}

pub fn format_optional_decimal(value: &Option<Decimal>) -> String {
    match value {
        Some(v) => format_decimal(*v),
        None => "-".to_string(),
    }
}

pub fn format_optional_price(value: &Option<Decimal>) -> String {
    match value {
        Some(v) => format!("${:.2}", v),
        None => "-".to_string(),
    }
}
