use alloy::primitives::{Address, U256};
use anyhow::{bail, Result};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use super::{random_salt, Order};
use crate::constants::ZERO_ADDRESS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_u8(&self) -> u8 {
        match self {
            Side::Buy => 0,
            Side::Sell => 1,
        }
    }
}

impl std::str::FromStr for Side {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "buy" | "b" | "0" => Ok(Side::Buy),
            "sell" | "s" | "1" => Ok(Side::Sell),
            _ => bail!("Invalid side: {}. Use 'buy' or 'sell'.", s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    Yes,
    No,
}

impl std::str::FromStr for Outcome {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "yes" | "y" | "0" => Ok(Outcome::Yes),
            "no" | "n" | "1" => Ok(Outcome::No),
            _ => bail!("Invalid outcome: {}. Use 'yes' or 'no'.", s),
        }
    }
}

const USDC_SCALE: u64 = 1_000_000; // 1e6

pub fn build_gtc_order(
    maker: Address,
    token_id: U256,
    side: Side,
    price: Decimal,
    size: Decimal,
    fee_rate_bps: u64,
    nonce: u64,
) -> Result<Order> {
    if price <= Decimal::ZERO || price >= Decimal::ONE {
        bail!("Price must be between 0 and 1 (exclusive), got {}", price);
    }
    if size <= Decimal::ZERO {
        bail!("Size must be positive, got {}", size);
    }

    let scale = Decimal::from(USDC_SCALE);

    let (maker_amount, taker_amount) = match side {
        Side::Buy => {
            // Buying shares: pay price*shares USDC, receive shares
            let usdc_amount = (price * size * scale)
                .to_u128()
                .ok_or_else(|| anyhow::anyhow!("makerAmount overflow"))?;
            let shares = (size * scale)
                .to_u128()
                .ok_or_else(|| anyhow::anyhow!("takerAmount overflow"))?;
            (U256::from(usdc_amount), U256::from(shares))
        }
        Side::Sell => {
            // Selling shares: pay shares, receive price*shares USDC
            let shares = (size * scale)
                .to_u128()
                .ok_or_else(|| anyhow::anyhow!("makerAmount overflow"))?;
            let usdc_amount = (price * size * scale)
                .to_u128()
                .ok_or_else(|| anyhow::anyhow!("takerAmount overflow"))?;
            (U256::from(shares), U256::from(usdc_amount))
        }
    };

    let taker: Address = ZERO_ADDRESS.parse().unwrap();

    Ok(Order {
        salt: random_salt(),
        maker,
        signer: maker,
        taker,
        tokenId: token_id,
        makerAmount: maker_amount,
        takerAmount: taker_amount,
        expiration: U256::ZERO, // No expiration for GTC
        nonce: U256::from(nonce),
        feeRateBps: U256::from(fee_rate_bps),
        side: side.as_u8(),
        signatureType: 0, // EOA
    })
}

pub fn build_fok_order(
    maker: Address,
    token_id: U256,
    side: Side,
    maker_amount: U256,
    taker_amount: U256,
    fee_rate_bps: u64,
    nonce: u64,
) -> Order {
    let taker: Address = ZERO_ADDRESS.parse().unwrap();

    Order {
        salt: random_salt(),
        maker,
        signer: maker,
        taker,
        tokenId: token_id,
        makerAmount: maker_amount,
        takerAmount: taker_amount,
        expiration: U256::ZERO,
        nonce: U256::from(nonce),
        feeRateBps: U256::from(fee_rate_bps),
        side: side.as_u8(),
        signatureType: 0,
    }
}
