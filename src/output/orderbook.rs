use colored::Colorize;
use rust_decimal::Decimal;
use tabled::Tabled;

use crate::client::trading::{HistoricalPriceSeries, MarketEventsResponse, OrderbookResponse};
use crate::output::print_table;

/// USDC has 6 decimals — sizes from the API are in raw units
const COLLATERAL_SCALE: Decimal = Decimal::from_parts(1_000_000, 0, 0, false, 0);

fn format_size(raw: Decimal) -> String {
    let scaled = raw / COLLATERAL_SCALE;
    format!("{:.2}", scaled)
}

fn format_raw_usdc(raw: &str) -> String {
    match raw.parse::<i64>() {
        Ok(v) => {
            let whole = v / 1_000_000;
            let frac = (v % 1_000_000).unsigned_abs();
            format!("{}.{:02}", whole, frac / 10_000)
        }
        Err(_) => raw.to_string(),
    }
}

fn truncate_addr(addr: &str) -> String {
    if addr.len() > 12 && addr.starts_with("0x") {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

fn format_event_time(iso: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        return dt.format("%b %d, %H:%M UTC").to_string();
    }
    if iso.len() >= 19 {
        if let Ok(dt) =
            chrono::NaiveDateTime::parse_from_str(&iso[..19], "%Y-%m-%dT%H:%M:%S")
        {
            return dt.format("%b %d, %H:%M UTC").to_string();
        }
    }
    iso.to_string()
}

#[derive(Tabled)]
struct OrderRow {
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Size")]
    size: String,
}

pub fn print_orderbook(book: &OrderbookResponse) {
    if let Some(mid) = book.adjusted_midpoint {
        println!("{} ${:.4}", "Midpoint:".cyan(), mid);
    }
    if let Some(last) = book.last_trade_price {
        println!("{} ${:.4}", "Last Trade:".cyan(), last);
    }

    let spread = if !book.asks.is_empty() && !book.bids.is_empty() {
        Some(book.asks[0].price - book.bids[0].price)
    } else {
        None
    };
    if let Some(s) = spread {
        println!("{} {:.4}", "Spread:".cyan(), s);
    }
    println!();

    println!("{}", "═══ BIDS (Buy) ═══".green().bold());
    let bid_rows: Vec<OrderRow> = book
        .bids
        .iter()
        .map(|l| OrderRow {
            price: format!("${:.4}", l.price).green().to_string(),
            size: format_size(l.size),
        })
        .collect();
    print_table(&bid_rows);

    println!();
    println!("{}", "═══ ASKS (Sell) ═══".red().bold());
    let ask_rows: Vec<OrderRow> = book
        .asks
        .iter()
        .map(|l| OrderRow {
            price: format!("${:.4}", l.price).red().to_string(),
            size: format_size(l.size),
        })
        .collect();
    print_table(&ask_rows);
}

pub fn print_price(book: &OrderbookResponse) {
    let best_bid = book.bids.first();
    let best_ask = book.asks.first();

    match (best_bid, best_ask) {
        (Some(bid), Some(ask)) => {
            let spread = ask.price - bid.price;
            println!(
                "{} {}  ({} shares)   {} {}  ({} shares)   {} {:.4}",
                "Bid:".green(),
                format!("${:.4}", bid.price).green(),
                format_size(bid.size),
                "Ask:".red(),
                format!("${:.4}", ask.price).red(),
                format_size(ask.size),
                "Spread:".cyan(),
                spread,
            );
        }
        (Some(bid), None) => {
            println!(
                "{} {}  ({} shares)   {} {}",
                "Bid:".green(),
                format!("${:.4}", bid.price).green(),
                format_size(bid.size),
                "Ask:".red(),
                "-".dimmed(),
            );
        }
        (None, Some(ask)) => {
            println!(
                "{} {}   {} {}  ({} shares)",
                "Bid:".green(),
                "-".dimmed(),
                "Ask:".red(),
                format!("${:.4}", ask.price).red(),
                format_size(ask.size),
            );
        }
        (None, None) => {
            println!("{}", "No bids or asks available.".dimmed());
        }
    }
}

pub fn print_midpoint(book: &OrderbookResponse) {
    if let Some(mid) = book.adjusted_midpoint {
        println!("{} ${:.4}", "Midpoint:".cyan(), mid);
    } else if !book.asks.is_empty() && !book.bids.is_empty() {
        let mid = (book.asks[0].price + book.bids[0].price) / 2.0;
        println!("{} ${:.4}", "Midpoint:".cyan(), mid);
    } else {
        println!("{}", "Cannot calculate midpoint.".dimmed());
    }
}

pub fn print_spread(book: &OrderbookResponse) {
    if !book.asks.is_empty() && !book.bids.is_empty() {
        let spread = book.asks[0].price - book.bids[0].price;
        println!("{} {:.4}", "Spread:".cyan(), spread);
    } else {
        println!("{}", "Cannot calculate spread.".dimmed());
    }
}

/// Format historical prices as a table
pub fn print_historical_prices(series: &[HistoricalPriceSeries]) {
    if series.is_empty() {
        println!("No historical price data.");
        return;
    }

    #[derive(Tabled)]
    struct PriceRow {
        #[tabled(rename = "Token")]
        token: String,
        #[tabled(rename = "Price")]
        price: String,
        #[tabled(rename = "Time")]
        time: String,
    }

    let mut rows: Vec<PriceRow> = Vec::new();

    for s in series {
        let token_name = s.title.as_deref().unwrap_or("?");
        for p in &s.prices {
            let price_str = p
                .price
                .map(|v| format!("${:.4}", v))
                .unwrap_or_else(|| "-".to_string());
            let time_str = p
                .timestamp
                .as_deref()
                .map(format_event_time)
                .unwrap_or_else(|| "-".to_string());
            rows.push(PriceRow {
                token: token_name.to_string(),
                price: price_str,
                time: time_str,
            });
        }
    }

    if rows.is_empty() {
        println!("{}", "No price data points.".dimmed());
        return;
    }

    println!("{} data point(s):", rows.len().to_string().bold());
    println!();
    print_table(&rows);
}

/// Format market feed events as a table
pub fn print_events_table(resp: &MarketEventsResponse) {
    if resp.events.is_empty() {
        println!("{}", "No events.".dimmed());
        return;
    }

    #[derive(Tabled)]
    struct EventRow {
        #[tabled(rename = "Side")]
        side: String,
        #[tabled(rename = "Price")]
        price: String,
        #[tabled(rename = "Size")]
        size: String,
        #[tabled(rename = "Cost")]
        cost: String,
        #[tabled(rename = "Trader")]
        trader: String,
        #[tabled(rename = "Time")]
        time: String,
    }

    let rows: Vec<EventRow> = resp
        .events
        .iter()
        .map(|e| {
            let side = match e.side {
                Some(0) => "BUY".green().to_string(),
                Some(1) => "SELL".red().to_string(),
                _ => "-".to_string(),
            };

            let price = e
                .price
                .map(|p| format!("${:.4}", p))
                .unwrap_or_else(|| "-".to_string());

            let size = e
                .matched_size
                .as_deref()
                .map(format_raw_usdc)
                .unwrap_or_else(|| "-".to_string());

            let cost = e
                .taker_amount
                .as_deref()
                .map(format_raw_usdc)
                .unwrap_or_else(|| "-".to_string());

            let trader = e
                .profile
                .as_ref()
                .and_then(|p| {
                    p.username
                        .as_deref()
                        .or(p.display_name.as_deref())
                        .or(p.account.as_deref())
                })
                .map(truncate_addr)
                .unwrap_or_else(|| "-".to_string());

            let time = e
                .created_at
                .as_deref()
                .map(format_event_time)
                .unwrap_or_else(|| "-".to_string());

            EventRow {
                side,
                price,
                size,
                cost,
                trader,
                time,
            }
        })
        .collect();

    // Pagination info
    if let (Some(page), Some(total_pages), Some(total_rows)) =
        (resp.page, resp.total_pages, resp.total_rows)
    {
        println!(
            "{} events (page {} of {}, {} total):",
            rows.len(),
            page,
            total_pages,
            total_rows
        );
    } else {
        println!("{} events:", rows.len());
    }
    println!();
    print_table(&rows);
}
