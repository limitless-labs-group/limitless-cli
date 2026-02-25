use anyhow::Result;
use rust_decimal::Decimal;
use tabled::Tabled;

use crate::output::print_json;

/// USDC has 6 decimals
const SCALE: i64 = 1_000_000;

fn raw_to_usdc(raw: &str) -> String {
    match raw.parse::<i64>() {
        Ok(v) => {
            let whole = v / SCALE;
            let frac = (v % SCALE).unsigned_abs();
            if v < 0 && whole == 0 {
                format!("-{}.{:02}", whole.unsigned_abs(), frac / 10_000)
            } else {
                format!("{}.{:02}", whole, frac / 10_000)
            }
        }
        Err(_) => raw.to_string(),
    }
}

fn raw_to_shares(raw: &str) -> String {
    match raw.parse::<i64>() {
        Ok(v) => {
            let d = Decimal::from(v) / Decimal::from(SCALE);
            format!("{:.2}", d)
        }
        Err(_) => raw.to_string(),
    }
}

#[derive(Tabled)]
struct PositionRow {
    #[tabled(rename = "Market")]
    market: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Side")]
    side: String,
    #[tabled(rename = "Shares")]
    shares: String,
    #[tabled(rename = "Avg Price")]
    avg_price: String,
    #[tabled(rename = "Mkt Value")]
    market_value: String,
    #[tabled(rename = "Cost")]
    cost: String,
    #[tabled(rename = "PnL")]
    pnl: String,
    #[tabled(rename = "Deadline")]
    deadline: String,
}

pub fn print_positions_table(data: &serde_json::Value, status_filter: &str) -> Result<()> {
    let mut rows: Vec<PositionRow> = Vec::new();

    // Process CLOB positions
    if let Some(clob) = data.get("clob").and_then(|v| v.as_array()) {
        for pos in clob {
            let market = pos.get("market").unwrap_or(&serde_json::Value::Null);
            let title = market
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let status = market
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("-");

            // Apply status filter
            match status_filter.to_lowercase().as_str() {
                "funded" | "open" | "active" => {
                    if status.to_uppercase() != "FUNDED" {
                        continue;
                    }
                }
                "resolved" | "closed" => {
                    if status.to_uppercase() != "RESOLVED" {
                        continue;
                    }
                }
                _ => {} // "all" — show everything
            }

            let deadline = market
                .get("expirationDate")
                .or_else(|| market.get("deadline"))
                .and_then(|v| v.as_str())
                .unwrap_or("-");

            let tokens_balance = pos.get("tokensBalance").unwrap_or(&serde_json::Value::Null);
            let positions = pos.get("positions").unwrap_or(&serde_json::Value::Null);

            let yes_balance = tokens_balance
                .get("yes")
                .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|_| "0")))
                .unwrap_or("0");
            let no_balance = tokens_balance
                .get("no")
                .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|_| "0")))
                .unwrap_or("0");

            let yes_bal: i64 = yes_balance.parse().unwrap_or(0);
            let no_bal: i64 = no_balance.parse().unwrap_or(0);

            if yes_bal > 0 {
                let yes_pos = &positions["yes"];
                let cost = yes_pos.get("cost").and_then(|v| v.as_str()).unwrap_or("0");
                let fill_price = yes_pos.get("fillPrice").and_then(|v| v.as_str()).unwrap_or("0");
                let mkt_value = yes_pos.get("marketValue").and_then(|v| v.as_str()).unwrap_or("0");
                let realised = yes_pos.get("realisedPnl").and_then(|v| v.as_str()).unwrap_or("0");
                let unrealised = yes_pos.get("unrealizedPnl").and_then(|v| v.as_str()).unwrap_or("0");
                let total_pnl: i64 = realised.parse::<i64>().unwrap_or(0)
                    + unrealised.parse::<i64>().unwrap_or(0);

                rows.push(PositionRow {
                    market: truncate_title(title, 45),
                    status: status.to_string(),
                    side: "YES".to_string(),
                    shares: raw_to_shares(yes_balance),
                    avg_price: format_fill_price(fill_price),
                    market_value: raw_to_usdc(mkt_value),
                    cost: raw_to_usdc(cost),
                    pnl: raw_to_usdc(&total_pnl.to_string()),
                    deadline: format_deadline_with_time(deadline),
                });
            }

            if no_bal > 0 {
                let no_pos = &positions["no"];
                let cost = no_pos.get("cost").and_then(|v| v.as_str()).unwrap_or("0");
                let fill_price = no_pos.get("fillPrice").and_then(|v| v.as_str()).unwrap_or("0");
                let mkt_value = no_pos.get("marketValue").and_then(|v| v.as_str()).unwrap_or("0");
                let realised = no_pos.get("realisedPnl").and_then(|v| v.as_str()).unwrap_or("0");
                let unrealised = no_pos.get("unrealizedPnl").and_then(|v| v.as_str()).unwrap_or("0");
                let total_pnl: i64 = realised.parse::<i64>().unwrap_or(0)
                    + unrealised.parse::<i64>().unwrap_or(0);

                rows.push(PositionRow {
                    market: truncate_title(title, 45),
                    status: status.to_string(),
                    side: "NO".to_string(),
                    shares: raw_to_shares(no_balance),
                    avg_price: format_fill_price(fill_price),
                    market_value: raw_to_usdc(mkt_value),
                    cost: raw_to_usdc(cost),
                    pnl: raw_to_usdc(&total_pnl.to_string()),
                    deadline: format_deadline_with_time(deadline),
                });
            }
        }
    }

    // Process AMM positions
    if let Some(amm) = data.get("amm").and_then(|v| v.as_array()) {
        for pos in amm {
            let market = pos.get("market").unwrap_or(&serde_json::Value::Null);
            let title = market
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let status = market
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("AMM");
            match status_filter.to_lowercase().as_str() {
                "funded" | "open" | "active" => {
                    if status.to_uppercase() != "FUNDED" {
                        continue;
                    }
                }
                "resolved" | "closed" => {
                    if status.to_uppercase() != "RESOLVED" {
                        continue;
                    }
                }
                _ => {}
            }
            rows.push(PositionRow {
                market: truncate_title(title, 45),
                status: status.to_string(),
                side: "-".to_string(),
                shares: "-".to_string(),
                avg_price: "-".to_string(),
                market_value: "-".to_string(),
                cost: "-".to_string(),
                pnl: "-".to_string(),
                deadline: "-".to_string(),
            });
        }
    }

    if rows.is_empty() {
        if status_filter != "all" {
            println!("No positions matching status '{}'.", status_filter);
        } else {
            println!("No open positions.");
        }
    } else {
        if let Some(points) = data.get("points").and_then(|v| v.as_str()) {
            if points != "0" && points != "0.00000000" {
                println!("Points: {}", points);
            }
        }
        println!("{} position(s):", rows.len());
        println!();
        crate::output::print_table(&rows);
    }

    Ok(())
}

