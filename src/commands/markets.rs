use anyhow::Result;
use clap::{Subcommand, ValueEnum};

use crate::client::LimitlessClient;
use crate::output::markets::{
    print_categories_table, print_market_detail, print_markets_table, print_slugs_table,
};
use crate::output::{print_json, OutputFormat};

#[derive(Clone, Debug, ValueEnum)]
pub enum SortBy {
    EndingSoon,
    HighValue,
    LpRewards,
    Newest,
    Trending,
}

impl std::fmt::Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortBy::EndingSoon => write!(f, "ending_soon"),
            SortBy::HighValue => write!(f, "high_value"),
            SortBy::LpRewards => write!(f, "lp_rewards"),
            SortBy::Newest => write!(f, "newest"),
            SortBy::Trending => write!(f, "trending"),
        }
    }
}

#[derive(Subcommand)]
pub enum MarketsCommand {
    /// List active markets
    List {
        /// Page number (starts at 0)
        #[arg(short, long)]
        page: Option<u32>,
        /// Results per page (max 25)
        #[arg(short, long, default_value = "20")]
        limit: Option<u32>,
        /// Sort by field
        #[arg(short, long, value_enum)]
        sort_by: Option<SortBy>,
        /// Filter by trade type (clob, amm, group)
        #[arg(short, long)]
        trade_type: Option<String>,
        /// Filter by category ID (use `markets categories` to see IDs)
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Get details for a specific market
    Get {
        /// Market slug (e.g. btc-100k-weekly)
        slug: String,
    },
    /// Search markets by query
    Search {
        /// Search query
        query: String,
        /// Max results
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// List all active market slugs
    Slugs,
    /// List market categories with counts
    Categories,
}

pub async fn execute(
    command: &MarketsCommand,
    output: &OutputFormat,
    api_key: Option<&str>,
) -> Result<()> {
    let client = LimitlessClient::new(api_key)?;

    match command {
        MarketsCommand::List {
            page,
            limit,
            sort_by,
            trade_type,
            category,
        } => {
            let sort_str = sort_by.as_ref().map(|s| s.to_string());
            let resp = client
                .get_active_markets(
                    *page,
                    *limit,
                    sort_str.as_deref(),
                    trade_type.as_deref(),
                    category.as_deref(),
                )
                .await?;

            match output {
                OutputFormat::Json => print_json(&resp)?,
                OutputFormat::Table => {
                    print_markets_table(&resp.data);
                    // Pagination info
                    let limit_val = limit.unwrap_or(20);
                    let current_page = page.unwrap_or(0);
                    let showing = resp.data.len();
                    if let Some(total) = resp.total_markets_count {
                        let total_pages = (total + limit_val - 1) / limit_val;
                        println!(
                            "\nPage {} of {} ({} markets total, showing {})",
                            current_page + 1,
                            total_pages,
                            total,
                            showing
                        );
                    } else {
                        println!("\nShowing {} markets (page {})", showing, current_page + 1);
                    }
                    if resp.next_page.is_some() {
                        println!("Next: --page {}", current_page + 1);
                    }
                }
            }
        }
        MarketsCommand::Get { slug } => {
            let market = client.get_market(slug).await?;
            match output {
                OutputFormat::Json => print_json(&market)?,
                OutputFormat::Table => print_market_detail(&market),
            }
        }
        MarketsCommand::Search { query, limit } => {
            let resp = client.search_markets(query, *limit).await?;
            match output {
                OutputFormat::Json => print_json(&resp)?,
                OutputFormat::Table => {
                    print_markets_table(&resp.markets);
                    if let Some(total) = resp.total_markets_count {
                        println!("Total results: {}", total);
                    }
                }
            }
        }
        MarketsCommand::Slugs => {
            let slugs = client.get_active_slugs().await?;
            match output {
                OutputFormat::Json => print_json(&slugs)?,
                OutputFormat::Table => print_slugs_table(&slugs),
            }
        }
        MarketsCommand::Categories => {
            let (categories, total) = client.get_categories_with_counts().await?;
            match output {
                OutputFormat::Json => print_json(&categories)?,
                OutputFormat::Table => print_categories_table(&categories, total),
            }
        }
    }

    Ok(())
}
