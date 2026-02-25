use anyhow::Result;
use clap::Subcommand;

use crate::auth::resolve_api_key;
use crate::client::LimitlessClient;
use crate::output::portfolio::{
    print_allowance_summary, print_history_table, print_pnl_summary, print_points_summary,
    print_positions_table, print_trades_table,
};
use crate::output::{print_json, OutputFormat};

#[derive(Subcommand)]
pub enum PortfolioCommand {
    /// Show open positions
    Positions {
        /// Filter by status: funded, resolved, all (default: all)
        #[arg(short, long, default_value = "all")]
        status: String,
    },
    /// Show trade history
    Trades,
    /// Show PnL chart data
    Pnl {
        /// Timeframe (e.g. 1d, 7d, 30d, all)
        #[arg(short, long)]
        timeframe: Option<String>,
    },
    /// Show portfolio history
    History {
        /// Page number
        #[arg(short, long)]
        page: Option<u32>,
        /// Results per page
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Show accumulated points
    Points,
    /// Check trading allowance/approval status
    Allowance {
        /// Trading type: clob or negrisk (default: clob)
        #[arg(short = 't', long, default_value = "clob")]
        trading_type: Option<String>,
        /// Spender contract address
        #[arg(short, long)]
        spender: Option<String>,
    },
}

pub async fn execute(
    command: &PortfolioCommand,
    output: &OutputFormat,
    api_key_flag: Option<&str>,
) -> Result<()> {
    let api_key = resolve_api_key(api_key_flag)?;
    let client = LimitlessClient::new(Some(&api_key))?;

    match command {
        PortfolioCommand::Positions { status } => {
            let data = client.get_positions().await?;
            match output {
                OutputFormat::Json => print_json(&data)?,
                OutputFormat::Table => print_positions_table(&data, status)?,
            }
        }
        PortfolioCommand::Trades => {
            let data = client.get_trades().await?;
            match output {
                OutputFormat::Json => print_json(&data)?,
                OutputFormat::Table => print_trades_table(&data)?,
            }
        }
        PortfolioCommand::Pnl { timeframe } => {
            let data = client.get_pnl_chart(timeframe.as_deref()).await?;
            match output {
                OutputFormat::Json => print_json(&data)?,
                OutputFormat::Table => print_pnl_summary(&data)?,
            }
        }
        PortfolioCommand::History { page, limit } => {
            let data = client.get_history(*page, *limit).await?;
            match output {
                OutputFormat::Json => print_json(&data)?,
                OutputFormat::Table => print_history_table(&data)?,
            }
        }
        PortfolioCommand::Points => {
            let data = client.get_points().await?;
            match output {
                OutputFormat::Json => print_json(&data)?,
                OutputFormat::Table => print_points_summary(&data)?,
            }
        }
        PortfolioCommand::Allowance {
            trading_type,
            spender,
        } => {
            let data = client
                .get_trading_allowance(trading_type.as_deref(), spender.as_deref())
                .await?;
            match output {
                OutputFormat::Json => print_json(&data)?,
                OutputFormat::Table => print_allowance_summary(&data)?,
            }
        }
    }

    Ok(())
}
