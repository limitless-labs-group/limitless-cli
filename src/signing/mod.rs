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

pub fn random_salt() -> U256 {
    use alloy::primitives::FixedBytes;
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("Failed to generate random bytes");
    U256::from_be_bytes(FixedBytes(bytes).0)
}

pub fn parse_address(s: &str) -> Result<Address> {
    s.parse::<Address>()
        .context(format!("Invalid address: {}", s))
}

pub fn parse_u256(s: &str) -> Result<U256> {
    U256::from_str_radix(s, 10).context(format!("Invalid U256: {}", s))
}

pub fn address_to_hex(addr: &Address) -> String {
    format!("0x{}", hex::encode(addr.as_slice()))
}

pub fn u256_to_string(v: &U256) -> String {
    v.to_string()
}

pub fn signature_hex(sig: &[u8]) -> String {
    format!("0x{}", hex::encode(sig))
}
