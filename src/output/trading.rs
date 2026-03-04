use colored::Colorize;
use tabled::Tabled;

use crate::client::trading::{LockedBalance, UserOrder};
use crate::output::{print_detail_table, print_table, truncate};

#[derive(Tabled)]
struct UserOrderRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Side")]
    side: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Remaining")]
    remaining: String,
    #[tabled(rename = "Type")]
    order_type: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

/// Format atomic amount (1e6 scale) to human-readable shares
fn format_atomic_size(raw: &Option<String>) -> String {
    raw.as_deref()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|v| format!("{:.2}", v / 1_000_000.0))
        .unwrap_or_else(|| "-".into())
}

pub fn print_user_orders_table(orders: &[UserOrder]) {
    let rows: Vec<UserOrderRow> = orders
        .iter()
        .map(|o| {
            let side_colored = match o.side.to_uppercase().as_str() {
                "BUY" => "BUY".green().to_string(),
                "SELL" => "SELL".red().to_string(),
                _ => o.side.clone(),
            };
            let status_colored = match o.status.to_uppercase().as_str() {
                "OPEN" | "ACTIVE" => o.status.green().to_string(),
                "FILLED" | "MATCHED" => o.status.cyan().to_string(),
                "CANCELLED" | "CANCELED" => o.status.dimmed().to_string(),
                _ => o.status.clone(),
            };
            UserOrderRow {
                id: truncate(&o.id, 12).dimmed().to_string(),
                side: side_colored,
                price: o.price.clone(),
                size: format_atomic_size(&o.original_size),
                remaining: format_atomic_size(&o.remaining_size),
                order_type: o.order_type.clone().unwrap_or_else(|| "-".to_string()),
                status: status_colored,
                created_at: o
                    .created_at
                    .as_deref()
                    .map(|s| s.split('T').next().unwrap_or(s).to_string())
                    .unwrap_or_else(|| "-".into()),
            }
        })
        .collect();
    print_table(&rows);
}

/// Parse a raw atomic value (string) from the API, dividing by 1e6 to get human units.
fn parse_atomic_raw(v: &serde_json::Value) -> f64 {
    v.as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| v.as_f64())
        .unwrap_or(0.0)
        / 1_000_000.0
}

pub fn print_order_created(resp: &serde_json::Value) {
    let order = &resp["order"];
    let exec = &resp["execution"];

    let side_num = order["side"].as_u64().unwrap_or(99);
    let side = match side_num {
        0 => "BUY",
        1 => "SELL",
        _ => "?",
    };

    let matched = exec["matched"].as_bool().unwrap_or(false);

    let market_title = order["market"]["title"]
        .as_str()
        .unwrap_or("-");

    let order_type = order["orderType"]
        .as_str()
        .unwrap_or("GTC");

    let is_fok = order_type == "FOK";

    let order_id = order["id"]
        .as_str()
        .unwrap_or("-");

    let created_at = order["createdAt"]
        .as_str()
        .unwrap_or("-");

    // Parse execution data
    let totals = &exec["totalsRaw"];
    let shares_filled = parse_atomic_raw(&totals["contractsGross"]);
    let usd_gross = parse_atomic_raw(&totals["usdGross"]);
    let usd_fee = parse_atomic_raw(&totals["usdFee"]);
    let usd_net = parse_atomic_raw(&totals["usdNet"]);
    let avg_price = if shares_filled > 0.0 {
        usd_gross / shares_filled
    } else {
        0.0
    };

    if matched {
        let action = if side_num == 0 {
            "Bought".green().bold().to_string()
        } else {
            "Sold".red().bold().to_string()
        };
        println!(
            "\n  {} {:.6} shares at {:.4} USDC/share\n",
            action, shares_filled, avg_price
        );

        let mut rows = vec![
            ("Market", market_title.to_string()),
            ("Order ID", truncate(order_id, 20)),
            ("Side", side.to_string()),
            ("Type", order_type.to_string()),
            ("Shares", format!("{:.6}", shares_filled)),
            ("Avg Price", format!("{:.4} USDC", avg_price)),
        ];

        if side_num == 0 {
            // BUY: you spent USDC to get shares
            rows.push(("Spent", format!("{:.6} USDC", usd_gross)));
        } else {
            // SELL: you sold shares to get USDC
            rows.push(("Received", format!("{:.6} USDC", usd_gross)));
        }

        rows.push(("Fee", format!("{:.6} USDC", usd_fee)));
        rows.push(("Net", format!("{:.6} USDC", usd_net)));
        rows.push(("Created", created_at.to_string()));

        print_detail_table(rows);
    } else {
        println!("\n  {} {}\n", "Order OPEN".yellow().bold(), "(resting on book)".dimmed());

        let maker_amount = order["makerAmount"].as_u64().unwrap_or(0) as f64 / 1_000_000.0;

        let mut rows = vec![
            ("Market", market_title.to_string()),
            ("Order ID", truncate(order_id, 20)),
            ("Side", side.to_string()),
        ];

        if is_fok {
            let label = if side_num == 0 { "Amount" } else { "Shares" };
            let unit = if side_num == 0 { " USDC" } else { "" };
            rows.push((label, format!("{:.6}{}", maker_amount, unit)));
        } else {
            let price = order["price"]
                .as_f64()
                .map(|p| format!("{:.4}", p))
                .unwrap_or_else(|| "-".into());
            let taker_amount =
                order["takerAmount"].as_u64().unwrap_or(0) as f64 / 1_000_000.0;
            rows.push(("Price", format!("{} USDC", price)));
            rows.push(("Shares", format!("{:.2}", taker_amount)));
            rows.push(("Cost", format!("{:.6} USDC", maker_amount)));
        }

        let fee_bps = order["feeRateBps"].as_u64().unwrap_or(0);
        rows.push(("Type", order_type.to_string()));
        rows.push((
            "Fee Rate",
            format!("{} bps ({}%)", fee_bps, fee_bps as f64 / 100.0),
        ));
        rows.push(("Created", created_at.to_string()));

        print_detail_table(rows);
    }
}

pub fn print_locked_balance(balance: &LockedBalance) {
    let formatted = balance
        .locked_balance_formatted
        .as_deref()
        .or(balance.locked_balance.as_deref())
        .unwrap_or("0");
    let count = balance.order_count.unwrap_or(0);
    let currency = balance.currency.as_deref().unwrap_or("USDC");
    println!("{} {} {}", "Locked:".cyan(), formatted.bold(), currency);
    println!("{} {}", "Open orders:".cyan(), count.to_string().bold());
}
