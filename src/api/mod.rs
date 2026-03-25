use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, info};

use crate::config::GardenConfig;
use crate::models::{
    OrderStatusResponse, Quote, QuoteResponse, SubmitOrderRequest, SubmitOrderResponse,
};

pub struct GardenApiClient {
    client: Client,
    config: GardenConfig,
    auth_token: Option<String>,
}

impl GardenApiClient {
    pub fn new(config: GardenConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .cookie_store(true) // Enable cookie storage for SIWE
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { 
            client, 
            config,
            auth_token: None,
        })
    }

    fn app_id_header(&self) -> (&'static str, String) {
        ("garden-app-id", self.config.app_id.clone())
    }

    fn kiosk_header(&self) -> (&'static str, &'static str) {
        ("x-kiosk-mode", "enabled")
    }

    /// Get Authorization header if we have a token
    fn auth_header(&self) -> Option<(&'static str, String)> {
        self.auth_token.as_ref().map(|token| {
            ("Authorization", format!("Bearer {}", token))
        })
    }

    /// Authenticate using SIWE to enable gasless transactions
    pub async fn authenticate_siwe(&mut self, evm_private_key: &str) -> Result<()> {
        use ethers::prelude::*;
        use ethers::signers::{LocalWallet, Signer};
        
        info!("Authenticating with SIWE to enable gasless transactions...");
        
        // Parse wallet
        let wallet: LocalWallet = evm_private_key.parse()
            .context("Failed to parse EVM private key for SIWE")?;
        let address = format!("{:?}", wallet.address());
        
        info!("SIWE: Getting nonce for address {}", address);
        
        // Step 1: Get nonce
        let nonce_url = format!("{}/siwe/challenges", self.config.api_base_url);
        let nonce_resp = self.client
            .post(&nonce_url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to get SIWE nonce")?;
        
        let nonce_body: Value = nonce_resp.json().await
            .context("Failed to parse nonce response")?;
        
        let nonce = nonce_body["result"]
            .as_str()
            .ok_or_else(|| anyhow!("No nonce in response"))?;
        
        info!("SIWE: Got nonce: {}", nonce);
        
        // Step 2: Create SIWE message
        let domain = "testnet.garden.finance";
        let uri = format!("https://{}", domain);
        let statement = "Garden.fi";
        let chain_id = 1; // Mainnet for signing (doesn't matter for testnet API)
        
        let expiration_time = chrono::Utc::now() + chrono::Duration::minutes(5);
        
        let message = format!(
            "{} wants you to sign in with your Ethereum account:\n\
            {}\n\n\
            {}\n\n\
            URI: {}\n\
            Version: 1\n\
            Chain ID: {}\n\
            Nonce: {}\n\
            Issued At: {}\n\
            Expiration Time: {}",
            domain,
            address,
            statement,
            uri,
            chain_id,
            nonce,
            chrono::Utc::now().to_rfc3339(),
            expiration_time.to_rfc3339()
        );
        
        info!("SIWE: Signing message...");
        debug!("SIWE message:\n{}", message);
        
        // Step 3: Sign the message
        let signature = wallet.sign_message(&message).await
            .context("Failed to sign SIWE message")?;
        let signature_hex = format!("0x{}", signature);
        
        info!("SIWE: Signature: {}", signature_hex);
        
        // Step 4: Get token
        let token_url = format!("{}/siwe/tokens", self.config.api_base_url);
        let token_payload = serde_json::json!({
            "message": message,
            "signature": signature_hex,
            "nonce": nonce
        });
        
        info!("SIWE: Getting auth token...");
        
        let token_resp = self.client
            .post(&token_url)
            .header("Content-Type", "application/json")
            .json(&token_payload)
            .send()
            .await
            .context("Failed to get SIWE token")?;
        
        let token_body: Value = token_resp.json().await
            .context("Failed to parse token response")?;
        
        let token = token_body["result"]
            .as_str()
            .ok_or_else(|| anyhow!("No token in response: {:?}", token_body))?
            .to_string();
        
        info!("SIWE: ✅ Authentication successful! Token obtained.");
        self.auth_token = Some(token);
        
        Ok(())
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
        let (kiosk_k, kiosk_v) = self.kiosk_header();
        
        let mut req = self
            .client
            .post(&url)
            .header(hk, &hv)
            .header(kiosk_k, kiosk_v)
            .header("content-type", "application/json");
        
        // Add Authorization header if we have a token
        if let Some((auth_k, auth_v)) = self.auth_header() {
            info!("Using Authorization header for gasless");
            req = req.header(auth_k, auth_v);
        }
        
        let resp = req
            .body(body)
            .send()
            .await
            .context("Submit order request failed")?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .context("Failed to read submit order body")?;
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

    /// Initiate swap using gasless PATCH endpoint (for EVM with signature)
    pub async fn initiate_swap_gasless_evm(
        &self,
        order_id: &str,
        signature: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/v2/orders/{}?action=initiate",
            self.config.api_base_url, order_id
        );

        info!("PATCH initiate swap (gasless EVM) for order {}", order_id);
        
        let payload = serde_json::json!({
            "signature": signature
        });
        
        info!("Request URL: {}", url);
        info!("Payload: {}", serde_json::to_string_pretty(&payload).unwrap_or_default());

        let (hk, hv) = self.app_id_header();
        
        let resp = self
            .client
            .patch(&url)
            .header(hk, &hv)
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Initiate swap gasless request failed")?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .context("Failed to read initiate response body")?;
        info!("Initiate response [{}]: {}", status, body);

        if !status.is_success() {
            return Err(anyhow!("Initiate swap API {} - {}", status, body));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&body).context("Failed to parse initiate response")?;

        if parsed.get("status").and_then(|s| s.as_str()) != Some("Ok") {
            return Err(anyhow!("Initiate swap non-Ok: {}", body));
        }

        Ok(())
    }

    /// Initiate swap using gasless endpoint (for Solana)
    /// Uses PATCH to /v2/orders/{id}?action=initiate with signature
    pub async fn initiate_swap_gasless_solana(
        &self,
        order_id: &str,
        serialized_tx: &str,
    ) -> Result<String> {
        let url = format!(
            "{}/v2/orders/{}?action=initiate",
            self.config.api_base_url, order_id
        );

        info!("PATCH initiate swap (gasless Solana) for order {}", order_id);
        info!("Request URL: {}", url);
        
        let payload = serde_json::json!({
            "signature": serialized_tx
        });

        let (hk, hv) = self.app_id_header();
        
        let mut req = self
            .client
            .patch(&url)
            .header(hk, &hv)
            .header("content-type", "application/json");
        
        // Add Authorization header if we have a token
        if let Some((auth_k, auth_v)) = self.auth_header() {
            info!("Using Authorization header for gasless Solana initiation");
            req = req.header(auth_k, auth_v);
        }
        
        let resp = req
            .json(&payload)
            .send()
            .await
            .context("Initiate swap gasless Solana request failed")?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .context("Failed to read initiate response body")?;
        info!("Initiate response [{}]: {}", status, body);

        if !status.is_success() {
            return Err(anyhow!("Initiate swap API {} - {}", status, body));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&body).context("Failed to parse initiate response")?;

        if parsed.get("status").and_then(|s| s.as_str()) != Some("Ok") {
            return Err(anyhow!("Initiate swap non-Ok: {}", body));
        }

        // Return the transaction hash from result
        let tx_hash = parsed.get("result")
            .and_then(|r| r.as_str())
            .ok_or_else(|| anyhow!("No result in initiate response"))?
            .to_string();

        Ok(tx_hash)
    }

    /// Initiate swap using gasless PATCH endpoint (legacy, for compatibility)
    pub async fn initiate_swap_gasless(
        &self,
        order_id: &str,
        payload: serde_json::Value,
    ) -> Result<()> {
        let url = format!(
            "{}/v2/orders/{}?action=initiate",
            self.config.api_base_url, order_id
        );

        info!("PATCH initiate swap (gasless) for order {}", order_id);
        info!("Request URL: {}", url);
        info!("Payload: {}", serde_json::to_string_pretty(&payload).unwrap_or_default());

        let (hk, hv) = self.app_id_header();
        
        // Try PATCH first (as documented)
        let resp = self
            .client
            .patch(&url)
            .header(hk, &hv)
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Initiate swap gasless request failed")?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .context("Failed to read initiate response body")?;
        info!("Initiate response [{}]: {}", status, body);

        if !status.is_success() {
            return Err(anyhow!("Initiate swap API {} - {}", status, body));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&body).context("Failed to parse initiate response")?;

        if parsed.get("status").and_then(|s| s.as_str()) != Some("Ok") {
            return Err(anyhow!("Initiate swap non-Ok: {}", body));
        }

        Ok(())
    }
}
