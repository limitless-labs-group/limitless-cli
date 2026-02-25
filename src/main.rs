mod auth;
mod client;
mod commands;
mod config;
mod constants;
mod output;
mod shell;
mod signing;
mod tui;

use anyhow::Result;
use clap::Parser;

use commands::Commands;
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "limitless", about = "CLI for Limitless Exchange", version)]
pub struct Cli {
    /// Output format
    #[arg(short, long, default_value = "table", global = true)]
    pub output: OutputFormat,

    /// API key (overrides env/config)
    #[arg(long, env = "LIMITLESS_API_KEY", global = true)]
    pub api_key: Option<String>,

    /// Private key for signing (overrides env/config)
    #[arg(long, env = "LIMITLESS_PRIVATE_KEY", global = true)]
    pub private_key: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = execute(cli).await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

pub async fn execute(cli: Cli) -> Result<()> {
    match &cli.command {
        Commands::Markets { command } => {
            commands::markets::execute(command, &cli.output, cli.api_key.as_deref()).await
        }
        Commands::Orderbook { command } => {
            commands::orderbook::execute(command, &cli.output, cli.api_key.as_deref()).await
        }
        Commands::Trading { command } => {
            commands::trading::execute(
                command,
                &cli.output,
                cli.api_key.as_deref(),
                cli.private_key.as_deref(),
            )
            .await
        }
        Commands::Portfolio { command } => {
            commands::portfolio::execute(command, &cli.output, cli.api_key.as_deref()).await
        }
        Commands::Profiles { command } => {
            commands::profiles::execute(command, &cli.output, cli.api_key.as_deref()).await
        }
        Commands::Approve { command } => {
            commands::approve::execute(
                command,
                &cli.output,
                cli.api_key.as_deref(),
                cli.private_key.as_deref(),
            )
            .await
        }
        Commands::Wallet { command } => {
            commands::wallet::execute(
                command,
                &cli.output,
                cli.api_key.as_deref(),
                cli.private_key.as_deref(),
            )
            .await
        }
        Commands::Shell => {
            shell::run_shell(cli.output.clone(), cli.api_key, cli.private_key).await
        }
        Commands::Setup => {
            println!("Setup wizard — configure your Limitless CLI");
            println!();
            println!("1. Get your API key from https://limitless.exchange (profile menu -> Api keys)");
            println!("2. Set it: export LIMITLESS_API_KEY=lmts_your_key_here");
            println!("3. Import or create a wallet: limitless wallet create / limitless wallet import <key>");
            println!("4. Fund your wallet with USDC on Base");
            println!("5. Approve tokens: limitless approve set --slug <market-slug>");
            Ok(())
        }
    }
}
