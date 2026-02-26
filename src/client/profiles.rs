use anyhow::Result;
use serde::Deserialize;

use super::LimitlessClient;

/// Profile response from GET /profiles/public/{address}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: u64,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub rank: Option<ProfileRank>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRank {
    #[serde(default)]
    pub fee_rate_bps: Option<u64>,
}

impl LimitlessClient {
    /// Fetch user profile by wallet address. Returns profile with id (ownerId) and fee rate.
    pub async fn get_profile(&self, address: &str) -> Result<Profile> {
        self.get(&format!("/profiles/public/{}", address)).await
    }
}