/// Formatted PnL summary
pub fn print_pnl_summary(data: &serde_json::Value) -> Result<()> {
    let timeframe = data
        .get("timeframe")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let current_value = data
        .get("currentValue")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let previous_value = data
        .get("previousValue")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let pct_change = data
        .get("percentChange")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    // Realised PnL from current.realised
    let realised_formatted = data
        .get("current")
        .and_then(|c| c.get("realised"))
        .and_then(|r| r.get("formatted"))
        .and_then(|v| v.as_str())
        .unwrap_or("0");

    println!("PnL Summary ({})", timeframe);
    println!("─────────────────────────");
    println!("Current Value:  ${:.2}", current_value);
    println!("Previous Value: ${:.2}", previous_value);
    println!("Change:         {:.2}%", pct_change);
    println!("Realised PnL:   {} USDC", realised_formatted);

    // Show data points count
    if let Some(points) = data.get("data").and_then(|v| v.as_array()) {
        println!("Data Points:    {}", points.len());
    }

    Ok(())
}

/// Formatted trades table
pub fn print_trades_table(data: &serde_json::Value) -> Result<()> {
    // Trades may be an array directly, or wrapped in { data: [...] } or { trades: [...] }
    let trades = data
        .as_array()
        .or_else(|| data.get("data").and_then(|v| v.as_array()))
        .or_else(|| data.get("trades").and_then(|v| v.as_array()));

    let trades = match trades {
        Some(t) if !t.is_empty() => t,
        _ => {
            println!("No trades found.");
            return Ok(());
        }
    };

    #[derive(Tabled)]
    struct TradeRow {
        #[tabled(rename = "Market")]
        market: String,
        #[tabled(rename = "Side")]
        side: String,
        #[tabled(rename = "Outcome")]
        outcome: String,
        #[tabled(rename = "Price")]
        price: String,
        #[tabled(rename = "Amount")]
        amount: String,
        #[tabled(rename = "Time")]
        time: String,
    }

    let rows: Vec<TradeRow> = trades
        .iter()
        .map(|t| {
            let market_title = t
                .get("market")
                .and_then(|m| m.get("title").or(m.get("slug")))
                .and_then(|v| v.as_str())
                .unwrap_or("?");

            let side = t
                .get("side")
                .or_else(|| t.get("type"))
                .or_else(|| t.get("strategy"))
                .and_then(|v| v.as_str())
                .unwrap_or("-");

            let outcome_idx = t.get("outcomeIndex").and_then(|v| v.as_u64());
            let outcome = match outcome_idx {
                Some(0) => "YES".to_string(),
                Some(1) => "NO".to_string(),
                _ => t
                    .get("outcome")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-")
                    .to_string(),
            };

            let price = t
                .get("price")
                .or_else(|| t.get("outcomeTokenPrice"))
                .and_then(|v| v.as_f64())
                .map(|p| format!("${:.4}", p))
                .unwrap_or_else(|| "-".to_string());

            let amount = t
                .get("amount")
                .or_else(|| t.get("outcomeTokenAmount"))
                .and_then(|v| v.as_str().or_else(|| v.as_f64().map(|_| "?")))
                .map(|a| {
                    // Try parsing as raw micro-USDC
                    match a.parse::<i64>() {
                        Ok(v) => raw_to_usdc(&v.to_string()),
                        Err(_) => a.to_string(),
                    }
                })
                .unwrap_or_else(|| "-".to_string());

            let time = t
                .get("timestamp")
                .or_else(|| t.get("blockTimestamp"))
                .or_else(|| t.get("createdAt"))
                .map(|v| {
                    if let Some(ts) = v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)) {
                        // Unix timestamp (seconds or millis)
                        let secs = if ts > 1_000_000_000_000 { ts / 1000 } else { ts };
                        chrono::DateTime::from_timestamp(secs as i64, 0)
                            .map(|dt| dt.format("%b %d, %H:%M UTC").to_string())
                            .unwrap_or_else(|| ts.to_string())
                    } else if let Some(s) = v.as_str() {
                        format_deadline_with_time(s)
                    } else {
                        "-".to_string()
                    }
                })
                .unwrap_or_else(|| "-".to_string());

            TradeRow {
                market: truncate_title(market_title, 40),
                side: side.to_uppercase(),
                outcome,
                price,
                amount,
                time,
            }
        })
        .collect();

    println!("{} trade(s):", rows.len());
    println!();
    crate::output::print_table(&rows);

    Ok(())
}

