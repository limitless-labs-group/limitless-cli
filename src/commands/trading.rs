use anyhow::{Context, Result};
use clap::Subcommand;
use rust_decimal::Decimal;

use crate::auth::{resolve_api_key, resolve_private_key};
use crate::client::LimitlessClient;
use crate::output::trading::{print_locked_balance, print_user_orders_table};
use crate::output::{print_json, OutputFormat};
use crate::signing;
use crate::signing::order::{build_gtc_order, Outcome, Side};

#[derive(Subcommand)]
pub enum TradingCommand {
    /// Place a GTC limit order
    Create {
        /// Market slug
        #[arg(long)]
        slug: String,
        /// Side: buy or sell
        #[arg(long)]
        side: String,
        /// Outcome: yes or no
        #[arg(long)]
        outcome: String,
        /// Price (0.01 to 0.99)
        #[arg(long)]
        price: Decimal,
        /// Size in shares (raw units, e.g. 100 for 100 shares)
        #[arg(long)]
        size: Decimal,
        /// Fee rate in basis points (default: 0)
        #[arg(long, default_value = "0")]
        fee_rate_bps: u64,
        /// Nonce (default: 0)
        #[arg(long, default_value = "0")]
        nonce: u64,
    },
    /// List your orders for a market
    Orders {
        /// Market slug
        slug: String,
        /// Filter by status (e.g. LIVE, FILLED, CANCELLED)
        #[arg(short, long)]
        status: Option<Vec<String>>,
        /// Max results
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Show locked balance for a market
    LockedBalance {
        /// Market slug
        slug: String,
    },
    /// Cancel a specific order
    Cancel {
        /// Order ID
        order_id: String,
    },
    /// Cancel multiple orders
    CancelBatch {
        /// Order IDs (comma-separated)
        #[arg(value_delimiter = ',')]
        order_ids: Vec<String>,
    },
    /// Cancel all orders for a market
    CancelAll {
        /// Market slug
        slug: String,
    },
}

pub async fn execute(
    command: &TradingCommand,
    output: &OutputFormat,
    api_key_flag: Option<&str>,
    private_key_flag: Option<&str>,
) -> Result<()> {
    match command {
        TradingCommand::Create {
            slug,
            side,
            outcome,
            price,
            size,
            fee_rate_bps,
            nonce,
        } => {
            let api_key = resolve_api_key(api_key_flag)?;
            let pk_str = resolve_private_key(private_key_flag)?;
            let client = LimitlessClient::new(Some(&api_key))?;

            // Parse side and outcome
            let side: Side = side.parse()?;
            let outcome: Outcome = outcome.parse()?;

            // Fetch market to get venue and token IDs
            let market = client.get_market(slug).await?;
            let venue = market
                .venue
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Market has no venue"))?;

            // Get token ID based on outcome
            let tokens = market
                .tokens
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Market has no token IDs"))?;
            let token_id_str = match outcome {
                Outcome::Yes => tokens.yes.as_deref(),
                Outcome::No => tokens.no.as_deref(),
            };
            let token_id_str = token_id_str
                .ok_or_else(|| anyhow::anyhow!("Could not find token ID for outcome {:?}", outcome))?;
            let token_id = signing::parse_u256(token_id_str)?;

            // Create signer
            let pk = pk_str.strip_prefix("0x").unwrap_or(&pk_str);
            let bytes = hex::decode(pk).context("Invalid hex in private key")?;
            let signer =
                alloy::signers::local::PrivateKeySigner::from_slice(&bytes)
                    .context("Invalid private key")?;
            let maker = signer.address();

            // Build order
            let order = build_gtc_order(
                maker,
                token_id,
                side,
                *price,
                *size,
                *fee_rate_bps,
                *nonce,
            )?;

            // Sign order
            let venue_exchange = signing::parse_address(&venue.exchange)?;
            let signature = signing::sign_order(&signer, &order, venue_exchange).await?;

            // Build payload
            let order_payload = serde_json::json!({
                "salt": signing::u256_to_string(&order.salt),
                "maker": signing::address_to_hex(&order.maker),
                "signer": signing::address_to_hex(&order.signer),
                "taker": signing::address_to_hex(&order.taker),
                "tokenId": signing::u256_to_string(&order.tokenId),
                "makerAmount": signing::u256_to_string(&order.makerAmount),
                "takerAmount": signing::u256_to_string(&order.takerAmount),
                "expiration": signing::u256_to_string(&order.expiration),
                "nonce": signing::u256_to_string(&order.nonce),
                "feeRateBps": signing::u256_to_string(&order.feeRateBps),
                "side": order.side.to_string(),
                "signatureType": order.signatureType.to_string(),
                "signature": signing::signature_hex(&signature),
            });

            let create_payload = serde_json::json!({
                "order": order_payload,
                "orderType": "GTC",
                "marketSlug": slug,
            });

            // Submit order
            let resp: serde_json::Value = client.post("/orders", &create_payload).await?;

            match output {
                OutputFormat::Json => print_json(&resp)?,
                OutputFormat::Table => {
                    println!("Order submitted successfully!");
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                }
            }
        }
        TradingCommand::Orders {
            slug,
            status,
            limit,
        } => {
            let api_key = resolve_api_key(api_key_flag)?;
            let client = LimitlessClient::new(Some(&api_key))?;
            let statuses: Option<Vec<&str>> = status
                .as_ref()
                .map(|s| s.iter().map(|x| x.as_str()).collect());
            let resp = client
                .get_user_orders(slug, statuses.as_deref(), *limit)
                .await?;
            match output {
                OutputFormat::Json => print_json(&resp)?,
                OutputFormat::Table => print_user_orders_table(&resp.orders),
            }
        }
        TradingCommand::LockedBalance { slug } => {
            let api_key = resolve_api_key(api_key_flag)?;
            let client = LimitlessClient::new(Some(&api_key))?;
            let balance = client.get_locked_balance(slug).await?;
            match output {
                OutputFormat::Json => print_json(&balance)?,
                OutputFormat::Table => print_locked_balance(&balance),
            }
        }
        TradingCommand::Cancel { order_id } => {
            let api_key = resolve_api_key(api_key_flag)?;
            let client = LimitlessClient::new(Some(&api_key))?;
            let resp = client.cancel_order(order_id).await?;
            match output {
                OutputFormat::Json => print_json(&resp)?,
                OutputFormat::Table => println!("Order {} cancelled.", order_id),
            }
        }
        TradingCommand::CancelBatch { order_ids } => {
            let api_key = resolve_api_key(api_key_flag)?;
            let client = LimitlessClient::new(Some(&api_key))?;
            let resp = client.cancel_batch(order_ids).await?;
            match output {
                OutputFormat::Json => print_json(&resp)?,
                OutputFormat::Table => println!("Cancelled {} orders.", order_ids.len()),
            }
        }
        TradingCommand::CancelAll { slug } => {
            let api_key = resolve_api_key(api_key_flag)?;
            let client = LimitlessClient::new(Some(&api_key))?;
            let resp = client.cancel_all(slug).await?;
            match output {
                OutputFormat::Json => print_json(&resp)?,
                OutputFormat::Table => println!("All orders cancelled for market: {}", slug),
            }
        }
    }

    Ok(())
}
