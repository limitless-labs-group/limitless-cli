use std::collections::HashMap;

use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::LimitlessClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    #[serde(default)]
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub trade_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    /// Raw volume in micro-USDC (e.g. "5702208369")
    #[serde(default)]
    pub volume: Option<String>,
    /// Human-readable volume (e.g. "5702.208369")
    #[serde(default)]
    pub volume_formatted: Option<String>,
    #[serde(default)]
    pub liquidity: Option<String>,
    /// Formatted expiration date (e.g. "Feb 25, 2026")
    #[serde(default)]
    pub expiration_date: Option<String>,
    #[serde(default)]
    pub expiration_timestamp: Option<u64>,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    /// Prices array: [yes_price, no_price]
    #[serde(default)]
    pub prices: Option<Vec<f64>>,
    #[serde(default)]
    pub position_ids: Option<Vec<String>>,
    /// Token IDs: { "yes": "...", "no": "..." }
    #[serde(default)]
    pub tokens: Option<MarketTokens>,
    #[serde(default)]
    pub venue: Option<Venue>,
    #[serde(default)]
    pub collateral_token: Option<CollateralToken>,
    #[serde(default)]
    pub resolution_source: Option<String>,
    #[serde(default)]
    pub automation_type: Option<String>,
    #[serde(default)]
    pub market_type: Option<String>,
    // NegRisk group fields
    #[serde(default)]
    pub markets: Option<Vec<Market>>,
    // Additional useful fields
    #[serde(default)]
    pub condition_id: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub categories: Option<Vec<String>>,
    #[serde(default)]
    pub open_interest_formatted: Option<String>,
    #[serde(default)]
    pub liquidity_formatted: Option<String>,
}

impl Market {
    /// Get YES price from the prices array
    pub fn yes_price(&self) -> Option<f64> {
        self.prices.as_ref().and_then(|p| p.first().copied())
    }
    /// Get NO price from the prices array
    pub fn no_price(&self) -> Option<f64> {
        self.prices.as_ref().and_then(|p| p.get(1).copied())
    }
    /// Get formatted volume as a decimal string
    pub fn display_volume(&self) -> Option<Decimal> {
        self.volume_formatted
            .as_ref()
            .and_then(|v| v.parse::<Decimal>().ok())
    }
    /// Get expiration display string with time from timestamp
    pub fn display_deadline(&self) -> Option<String> {
        if let Some(ts) = self.expiration_timestamp {
            // Convert millis timestamp to "Feb 25, 16:00 UTC"
            let secs = (ts / 1000) as i64;
            let dt = chrono::DateTime::from_timestamp(secs, 0);
            if let Some(dt) = dt {
                return Some(dt.format("%b %d, %H:%M UTC").to_string());
            }
        }
        self.expiration_date
            .clone()
            .or_else(|| self.deadline.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTokens {
    #[serde(default)]
    pub yes: Option<String>,
    #[serde(default)]
    pub no: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Venue {
    pub exchange: String,
    #[serde(default)]
    pub adapter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollateralToken {
    pub symbol: String,
    pub address: String,
    pub decimals: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveMarketsResponse {
    pub data: Vec<Market>,
    #[serde(default)]
    pub next_page: Option<u32>,
    /// API returns totalMarketsCount
    #[serde(default)]
    pub total_markets_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMarketsResponse {
    pub markets: Vec<Market>,
    #[serde(default)]
    pub total_markets_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSlug {
    pub slug: String,
    #[serde(default)]
    pub ticker: Option<String>,
    #[serde(default)]
    pub strike_price: Option<String>,
    #[serde(default)]
    pub deadline: Option<String>,
}

/// API returns: {"category": {"1": 28, "2": 16, ...}, "totalCount": 487}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoriesCountResponse {
    pub category: HashMap<String, u32>,
    #[serde(default)]
    pub total_count: Option<u32>,
}

/// Category metadata from GET /categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Combined category info with name + count for display
#[derive(Debug, Clone, Serialize)]
pub struct CategoryWithCount {
    pub id: u32,
    pub name: String,
    pub count: u32,
}

impl LimitlessClient {
    pub async fn get_active_markets(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
        sort_by: Option<&str>,
        trade_type: Option<&str>,
        category: Option<&str>,
    ) -> Result<ActiveMarketsResponse> {
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
        if let Some(s) = sort_by {
            params.push(("sortBy", s));
        }
        if let Some(t) = trade_type {
            params.push(("tradeType", t));
        }

        // Category uses a path parameter, not query: /markets/active/{categoryId}
        let path = match category {
            Some(c) => format!("/markets/active/{}", c),
            None => "/markets/active".to_string(),
        };

        self.get_with_params(&path, &params).await
    }

    pub async fn get_market(&self, slug: &str) -> Result<Market> {
        self.get(&format!("/markets/{}", slug)).await
    }

    pub async fn search_markets(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<SearchMarketsResponse> {
        let mut params = vec![("query", query)];
        let limit_str;
        if let Some(l) = limit {
            limit_str = l.to_string();
            params.push(("limit", limit_str.as_str()));
        }

        self.get_with_params("/markets/search", &params).await
    }

    pub async fn get_active_slugs(&self) -> Result<Vec<MarketSlug>> {
        self.get("/markets/active/slugs").await
    }

    /// Fetch category counts and category names, then join them
    pub async fn get_categories_with_counts(&self) -> Result<(Vec<CategoryWithCount>, Option<u32>)> {
        let counts: CategoriesCountResponse = self.get("/markets/categories/count").await?;
        let categories: Vec<Category> = self.get("/categories").await?;

        // Build id -> name map
        let name_map: HashMap<String, String> = categories
            .into_iter()
            .map(|c| (c.id.to_string(), c.name))
            .collect();

        let mut result: Vec<CategoryWithCount> = counts
            .category
            .iter()
            .map(|(id, count)| CategoryWithCount {
                id: id.parse().unwrap_or(0),
                name: name_map
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| format!("Unknown ({})", id)),
                count: *count,
            })
            .collect();

        result.sort_by(|a, b| b.count.cmp(&a.count));

        Ok((result, counts.total_count))
    }
}
