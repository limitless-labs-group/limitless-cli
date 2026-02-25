use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::LimitlessClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioPositions {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioTrades {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PnlChart {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioHistory {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioPoints {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingAllowance {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl LimitlessClient {
    pub async fn get_positions(&self) -> Result<serde_json::Value> {
        self.get("/portfolio/positions").await
    }

    pub async fn get_trades(&self) -> Result<serde_json::Value> {
        self.get("/portfolio/trades").await
    }

    pub async fn get_pnl_chart(&self, timeframe: Option<&str>) -> Result<serde_json::Value> {
        let mut params = Vec::new();
        if let Some(tf) = timeframe {
            params.push(("timeframe", tf));
        }
        self.get_with_params("/portfolio/pnl-chart", &params).await
    }

    pub async fn get_history(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> Result<serde_json::Value> {
        let mut params = Vec::new();
        let page_str;
        let limit_str;
        if let Some(p) = page {
            page_str = p.to_string();
            params.push(("page", page_str.as_str()));
        }
        if let Some(l) = limit {
            limit_str = l.to_string();
            params.push(("limit", limit_str.as_str()));
        }
        self.get_with_params("/portfolio/history", &params).await
    }

    pub async fn get_points(&self) -> Result<serde_json::Value> {
        self.get("/portfolio/points").await
    }

    pub async fn get_trading_allowance(
        &self,
        trading_type: Option<&str>,
        spender: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut params = Vec::new();
        if let Some(t) = trading_type {
            params.push(("type", t));
        }
        if let Some(s) = spender {
            params.push(("spender", s));
        }
        self.get_with_params("/portfolio/trading/allowance", &params)
            .await
    }
}
