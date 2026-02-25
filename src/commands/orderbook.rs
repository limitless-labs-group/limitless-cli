use anyhow::Result;
use clap::Subcommand;

use crate::client::LimitlessClient;
use crate::output::orderbook::{
    print_events_table, print_historical_prices, print_midpoint, print_orderbook, print_price,
    print_spread,
};
use crate::output::{print_json, OutputFormat};

#[derive(Subcommand)]
pub enum OrderbookCommand {
    /// Show full orderbook for a market
    Book {
        /// Market slug
        slug: String,
    },
    /// Show best bid and ask prices with sizes
    Price {
        /// Market slug
        slug: String,
    },
    /// Show midpoint price
    Midpoint {
        /// Market slug
        slug: String,
    },
    /// Show bid-ask spread
    Spread {
        /// Market slug
        slug: String,
    },
    /// Show last trade price
    LastTrade {
        /// Market slug
        slug: String,
    },
    /// Show historical prices
    History {
        /// Market slug
        slug: String,
        /// Start timestamp or date
        #[arg(long)]
        from: Option<String>,
        /// End timestamp or date
        #[arg(long)]
        to: Option<String>,
        /// Interval (e.g. 1m, 1h, 1d)
        #[arg(short, long)]
        interval: Option<String>,
    },
    /// Show market feed events
    Events {
        /// Market slug
        slug: String,
        /// Page number
        #[arg(short, long)]
        page: Option<u32>,
        /// Results per page
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Live orderbook monitor (TUI)
    Monitor {
        /// Market slug
        slug: String,
    },
}

pub async fn execute(
    command: &OrderbookCommand,
    output: &OutputFormat,
    api_key: Option<&str>,
) -> Result<()> {
    // Monitor has its own client and TUI — handle before creating shared client
    if let OrderbookCommand::Monitor { slug } = command {
        return crate::tui::app::run_monitor(slug, api_key).await;
    }

    let client = LimitlessClient::new(api_key)?;

    match command {
        OrderbookCommand::Book { slug } => {
            let book = client.get_orderbook(slug).await?;
            match output {
                OutputFormat::Json => print_json(&book)?,
                OutputFormat::Table => print_orderbook(&book),
            }
        }
        OrderbookCommand::Price { slug } => {
            let book = client.get_orderbook(slug).await?;
            match output {
                OutputFormat::Json => {
                    let best_bid = book.bids.first().map(|l| l.price);
                    let best_ask = book.asks.first().map(|l| l.price);
                    print_json(&serde_json::json!({
                        "bid": best_bid,
                        "ask": best_ask,
                    }))?;
                }
                OutputFormat::Table => print_price(&book),
            }
        }
        OrderbookCommand::Midpoint { slug } => {
            let book = client.get_orderbook(slug).await?;
            match output {
                OutputFormat::Json => {
                    print_json(&serde_json::json!({"midpoint": book.adjusted_midpoint}))?;
                }
                OutputFormat::Table => print_midpoint(&book),
            }
        }
        OrderbookCommand::Spread { slug } => {
            let book = client.get_orderbook(slug).await?;
            match output {
                OutputFormat::Json => {
                    let spread = if !book.asks.is_empty() && !book.bids.is_empty() {
                        Some(book.asks[0].price - book.bids[0].price)
                    } else {
                        None
                    };
                    print_json(&serde_json::json!({"spread": spread}))?;
                }
                OutputFormat::Table => print_spread(&book),
            }
        }
        OrderbookCommand::LastTrade { slug } => {
            let book = client.get_orderbook(slug).await?;
            match output {
                OutputFormat::Json => {
                    print_json(
                        &serde_json::json!({"last_trade_price": book.last_trade_price}),
                    )?;
                }
                OutputFormat::Table => {
                    if let Some(price) = book.last_trade_price {
                        println!("Last Trade: ${:.4}", price);
                    } else {
                        println!("No last trade price available.");
                    }
                }
            }
        }
        OrderbookCommand::History {
            slug,
            from,
            to,
            interval,
        } => {
            let prices = client
                .get_historical_prices(
                    slug,
                    from.as_deref(),
                    to.as_deref(),
                    interval.as_deref(),
                )
                .await?;
            match output {
                OutputFormat::Json => print_json(&prices)?,
                OutputFormat::Table => print_historical_prices(&prices),
            }
        }
        OrderbookCommand::Events { slug, page, limit } => {
            let resp = client.get_market_events(slug, *page, *limit).await?;
            match output {
                OutputFormat::Json => print_json(&resp)?,
                OutputFormat::Table => print_events_table(&resp),
            }
        }
        OrderbookCommand::Monitor { .. } => unreachable!("handled above"),
    }

    Ok(())
}
