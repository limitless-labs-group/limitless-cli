use anyhow::Result;

use super::LimitlessClient;

impl LimitlessClient {
    pub async fn get_public_positions(&self, address: &str) -> Result<serde_json::Value> {
        self.get(&format!("/portfolio/{}/positions", address)).await
    }

    pub async fn get_public_traded_volume(&self, address: &str) -> Result<serde_json::Value> {
        self.get(&format!("/portfolio/{}/traded-volume", address))
            .await
    }

    pub async fn get_public_pnl_chart(
        &self,
        address: &str,
        timeframe: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut params = Vec::new();
        if let Some(tf) = timeframe {
            params.push(("timeframe", tf));
        }
        self.get_with_params(&format!("/portfolio/{}/pnl-chart", address), &params)
            .await
    }
}
