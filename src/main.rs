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
use colored::Colorize;

use commands::Commands;
use output::OutputFormat;

fn print_banner() {
    let g = |s: &str| s.truecolor(197, 232, 77);

    eprintln!();
    eprintln!("{}", g(r"      \  |"));
    eprintln!("{}", g(r"       \ | /"));
    eprintln!("{}", g(r"        \|/"));
    eprintln!("{}  {}",
        g("  \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}*\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}"),
        "L I M I T L E S S".truecolor(197, 232, 77).bold(),
    );
    eprintln!("{}", g(r"        / \"));
    eprintln!("{}", g(r"       /   \"));
    eprintln!("{}", g(r"      /"));
    eprintln!();
    eprintln!("    {}  {}",
        "Prediction Markets on Base".dimmed(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed(),
    );
    eprintln!("    {}", "https://limitless.exchange".dimmed().underline());
    eprintln!();
}

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
    pub command: Option<Commands>,
}

#[tokio::main]
async fn main() {
    // Show banner before --help / -h
    let args: Vec<String> = std::env::args().collect();
    let wants_help = args.len() == 2 && (args[1] == "--help" || args[1] == "-h");
    if args.len() == 1 || wants_help {
        print_banner();
    }
    if args.len() == 1 {
        Cli::parse_from(["limitless", "--help"]);
        return;
    }

    let cli = Cli::parse();

    if let Err(e) = execute(cli).await {
        eprintln!("{} {:#}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

pub async fn execute(cli: Cli) -> Result<()> {
    let command = cli.command.expect("command is required");
    match &command {
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
        Commands::Setup => commands::setup::execute().await,
    }
}
