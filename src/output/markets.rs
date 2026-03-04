use colored::Colorize;
use rust_decimal::Decimal;
use tabled::Tabled;

use crate::client::markets::{CategoryWithCount, Market, MarketSlug};
use crate::output::{format_decimal, print_table, truncate};

#[derive(Tabled)]
struct MarketRow {
    #[tabled(rename = "Question")]
    title: String,
    #[tabled(rename = "Price (Yes)")]
    yes_price: String,
    #[tabled(rename = "Volume")]
    volume: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Deadline")]
    deadline: String,
}

fn format_price(price: Option<f64>) -> String {
    match price {
        Some(p) => format!("${:.2}", p),
        None => "—".to_string(),
    }
}

fn format_price_cents(price: Option<f64>, is_amm: bool) -> String {
    match price {
        Some(p) => {
            let normalized = if is_amm && p > 1.0 { p / 100.0 } else { p };
            let cents = normalized * 100.0;
            format!("{:.2}¢", cents)
        }
        None => "—".to_string(),
    }
}

fn format_volume_usd(market: &Market) -> String {
    match market.display_volume() {
        Some(v) => format!("${}", format_decimal(v)),
        None => "—".to_string(),
    }
}

fn format_status(market: &Market) -> String {
    match market.status.as_deref() {
        Some("FUNDED") => "Active".to_string(),
        Some("RESOLVED") => "Resolved".to_string(),
        Some(s) => s.to_string(),
        None => "—".to_string(),
    }
}

pub fn print_markets_table(markets: &[Market]) {
    let rows: Vec<MarketRow> = markets
        .iter()
        .map(|m| {
            let is_amm = m.trade_type.as_deref() == Some("amm");
            MarketRow {
                title: truncate(&m.title, 60),
                yes_price: format_price_cents(m.yes_price(), is_amm),
                volume: format_volume_usd(m),
                status: format_status(m),
                deadline: m
                    .display_deadline()
                    .unwrap_or_else(|| "—".to_string()),
            }
        })
        .collect();
    print_table(&rows);
}

pub fn print_market_detail(market: &Market) {
    let mut rows: Vec<(&str, String)> = vec![
        ("Slug", market.slug.clone()),
        ("Title", market.title.clone()),
    ];

    if let Some(desc) = &market.description {
        // Strip HTML tags for clean display
        let clean = desc
            .replace("<p>", "")
            .replace("</p>", "\n")
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<br />", "\n")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"");
        // Remove any remaining HTML tags
        let mut result = String::new();
        let mut in_tag = false;
        for ch in clean.chars() {
            if ch == '<' { in_tag = true; continue; }
            if ch == '>' { in_tag = false; continue; }
            if !in_tag { result.push(ch); }
        }
        let trimmed = result.trim().to_string();
        rows.push(("Description", trimmed));
    }
    if let Some(tt) = &market.trade_type {
        rows.push(("Trade Type", tt.clone()));
    }
    if let Some(mt) = &market.market_type {
        rows.push(("Market Type", mt.clone()));
    }
    if let Some(s) = &market.status {
        rows.push(("Status", s.clone()));
    }
    if let Some(v) = market.display_volume() {
        rows.push(("Volume", format!("{} USDC", format_decimal(v))));
    }
    if let Some(liq) = &market.liquidity {
        if let Ok(l) = liq.parse::<Decimal>() {
            rows.push(("Liquidity", format_decimal(l)));
        }
    }
    rows.push(("YES Price", format_price(market.yes_price())));
    rows.push(("NO Price", format_price(market.no_price())));
    if let Some(d) = market.display_deadline() {
        rows.push(("Deadline", d));
    }
    if let Some(rs) = &market.resolution_source {
        rows.push(("Resolution", rs.clone()));
    }
    if let Some(at) = &market.automation_type {
        rows.push(("Automation", at.clone()));
    }
    if let Some(venue) = &market.venue {
        rows.push(("Exchange", venue.exchange.clone()));
        if let Some(adapter) = &venue.adapter {
            rows.push(("Adapter", adapter.clone()));
        }
    }
    if let Some(tokens) = &market.tokens {
        if let Some(yes) = &tokens.yes {
            rows.push(("YES Token ID", yes.clone()));
        }
        if let Some(no) = &tokens.no {
            rows.push(("NO Token ID", no.clone()));
        }
    }
    if let Some(ct) = &market.collateral_token {
        rows.push(("Collateral", format!("{} ({})", ct.symbol, ct.address)));
    }
    if let Some(cats) = &market.categories {
        if !cats.is_empty() {
            rows.push(("Categories", cats.join(", ")));
        }
    }
    if let Some(tags) = &market.tags {
        if !tags.is_empty() {
            rows.push(("Tags", tags.join(", ")));
        }
    }
    if let Some(ca) = &market.created_at {
        rows.push(("Created", ca.clone()));
    }

    // NegRisk group sub-markets
    if let Some(sub_markets) = &market.markets {
        rows.push(("Sub-Markets", sub_markets.len().to_string()));
        for (i, sub) in sub_markets.iter().enumerate() {
            rows.push(("", format!("  [{}] {}", i, sub.title)));
        }
    }

    crate::output::print_detail_table(rows);
}

#[derive(Tabled)]
struct SlugRow {
    #[tabled(rename = "Slug")]
    slug: String,
    #[tabled(rename = "Ticker")]
    ticker: String,
    #[tabled(rename = "Strike")]
    strike: String,
    #[tabled(rename = "Deadline")]
    deadline: String,
}

pub fn print_slugs_table(slugs: &[MarketSlug]) {
    let rows: Vec<SlugRow> = slugs
        .iter()
        .map(|s| SlugRow {
            slug: s.slug.clone(),
            ticker: s.ticker.clone().unwrap_or_else(|| "—".to_string()),
            strike: s.strike_price.clone().unwrap_or_else(|| "—".to_string()),
            deadline: s.deadline.clone().unwrap_or_else(|| "—".to_string()),
        })
        .collect();
    print_table(&rows);
}

#[derive(Tabled)]
struct CategoryRow {
    #[tabled(rename = "ID")]
    id: u32,
    #[tabled(rename = "Category")]
    name: String,
    #[tabled(rename = "Markets")]
    count: u32,
}

pub fn print_categories_table(categories: &[CategoryWithCount], total: Option<u32>) {
    let rows: Vec<CategoryRow> = categories
        .iter()
        .map(|c| CategoryRow {
            id: c.id,
            name: c.name.clone(),
            count: c.count,
        })
        .collect();

    crate::output::print_table(&rows);

    if let Some(total) = total {
        println!("{} {}", "Total markets:".cyan(), total.to_string().bold());
    }
}
