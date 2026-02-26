pub mod markets;
pub mod portfolio;
pub mod profiles;
pub mod public_portfolio;
pub mod trading;

use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::constants::API_BASE_URL;

pub struct LimitlessClient {
    http: reqwest::Client,
    base_url: String,
}

impl LimitlessClient {
    pub fn new(api_key: Option<&str>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        // Don't set Content-Type as default — reqwest's .json() sets it automatically
        // for POST requests. Setting it globally breaks DELETE requests (empty body
        // with Content-Type: application/json is rejected by the server).
        if let Some(key) = api_key {
            headers.insert(
                "X-API-Key",
                HeaderValue::from_str(key).context("Invalid API key format")?,
            );
        }

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            http,
            base_url: API_BASE_URL.to_string(),
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("HTTP request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("API error {status}: {body}");
        }

        resp.json::<T>().await.context("Failed to parse response")
    }

    pub async fn get_with_params<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .query(params)
            .send()
            .await
            .context("HTTP request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("API error {status}: {body}");
        }

        resp.json::<T>().await.context("Failed to parse response")
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .json(body)
            .send()
            .await
            .context("HTTP request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("API error {status}: {body}");
        }

        resp.json::<T>().await.context("Failed to parse response")
    }

    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .delete(&url)
            .send()
            .await
            .context("HTTP request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("API error {status}: {body}");
        }

        resp.json::<T>().await.context("Failed to parse response")
    }
}
