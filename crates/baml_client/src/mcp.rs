use anyhow::Result;
use domain::*;
use serde_json::{json, Value};
use tracing::info;

pub struct McpClient {
    server_url: String,
    http_client: reqwest::Client,
}

impl McpClient {
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn balance(&self, req: &BalanceRequest) -> Result<String> {
        let response = self
            .http_client
            .post(&format!("{}/balance", self.server_url))
            .json(&json!({
                "who": match req.who() {
                    domain::AddressOrEns::Address(addr) => addr.as_str(),
                    domain::AddressOrEns::Ens(ens) => ens.as_str(),
                }
            }))
            .send()
            .await?;

        let result: Value = response.json().await?;
        info!("Balance response: {}", serde_json::to_string_pretty(&result)?);
        
        Ok(result["balance"].as_str().unwrap_or("0").to_string())
    }

    pub async fn code(&self, req: &CodeRequest) -> Result<(bool, u64)> {
        let response = self
            .http_client
            .post(&format!("{}/code", self.server_url))
            .json(&json!({
                "addr": req.addr().as_str()
            }))
            .send()
            .await?;

        let result: Value = response.json().await?;
        info!("Code response: {}", serde_json::to_string_pretty(&result)?);
        
        let deployed = result["deployed"].as_bool().unwrap_or(false);
        let bytecode_len = result["bytecode_len"].as_u64().unwrap_or(0);
        Ok((deployed, bytecode_len))
    }

    pub async fn erc20_balance_of(&self, req: &Erc20BalanceRequest) -> Result<String> {
        let response = self
            .http_client
            .post(&format!("{}/erc20_balance_of", self.server_url))
            .json(&json!({
                "token": req.token().as_str(),
                "holder": req.holder().as_str()
            }))
            .send()
            .await?;

        let result: Value = response.json().await?;
        info!("ERC20 balance response: {}", serde_json::to_string_pretty(&result)?);
        
        Ok(result["amount"].as_str().unwrap_or("0").to_string())
    }

    pub async fn send(&self, req: &SendRequest) -> Result<TxResult> {
        let response = self
            .http_client
            .post(&format!("{}/send", self.server_url))
            .json(&json!({
                "from": req.from().as_str(),
                "to": req.to().as_str(),
                "amount_eth": req.amount_eth(),
                "simulate": req.simulate(),
                "fork_block": req.fork_block()
            }))
            .send()
            .await?;

        let result: Value = response.json().await?;
        info!("Send response: {}", serde_json::to_string_pretty(&result)?);
        
        Ok(TxResult::new(
            result["tx_hash"].as_str().unwrap_or("").to_string(),
            None, // gas_used
            result["success"].as_bool(), // status
        ))
    }

    // Bonus: external API token lookup
    pub async fn token_lookup_address(&self, symbol: &str, chain: &str) -> Result<Option<String>> {
        let response = self
            .http_client
            .post(&format!("{}/token_lookup", self.server_url))
            .json(&json!({
                "symbol": symbol,
                "chain": chain
            }))
            .send()
            .await?;

        let result: Value = response.json().await?;
        info!("Token lookup response: {}", serde_json::to_string_pretty(&result)?);
        Ok(result["address"].as_str().map(|s| s.to_string()))
    }
}
