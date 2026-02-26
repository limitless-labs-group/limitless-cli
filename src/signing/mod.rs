pub mod order;

use alloy::primitives::{Address, U256};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use alloy::sol;
use alloy::sol_types::SolStruct;
use anyhow::{Context, Result};

use crate::constants::CHAIN_ID;

sol! {
    #[derive(Debug)]
    struct Order {
        uint256 salt;
        address maker;
        address signer;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 expiration;
        uint256 nonce;
        uint256 feeRateBps;
        uint8 side;
        uint8 signatureType;
    }
}

pub fn eip712_domain(venue_exchange: Address) -> alloy::sol_types::Eip712Domain {
    alloy::sol_types::Eip712Domain {
        name: Some("Limitless CTF Exchange".into()),
        version: Some("1".into()),
        chain_id: Some(U256::from(CHAIN_ID)),
        verifying_contract: Some(venue_exchange),
        salt: None,
    }
}

pub async fn sign_order(
    signer: &PrivateKeySigner,
    order: &Order,
    venue_exchange: Address,
) -> Result<Vec<u8>> {
    let domain = eip712_domain(venue_exchange);
    let signing_hash = order.eip712_signing_hash(&domain);

    let signature = signer
        .sign_hash(&signing_hash)
        .await
        .context("Failed to sign order")?;

    Ok(signature_to_bytes(&signature))
}

fn signature_to_bytes(sig: &alloy::primitives::Signature) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(65);
    bytes.extend_from_slice(&sig.r().to_be_bytes::<32>());
    bytes.extend_from_slice(&sig.s().to_be_bytes::<32>());
    bytes.push(if sig.v() { 28 } else { 27 });
    bytes
}

/// Generate a random salt that fits in a JavaScript-safe integer (2^53 - 1).
/// The API expects salt as a JSON number, not a string.
pub fn random_salt() -> U256 {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).expect("Failed to generate random bytes");
    // Mask to 53 bits to stay within JS Number.MAX_SAFE_INTEGER
    let val = u64::from_be_bytes(bytes) & ((1u64 << 53) - 1);
    U256::from(val)
}

/// Convert U256 to u64 for JSON serialization as a number.
/// Panics if value exceeds u64::MAX (should never happen for our amounts).
pub fn u256_to_u64(v: &U256) -> u64 {
    v.to::<u64>()
}

pub fn parse_address(s: &str) -> Result<Address> {
    s.parse::<Address>()
        .context(format!("Invalid address: {}", s))
}

pub fn parse_u256(s: &str) -> Result<U256> {
    U256::from_str_radix(s, 10).context(format!("Invalid U256: {}", s))
}

pub fn address_to_hex(addr: &Address) -> String {
    addr.to_checksum(None)
}

pub fn u256_to_string(v: &U256) -> String {
    v.to_string()
}

pub fn signature_hex(sig: &[u8]) -> String {
    format!("0x{}", hex::encode(sig))
}
