use anyhow::Result;
use clap::Subcommand;

use crate::client::LimitlessClient;
use crate::output::profiles::{print_public_pnl, print_public_positions, print_public_volume};
use crate::output::{print_json, OutputFormat};

#[derive(Subcommand)]
pub enum ProfilesCommand {
    /// Show positions for a public address
    Positions {
        /// Wallet address
        address: String,
    },
    /// Show traded volume for a public address
    Volume {
        /// Wallet address
        address: String,
    },
    /// Show PnL chart for a public address
    Pnl {
        /// Wallet address
        address: String,
        /// Timeframe (e.g. 1d, 7d, 30d, all)
        #[arg(short, long)]
        timeframe: Option<String>,
    },
}

pub async fn execute(
    command: &ProfilesCommand,
    output: &OutputFormat,
    api_key: Option<&str>,
) -> Result<()> {
    let client = LimitlessClient::new(api_key)?;

    match command {
        ProfilesCommand::Positions { address } => {
            let data = client.get_public_positions(address).await?;
            match output {
                OutputFormat::Json => print_json(&data)?,
                OutputFormat::Table => print_public_positions(&data)?,
            }
        }
        ProfilesCommand::Volume { address } => {
            let data = client.get_public_traded_volume(address).await?;
            match output {
                OutputFormat::Json => print_json(&data)?,
                OutputFormat::Table => print_public_volume(&data)?,
            }
        }
        ProfilesCommand::Pnl {
            address,
            timeframe,
        } => {
            let data = client
                .get_public_pnl_chart(address, timeframe.as_deref())
                .await?;
            match output {
                OutputFormat::Json => print_json(&data)?,
                OutputFormat::Table => print_public_pnl(&data)?,
            }
        }
    }

    Ok(())
}
