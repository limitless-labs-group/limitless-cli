use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::LimitlessClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderbookResponse {
    #[serde(default, deserialize_with = "deserialize_optional_f64")]
    pub adjusted_midpoint: Option<f64>,
    #[serde(default)]
    pub asks: Vec<OrderLevel>,
    #[serde(default)]
    pub bids: Vec<OrderLevel>,
    #[serde(default, deserialize_with = "deserialize_optional_f64")]
    pub last_trade_price: Option<f64>,
    #[serde(default)]
    pub max_spread: Option<serde_json::Value>,
    #[serde(default)]
    pub min_size: Option<serde_json::Value>,
    #[serde(default)]
    pub token_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderLevel {
    #[serde(deserialize_with = "deserialize_f64")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub size: Decimal,
}

fn deserialize_f64<'de, D>(deserializer: D) -> std::result::Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::Number(n) => n.as_f64().ok_or_else(|| serde::de::Error::custom("not f64")),
        serde_json::Value::String(s) => s.parse::<f64>().map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom("expected number or string")),
    }
}

fn deserialize_optional_f64<'de, D>(deserializer: D) -> std::result::Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Option<serde_json::Value> = Deserialize::deserialize(deserializer)?;
    match v {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(n)) => Ok(n.as_f64()),
        Some(serde_json::Value::String(s)) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<f64>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        _ => Err(serde::de::Error::custom("expected number, string, or null")),
    }
}

fn deserialize_decimal<'de, D>(deserializer: D) -> std::result::Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::Number(n) => {
            let s = n.to_string();
            s.parse::<Decimal>().map_err(serde::de::Error::custom)
        }
        serde_json::Value::String(s) => s.parse::<Decimal>().map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom("expected number or string")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOrder {
    pub id: String,
    pub side: String,
    pub price: String,
    #[serde(default)]
    pub quantity: Option<String>,
    #[serde(default)]
    pub size: Option<String>,
    pub status: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub token_id: Option<String>,
    #[serde(default)]
    pub order_type: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOrdersResponse {
    pub orders: Vec<UserOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedBalance {
    #[serde(default)]
    pub locked_balance: Option<String>,
    #[serde(default)]
    pub locked_balance_formatted: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub order_count: Option<u32>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// API returns: [{title: "YES Token", prices: [{price, timestamp}]}]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalPriceSeries {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub prices: Vec<HistoricalPricePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalPricePoint {
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Feed event from GET /markets/{slug}/events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketFeedEvent {
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub side: Option<u64>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub maker_amount: Option<String>,
    #[serde(default)]
    pub taker_amount: Option<String>,
    #[serde(default)]
    pub matched_size: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub token_id: Option<String>,
    #[serde(default)]
    pub tx_hash: Option<String>,
    #[serde(default)]
    pub profile: Option<EventProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventProfile {
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketEventsResponse {
    #[serde(default)]
    pub events: Vec<MarketFeedEvent>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub total_pages: Option<u32>,
    #[serde(default)]
    pub total_rows: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderPayload {
    pub order: serde_json::Value,
    pub order_type: String,
    pub market_slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl LimitlessClient {
    pub async fn get_orderbook(&self, slug: &str) -> Result<OrderbookResponse> {
        self.get(&format!("/markets/{}/orderbook", slug)).await
    }

    pub async fn get_user_orders(
        &self,
        slug: &str,
        statuses: Option<&[&str]>,
        limit: Option<u32>,
    ) -> Result<UserOrdersResponse> {
        let mut params = Vec::new();
        let limit_str;
        if let Some(l) = limit {
            limit_str = l.to_string();
            params.push(("limit", limit_str.as_str()));
        }
        // Statuses as repeated query params
        let status_strs: Vec<String>;
        if let Some(s) = statuses {
            status_strs = s.iter().map(|x| x.to_string()).collect();
            for st in &status_strs {
                params.push(("statuses", st.as_str()));
            }
        }

        self.get_with_params(&format!("/markets/{}/user-orders", slug), &params)
            .await
    }

    pub async fn get_locked_balance(&self, slug: &str) -> Result<LockedBalance> {
        self.get(&format!("/markets/{}/locked-balance", slug)).await
    }

    pub async fn get_historical_prices(
        &self,
        slug: &str,
        from: Option<&str>,
        to: Option<&str>,
        interval: Option<&str>,
    ) -> Result<Vec<HistoricalPriceSeries>> {
        let mut params = Vec::new();
        if let Some(f) = from {
            params.push(("from", f));
        }
        if let Some(t) = to {
            params.push(("to", t));
        }
        if let Some(i) = interval {
            params.push(("interval", i));
        }

        self.get_with_params(&format!("/markets/{}/historical-price", slug), &params)
            .await
    }

    pub async fn get_market_events(
        &self,
        slug: &str,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> Result<MarketEventsResponse> {
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

        self.get_with_params(&format!("/markets/{}/events", slug), &params)
            .await
    }

    pub async fn create_order(&self, payload: &CreateOrderPayload) -> Result<CreateOrderResponse> {
        self.post("/orders", payload).await
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelResponse> {
        self.delete(&format!("/orders/{}", order_id)).await
    }

    pub async fn cancel_batch(&self, order_ids: &[String]) -> Result<CancelResponse> {
        let body = serde_json::json!({ "orderIds": order_ids });
        self.post("/orders/cancel-batch", &body).await
    }

    pub async fn cancel_all(&self, slug: &str) -> Result<CancelResponse> {
        self.delete(&format!("/orders/all/{}", slug)).await
    }
}
