use alloy::primitives::U256;
use anyhow::{Context, Result};
use clap::Subcommand;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::auth::{resolve_api_key, resolve_private_key};
use crate::client::LimitlessClient;
use crate::output::trading::{print_locked_balance, print_order_created, print_user_orders_table};
use crate::output::{print_json, OutputFormat};
use crate::signing;
use crate::signing::order::{build_fok_order, build_gtc_order, Outcome, Side};

#[derive(Subcommand)]
pub enum TradingCommand {
    /// Place an order (GTC limit or FOK market)
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
        /// Price (0.01 to 0.99) — required for GTC, ignored for FOK
        #[arg(long)]
        price: Option<Decimal>,
        /// For GTC: number of shares. For FOK buy: USDC to spend. For FOK sell: shares to sell.
        #[arg(long)]
        size: Decimal,
        /// Order type: GTC (default) or FOK
        #[arg(long, default_value = "GTC")]
        order_type: String,
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
            order_type,
            nonce,
        } => {
            let api_key = resolve_api_key(api_key_flag)?;
            let pk_str = resolve_private_key(private_key_flag)?;
            let client = LimitlessClient::new(Some(&api_key))?;

            // Validate order type
            let order_type_upper = order_type.to_uppercase();
            let is_fok = order_type_upper == "FOK";
            if order_type_upper != "GTC" && !is_fok {
                anyhow::bail!("Invalid order type: {}. Use 'GTC' or 'FOK'.", order_type);
            }

            // GTC requires price
            if !is_fok && price.is_none() {
                anyhow::bail!("--price is required for GTC orders. Use --order-type FOK for market orders without a price.");
            }

            // Parse side and outcome
            let side: Side = side.parse()?;
            let outcome: Outcome = outcome.parse()?;

            // Create signer and derive address
            let pk = pk_str.strip_prefix("0x").unwrap_or(&pk_str);
            let bytes = hex::decode(pk).context("Invalid hex in private key")?;
            let signer = alloy::signers::local::PrivateKeySigner::from_slice(&bytes)
                .context("Invalid private key")?;
            let maker = signer.address();

            // Fetch profile to get ownerId and feeRateBps
            let profile = client
                .get_profile(&signing::address_to_hex(&maker))
                .await
                .context("Failed to fetch user profile. Make sure your API key and wallet address are linked.")?;
            let owner_id = profile.id;
            let fee_rate_bps = profile
                .rank
                .as_ref()
                .and_then(|r| r.fee_rate_bps)
                .unwrap_or(300);

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

            // Build order — GTC and FOK have different amount semantics
            let (order, order_price) = if is_fok {
                // FOK: makerAmount = size * 1e6, takerAmount = 1
                // For BUY: size = USDC to spend
                // For SELL: size = shares to sell
                let scale = Decimal::from(1_000_000u64);
                let maker_amount_scaled = (*size * scale)
                    .to_u128()
                    .ok_or_else(|| anyhow::anyhow!("makerAmount overflow"))?;

                let order = build_fok_order(
                    maker,
                    token_id,
                    side,
                    U256::from(maker_amount_scaled),
                    U256::from(1u64), // takerAmount is always 1 for FOK
                    fee_rate_bps,
                    *nonce,
                );
                (order, None) // No price for FOK
            } else {
                // GTC: standard price * size computation
                let p = price.unwrap(); // safe: validated above
                let order = build_gtc_order(
                    maker,
                    token_id,
                    side,
                    p,
                    *size,
                    fee_rate_bps,
                    *nonce,
                )?;
                let price_f64 = p.to_string().parse::<f64>().unwrap_or(0.0);
                (order, Some(price_f64))
            };

            // Sign order
            let venue_exchange = signing::parse_address(&venue.exchange)?;
            let signature = signing::sign_order(&signer, &order, venue_exchange).await?;

            // Build order payload with NUMERIC fields
            let mut order_payload = serde_json::json!({
                "salt": signing::u256_to_u64(&order.salt),
                "maker": signing::address_to_hex(&order.maker),
                "signer": signing::address_to_hex(&order.signer),
                "taker": signing::address_to_hex(&order.taker),
                "tokenId": token_id_str,
                "makerAmount": signing::u256_to_u64(&order.makerAmount),
                "takerAmount": signing::u256_to_u64(&order.takerAmount),
                "expiration": signing::u256_to_u64(&order.expiration).to_string(),
                "nonce": signing::u256_to_u64(&order.nonce),
                "feeRateBps": signing::u256_to_u64(&order.feeRateBps),
                "side": order.side as u64,
                "signatureType": order.signatureType as u64,
                "signature": signing::signature_hex(&signature),
            });

            // GTC orders include price; FOK orders do not
            if let Some(p) = order_price {
                order_payload
                    .as_object_mut()
                    .unwrap()
                    .insert("price".to_string(), serde_json::json!(p));
            }

            let create_payload = serde_json::json!({
                "order": order_payload,
                "orderType": order_type_upper,
                "marketSlug": slug,
                "ownerId": owner_id,
            });

            // Submit order
            let resp: serde_json::Value = client.post("/orders", &create_payload).await?;

            match output {
                OutputFormat::Json => print_json(&resp)?,
                OutputFormat::Table => print_order_created(&resp),
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
