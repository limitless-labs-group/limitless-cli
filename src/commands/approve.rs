use alloy::primitives::{Address, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use anyhow::{Context, Result};
use clap::Subcommand;

use crate::auth::{resolve_api_key, resolve_private_key};
use crate::client::LimitlessClient;
use crate::constants::{CT_ADDRESS, DEFAULT_RPC_URL, USDC_ADDRESS};
use crate::output::{print_json, OutputFormat};

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
    }

    #[sol(rpc)]
    interface IERC1155 {
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address account, address operator) external view returns (bool);
    }
}

#[derive(Subcommand)]
pub enum ApproveCommand {
    /// Check approval status for a market
    Check {
        /// Market slug (fetches venue from market data)
        #[arg(long)]
        slug: String,
    },
    /// Set token approvals for a market
    Set {
        /// Market slug (fetches venue from market data)
        #[arg(long)]
        slug: String,
    },
}

pub async fn execute(
    command: &ApproveCommand,
    output: &OutputFormat,
    api_key_flag: Option<&str>,
    private_key_flag: Option<&str>,
) -> Result<()> {
    match command {
        ApproveCommand::Check { slug } => {
            let api_key = resolve_api_key(api_key_flag)?;
            let pk_str = resolve_private_key(private_key_flag)?;
            let client = LimitlessClient::new(Some(&api_key))?;

            let market = client.get_market(slug).await?;
            let venue = market
                .venue
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Market has no venue"))?;

            let pk = pk_str.strip_prefix("0x").unwrap_or(&pk_str);
            let bytes = hex::decode(pk).context("Invalid private key hex")?;
            let signer = PrivateKeySigner::from_slice(&bytes).context("Invalid private key")?;
            let owner = signer.address();

            let rpc_url = crate::config::load_config()
                .map(|c| c.rpc_url)
                .unwrap_or_else(|| DEFAULT_RPC_URL.to_string());
            let provider = ProviderBuilder::new()
                .connect_http(rpc_url.parse().context("Invalid RPC URL")?);

            let usdc: Address = USDC_ADDRESS.parse().unwrap();
            let ct: Address = CT_ADDRESS.parse().unwrap();
            let exchange: Address = venue.exchange.parse().context("Invalid exchange address")?;

            let usdc_contract = IERC20::new(usdc, &provider);
            let ct_contract = IERC1155::new(ct, &provider);

            let usdc_allowance = usdc_contract
                .allowance(owner, exchange)
                .call()
                .await
                .map(|r| r)
                .unwrap_or(U256::ZERO);

            let ct_approved = ct_contract
                .isApprovedForAll(owner, exchange)
                .call()
                .await
                .map(|r| r)
                .unwrap_or(false);

            let mut results = serde_json::json!({
                "market": slug,
                "exchange": venue.exchange,
                "owner": format!("{}", owner),
                "usdc_allowance": usdc_allowance.to_string(),
                "usdc_approved": usdc_allowance > U256::ZERO,
                "ct_approved_for_all": ct_approved,
            });

            // Check adapter approval for NegRisk
            if let Some(adapter) = &venue.adapter {
                if !adapter.is_empty() && adapter != "0x0000000000000000000000000000000000000000" {
                    let adapter_addr: Address =
                        adapter.parse().context("Invalid adapter address")?;
                    let ct_adapter_approved = ct_contract
                        .isApprovedForAll(owner, adapter_addr)
                        .call()
                        .await
                        .map(|r| r)
                        .unwrap_or(false);
                    results["adapter"] = serde_json::Value::String(adapter.clone());
                    results["ct_approved_for_adapter"] =
                        serde_json::Value::Bool(ct_adapter_approved);
                }
            }

            match output {
                OutputFormat::Json => print_json(&results)?,
                OutputFormat::Table => {
                    println!("Market:     {}", slug);
                    println!("Exchange:   {}", venue.exchange);
                    println!("Owner:      {}", owner);
                    println!(
                        "USDC:       {} (allowance: {})",
                        if usdc_allowance > U256::ZERO {
                            "approved"
                        } else {
                            "NOT approved"
                        },
                        usdc_allowance
                    );
                    println!(
                        "CT:         {}",
                        if ct_approved {
                            "approved for all"
                        } else {
                            "NOT approved"
                        }
                    );
                    if let Some(adapter) = &venue.adapter {
                        if !adapter.is_empty()
                            && adapter != "0x0000000000000000000000000000000000000000"
                        {
                            let approved = results
                                .get("ct_approved_for_adapter")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            println!(
                                "CT->Adapter: {}",
                                if approved {
                                    "approved for all"
                                } else {
                                    "NOT approved"
                                }
                            );
                        }
                    }
                }
            }
        }
        ApproveCommand::Set { slug } => {
            let api_key = resolve_api_key(api_key_flag)?;
            let pk_str = resolve_private_key(private_key_flag)?;
            let client = LimitlessClient::new(Some(&api_key))?;

            let market = client.get_market(slug).await?;
            let venue = market
                .venue
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Market has no venue"))?;

            let pk = pk_str.strip_prefix("0x").unwrap_or(&pk_str);
            let bytes = hex::decode(pk).context("Invalid private key hex")?;
            let signer = PrivateKeySigner::from_slice(&bytes).context("Invalid private key")?;

            let rpc_url = crate::config::load_config()
                .map(|c| c.rpc_url)
                .unwrap_or_else(|| DEFAULT_RPC_URL.to_string());

            let provider = ProviderBuilder::new()
                .wallet(alloy::network::EthereumWallet::from(signer))
                .connect_http(rpc_url.parse().context("Invalid RPC URL")?);

            let usdc: Address = USDC_ADDRESS.parse().unwrap();
            let ct: Address = CT_ADDRESS.parse().unwrap();
            let exchange: Address = venue.exchange.parse().context("Invalid exchange address")?;

            let usdc_contract = IERC20::new(usdc, &provider);
            let ct_contract = IERC1155::new(ct, &provider);

            let mut tx_hashes = Vec::new();

            // 1. Approve USDC
            println!("Approving USDC to exchange...");
            let tx = usdc_contract
                .approve(exchange, U256::MAX)
                .send()
                .await
                .context("Failed to send USDC approval tx")?;
            let hash = *tx.tx_hash();
            println!("USDC approval tx: 0x{}", hex::encode(hash));
            tx_hashes.push(("USDC -> Exchange", format!("0x{}", hex::encode(hash))));

            // 2. Approve CT
            println!("Approving CT to exchange...");
            let tx = ct_contract
                .setApprovalForAll(exchange, true)
                .send()
                .await
                .context("Failed to send CT approval tx")?;
            let hash = *tx.tx_hash();
            println!("CT approval tx: 0x{}", hex::encode(hash));
            tx_hashes.push(("CT -> Exchange", format!("0x{}", hex::encode(hash))));

            // 3. NegRisk adapter approval if needed
            if let Some(adapter) = &venue.adapter {
                if !adapter.is_empty() && adapter != "0x0000000000000000000000000000000000000000" {
                    let adapter_addr: Address =
                        adapter.parse().context("Invalid adapter address")?;
                    println!("Approving CT to NegRisk adapter...");
                    let tx = ct_contract
                        .setApprovalForAll(adapter_addr, true)
                        .send()
                        .await
                        .context("Failed to send CT adapter approval tx")?;
                    let hash = *tx.tx_hash();
                    println!("CT adapter approval tx: 0x{}", hex::encode(hash));
                    tx_hashes.push(("CT -> Adapter", format!("0x{}", hex::encode(hash))));
                }
            }

            match output {
                OutputFormat::Json => {
                    let result: Vec<serde_json::Value> = tx_hashes
                        .iter()
                        .map(|(label, hash)| {
                            serde_json::json!({"approval": label, "tx_hash": hash})
                        })
                        .collect();
                    print_json(&result)?;
                }
                OutputFormat::Table => {
                    println!();
                    println!("All approvals submitted successfully!");
                    for (label, hash) in &tx_hashes {
                        println!("  {}: {}", label, hash);
                    }
                }
            }
        }
    }

    Ok(())
}