/// Formatted history table
pub fn print_history_table(data: &serde_json::Value) -> Result<()> {
    let entries = data.get("data").and_then(|v| v.as_array());

    let entries = match entries {
        Some(e) if !e.is_empty() => e,
        _ => {
            println!("No history entries.");
            return Ok(());
        }
    };

    #[derive(Tabled)]
    struct HistoryRow {
        #[tabled(rename = "Type")]
        strategy: String,
        #[tabled(rename = "Market")]
        market: String,
        #[tabled(rename = "Outcome")]
        outcome: String,
        #[tabled(rename = "Shares")]
        shares: String,
        #[tabled(rename = "Price")]
        price: String,
        #[tabled(rename = "USDC")]
        collateral: String,
        #[tabled(rename = "Time")]
        time: String,
    }

    let rows: Vec<HistoryRow> = entries
        .iter()
        .map(|e| {
            let strategy = e
                .get("strategy")
                .and_then(|v| v.as_str())
                .unwrap_or("?");

            let market_title = e
                .get("market")
                .and_then(|m| m.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("?");

            let outcome_idx = e.get("outcomeIndex").and_then(|v| v.as_u64());
            let outcome = match outcome_idx {
                Some(0) => "YES".to_string(),
                Some(1) => "NO".to_string(),
                _ => "-".to_string(),
            };

            let shares = e
                .get("outcomeTokenAmount")
                .and_then(|v| v.as_str())
                .map(|a| raw_to_shares(a))
                .unwrap_or_else(|| "-".to_string());

            let price = e
                .get("outcomeTokenPrice")
                .and_then(|v| v.as_f64())
                .map(|p| format!("${:.4}", p))
                .unwrap_or_else(|| "-".to_string());

            let collateral = e
                .get("collateralAmount")
                .and_then(|v| v.as_str())
                .map(|a| raw_to_usdc(a))
                .unwrap_or_else(|| "-".to_string());

            let time = e
                .get("blockTimestamp")
                .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
                .map(|ts| {
                    let secs = if ts > 1_000_000_000_000 { ts / 1000 } else { ts };
                    chrono::DateTime::from_timestamp(secs as i64, 0)
                        .map(|dt| dt.format("%b %d, %H:%M UTC").to_string())
                        .unwrap_or_else(|| ts.to_string())
                })
                .unwrap_or_else(|| "-".to_string());

            HistoryRow {
                strategy: strategy.to_string(),
                market: truncate_title(market_title, 40),
                outcome,
                shares,
                price,
                collateral,
                time,
            }
        })
        .collect();

    // Show total count if available
    if let Some(total) = data.get("totalCount").and_then(|v| v.as_u64()) {
        println!("{} entries (showing {}):", total, rows.len());
    } else {
        println!("{} entries:", rows.len());
    }
    println!();
    crate::output::print_table(&rows);

    Ok(())
}

/// Formatted points summary
pub fn print_points_summary(data: &serde_json::Value) -> Result<()> {
    // Points may be a single value, an object with breakdown, or nested
    if data.is_null() || (data.is_object() && data.as_object().map_or(true, |o| o.is_empty())) {
        println!("No points data.");
        return Ok(());
    }

    println!("Points Breakdown");
    println!("─────────────────────────");

    // Try to extract known fields
    if let Some(obj) = data.as_object() {
        // Check for total
        if let Some(total) = obj.get("total").or(obj.get("points")).or(obj.get("totalPoints")) {
            let total_str = match total {
                v if v.is_string() => v.as_str().unwrap_or("0").to_string(),
                v if v.is_f64() => format!("{:.2}", v.as_f64().unwrap_or(0.0)),
                v if v.is_i64() => v.as_i64().unwrap_or(0).to_string(),
                _ => format!("{}", total),
            };
            println!("Total:          {}", total_str);
        }

        // Print all other fields as key-value pairs
        let mut printed_total = false;
        for (key, value) in obj {
            if key == "total" || key == "points" || key == "totalPoints" {
                if !printed_total {
                    printed_total = true;
                }
                continue;
            }
            let val_str = match value {
                v if v.is_string() => v.as_str().unwrap_or("-").to_string(),
                v if v.is_f64() => format!("{:.2}", v.as_f64().unwrap_or(0.0)),
                v if v.is_i64() => v.as_i64().unwrap_or(0).to_string(),
                v if v.is_boolean() => v.as_bool().unwrap_or(false).to_string(),
                v if v.is_object() || v.is_array() => {
                    // For nested objects, show a summary
                    serde_json::to_string(v).unwrap_or_else(|_| "-".to_string())
                }
                _ => "-".to_string(),
            };
            // Convert camelCase to readable name
            let label = camel_to_title(key);
            println!("{:<16}{}", format!("{}:", label), val_str);
        }
    } else {
        // Single value (number or string)
        println!("Total:          {}", data);
    }

    Ok(())
}

/// Formatted allowance summary
pub fn print_allowance_summary(data: &serde_json::Value) -> Result<()> {
    if data.is_null() || (data.is_object() && data.as_object().map_or(true, |o| o.is_empty())) {
        println!("No allowance data.");
        return Ok(());
    }

    println!("Trading Allowance");
    println!("─────────────────────────");

    let trading_type = data
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("-");
    let has_min = data
        .get("hasMinimumAllowance")
        .and_then(|v| v.as_bool())
        .map(|b| if b { "✓ Yes" } else { "✗ No" })
        .unwrap_or("-");

    let allowance_raw = data
        .get("allowance")
        .and_then(|v| v.as_str())
        .unwrap_or("0");

    // Convert raw allowance to USDC (6 decimals)
    let allowance_display = match allowance_raw.parse::<u128>() {
        Ok(v) => {
            let whole = v / 1_000_000;
            let frac = v % 1_000_000;
            if whole > 1_000_000_000 {
                "Unlimited".to_string()
            } else {
                format!("{}.{:02} USDC", whole, frac / 10_000)
            }
        }
        Err(_) => allowance_raw.to_string(),
    };

    let spender = data
        .get("spender")
        .and_then(|v| v.as_str())
        .unwrap_or("-");
    let checked = data
        .get("checkedAddress")
        .and_then(|v| v.as_str())
        .unwrap_or("-");

    println!("Type:           {}", trading_type.to_uppercase());
    println!("Allowance:      {}", allowance_display);
    println!("Sufficient:     {}", has_min);
    println!("Spender:        {}", truncate_address(spender));
    println!("Your Address:   {}", truncate_address(checked));

    Ok(())
}

/// Generic fallback for unknown portfolio data shapes
pub fn print_portfolio_data(label: &str, data: &serde_json::Value) -> Result<()> {
    println!("--- {} ---", label);
    print_json(data)?;
    Ok(())
}

fn truncate_title(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn format_fill_price(raw: &str) -> String {
    match raw.parse::<i64>() {
        Ok(v) if v == 0 => "-".to_string(),
        Ok(v) => {
            let price = v as f64 / 1_000_000.0;
            format!("${:.4}", price)
        }
        Err(_) => raw.to_string(),
    }
}

fn truncate_address(addr: &str) -> String {
    if addr.len() > 12 && addr.starts_with("0x") {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
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

fn format_deadline_with_time(d: &str) -> String {
    if d == "-" || d.is_empty() {
        return "-".to_string();
    }
    // Parse ISO date "2026-04-01T02:59:00.000Z" → "Apr 01, 02:59 UTC"
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(d) {
        return dt.format("%b %d, %H:%M UTC").to_string();
    }
    // Try without millis
    if d.len() >= 19 {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&d[..19], "%Y-%m-%dT%H:%M:%S") {
            return dt.format("%b %d, %H:%M UTC").to_string();
        }
    }
    d.to_string()
}
