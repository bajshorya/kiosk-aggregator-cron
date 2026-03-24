use anyhow::{Context, Result, anyhow};
use reqwest::Client; // ← async Client, not blocking
use tracing::{debug, info};

use crate::config::GardenConfig;
use crate::models::{
    OrderStatusResponse, Quote, QuoteResponse, SubmitOrderRequest, SubmitOrderResponse,
};

pub struct GardenApiClient {
    client: Client,
    config: GardenConfig,
}

impl GardenApiClient {
    pub fn new(config: GardenConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client, config })
    }

    fn app_id_header(&self) -> (&'static str, String) {
        ("garden-app-id", self.config.app_id.clone())
    }

    // All three methods become async
    pub async fn get_quote(
        &self,
        from_asset: &str,
        to_asset: &str,
        from_amount: &str,
    ) -> Result<Quote> {
        let url = format!(
            "{}/v2/quote?from={}&to={}&from_amount={}",
            self.config.api_base_url, from_asset, to_asset, from_amount
        );

        info!("GET quote {} -> {} ({})", from_asset, to_asset, from_amount);
        debug!("URL: {}", url);

        let (hk, hv) = self.app_id_header();
        let resp = self
            .client
            .get(&url)
            .header(hk, &hv)
            .header("accept", "application/json")
            .send() // ← .await added
            .await
            .context("Quote request failed")?;

        let status = resp.status();
        let body = resp.text().await.context("Failed to read quote body")?; // ← .await
        debug!("Quote response [{}]: {}", status, body);

        if !status.is_success() {
            return Err(anyhow!("Quote API {} - {}", status, body));
        }

        let parsed: QuoteResponse =
            serde_json::from_str(&body).context("Failed to parse quote response")?;

        if parsed.status != "Ok" {
            return Err(anyhow!("Quote non-Ok: {}", body));
        }

        parsed
            .result
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No quotes returned for {} -> {}", from_asset, to_asset))
    }

    pub async fn submit_order(&self, request: &SubmitOrderRequest) -> Result<SubmitOrderResponse> {
        let url = format!("{}/v2/orders", self.config.api_base_url);
        let body = serde_json::to_string(request).context("Failed to serialize order")?;

        info!(
            "POST order {} -> {}",
            request.source.asset, request.destination.asset
        );

        let (hk, hv) = self.app_id_header();
        let resp = self
            .client
            .post(&url)
            .header(hk, &hv)
            .header("content-type", "application/json")
            .body(body)
            .send() // ← .await
            .await
            .context("Submit order request failed")?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .context("Failed to read submit order body")?; // ← .await
        debug!("Submit order response [{}]: {}", status, body);

        if !status.is_success() {
            return Err(anyhow!("Submit order API {} - {}", status, body));
        }

        let parsed: SubmitOrderResponse =
            serde_json::from_str(&body).context("Failed to parse submit order response")?;

        if parsed.status != "Ok" {
            return Err(anyhow!("Submit order non-Ok: {}", body));
        }

        Ok(parsed)
    }

    pub async fn get_order_status(&self, order_id: &str) -> Result<OrderStatusResponse> {
        let url = format!("{}/v2/orders/{}", self.config.api_base_url, order_id);
        debug!("GET order status {}", url);

        let (hk, hv) = self.app_id_header();
        let resp = self
            .client
            .get(&url)
            .header(hk, &hv)
            .header("accept", "application/json")
            .send() // ← .await
            .await
            .context("Order status request failed")?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .context("Failed to read order status body")?; // ← .await
        debug!("Order status response [{}]: {}", status, body);

        if !status.is_success() {
            return Err(anyhow!("Order status API {} - {}", status, body));
        }

        let parsed: OrderStatusResponse =
            serde_json::from_str(&body).context("Failed to parse order status response")?;

        Ok(parsed)
    }
}
