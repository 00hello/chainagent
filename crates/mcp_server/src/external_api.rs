use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub chain: String,
}

#[derive(Debug)]
pub struct TokenLookupClient {
    base_url: String,
    http: reqwest::Client,
    cache_ttl: Duration,
    cache: HashMap<String, (TokenInfo, Instant)>,
}

impl TokenLookupClient {
    pub fn new(base_url: impl Into<String>, cache_ttl_seconds: u64) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::new(),
            cache_ttl: Duration::from_secs(cache_ttl_seconds),
            cache: HashMap::new(),
        }
    }

    fn cache_key(symbol: &str, chain: &str) -> String {
        format!("{}::{}", chain.to_lowercase(), symbol.to_uppercase())
    }

    pub async fn lookup_by_symbol(&mut self, symbol: &str, chain: &str) -> Result<Option<TokenInfo>> {
        // Cache check
        let key = Self::cache_key(symbol, chain);
        if let Some((info, ts)) = self.cache.get(&key) {
            if ts.elapsed() < self.cache_ttl {
                return Ok(Some(info.clone()));
            }
        }

        // HTTP request with simple retry/backoff
        let url = format!("{}/tokens", self.base_url.trim_end_matches('/'));
        let req = self
            .http
            .get(url)
            .query(&[("symbol", symbol), ("chain", chain)]);

        let mut attempt = 0;
        let info: Option<TokenInfo> = loop {
            attempt += 1;
            let resp = req.try_clone().unwrap().send().await;
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        let v: serde_json::Value = r.json().await?;
                        // Expected shape: { address, symbol, name, chain }
                        let address = v.get("address").and_then(|x| x.as_str()).map(|s| s.to_string());
                        let symbol_out = v.get("symbol").and_then(|x| x.as_str()).map(|s| s.to_string());
                        let name = v.get("name").and_then(|x| x.as_str()).map(|s| s.to_string());
                        let chain_out = v.get("chain").and_then(|x| x.as_str()).map(|s| s.to_string()).or_else(|| Some(chain.to_string()));
                        if let (Some(address), Some(symbol_out), Some(name), Some(chain_out)) = (address, symbol_out, name, chain_out) {
                            break Some(TokenInfo { address, symbol: symbol_out, name, chain: chain_out });
                        } else {
                            break None;
                        }
                    } else if r.status().as_u16() == 429 && attempt < 3 {
                        tokio::time::sleep(Duration::from_millis(200 * attempt)).await;
                        continue;
                    } else {
                        break None;
                    }
                }
                Err(_) if attempt < 2 => {
                    tokio::time::sleep(Duration::from_millis(100 * attempt)).await;
                    continue;
                }
                Err(_) => break None,
            }
        };

        // Cache and return
        if let Some(ref info) = info {
            self.cache.insert(key, (info.clone(), Instant::now()));
        }
        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{prelude::*, Method::GET};

    #[tokio::test]
    async fn test_lookup_by_symbol_success() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET)
                .path("/tokens")
                .query_param("symbol", "USDC")
                .query_param("chain", "ethereum");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(serde_json::json!({
                    "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                    "symbol": "USDC",
                    "name": "USD Coin",
                    "chain": "ethereum"
                }));
        });

        let mut client = TokenLookupClient::new(server.base_url(), 60);
        let res = client.lookup_by_symbol("USDC", "ethereum").await.unwrap();
        assert!(res.is_some());
        let info = res.unwrap();
        assert_eq!(info.address, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        assert_eq!(info.symbol, "USDC");
        assert_eq!(info.chain, "ethereum");

        // Second call should hit cache
        let res2 = client.lookup_by_symbol("USDC", "ethereum").await.unwrap();
        assert!(res2.is_some());
        _m.assert_hits(1);
    }

    #[tokio::test]
    async fn test_lookup_not_found() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/tokens");
            then.status(404);
        });
        let mut client = TokenLookupClient::new(server.base_url(), 60);
        let res = client.lookup_by_symbol("FOO", "ethereum").await.unwrap();
        assert!(res.is_none());
    }
}
