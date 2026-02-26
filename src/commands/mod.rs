pub mod approve;
pub mod markets;
pub mod orderbook;
pub mod portfolio;
pub mod profiles;
pub mod setup;
pub mod trading;
pub mod wallet;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Browse and search prediction markets
    Markets {
        #[command(subcommand)]
        command: markets::MarketsCommand,
    },
    /// View orderbook, prices, and spreads
    Orderbook {
        #[command(subcommand)]
        command: orderbook::OrderbookCommand,
    },
    /// Manage orders and trading
    Trading {
        #[command(subcommand)]
        command: trading::TradingCommand,
    },
    /// View portfolio positions, trades, and PnL
    Portfolio {
        #[command(subcommand)]
        command: portfolio::PortfolioCommand,
    },
    /// View public portfolio data for any address
    Profiles {
        #[command(subcommand)]
        command: profiles::ProfilesCommand,
    },
    /// Manage token approvals for trading
    Approve {
        #[command(subcommand)]
        command: approve::ApproveCommand,
    },
    /// Manage wallet keys
    Wallet {
        #[command(subcommand)]
        command: wallet::WalletCommand,
    },
    /// Interactive shell mode
    Shell,
    /// First-time setup wizard
    Setup,
}
