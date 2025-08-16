use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct CachedAbi {
    pub abi: String,
    pub cached_at: Instant,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct CachedContract {
    pub address: String,
    pub name: String,
    pub abi: Option<String>,
    pub cached_at: Instant,
}

pub struct LruCache {
    abis: HashMap<String, CachedAbi>,
    contracts: HashMap<String, CachedContract>,
    max_size: usize,
    ttl: Duration,
}

impl LruCache {
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            abis: HashMap::new(),
            contracts: HashMap::new(),
            max_size,
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get_abi(&self, key: &str) -> Option<&CachedAbi> {
        self.abis.get(key).and_then(|cached| {
            if cached.cached_at.elapsed() < self.ttl {
                Some(cached)
            } else {
                None
            }
        })
    }

    pub fn set_abi(&mut self, key: String, abi: String, verified: bool) {
        if self.abis.len() >= self.max_size {
            // Simple LRU: remove oldest entry
            let oldest_key = self.abis.keys().next().cloned();
            if let Some(old_key) = oldest_key {
                self.abis.remove(&old_key);
            }
        }
        
        self.abis.insert(key, CachedAbi {
            abi,
            cached_at: Instant::now(),
            verified,
        });
    }

    pub fn get_contract(&self, address: &str) -> Option<&CachedContract> {
        self.contracts.get(address).and_then(|cached| {
            if cached.cached_at.elapsed() < self.ttl {
                Some(cached)
            } else {
                None
            }
        })
    }

    pub fn set_contract(&mut self, address: String, name: String, abi: Option<String>) {
        if self.contracts.len() >= self.max_size {
            // Simple LRU: remove oldest entry
            let oldest_key = self.contracts.keys().next().cloned();
            if let Some(key) = oldest_key {
                self.contracts.remove(&key);
            }
        }
        
        self.contracts.insert(address.clone(), CachedContract {
            address,
            name,
            abi,
            cached_at: Instant::now(),
        });
    }

    pub fn clear_expired(&mut self) {
        let now = Instant::now();
        self.abis.retain(|_, cached| now.duration_since(cached.cached_at) < self.ttl);
        self.contracts.retain(|_, cached| now.duration_since(cached.cached_at) < self.ttl);
    }
}

// Etherscan API interface for fallback
pub struct EtherscanClient {
    api_key: String,
    base_url: String,
}

impl EtherscanClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.etherscan.io/api".to_string(),
        }
    }

    pub async fn get_contract_abi(&self, address: &str) -> Result<Option<String>> {
        let url = format!(
            "{}?module=contract&action=getabi&address={}&apikey={}",
            self.base_url, address, self.api_key
        );
        
        let response = reqwest::get(&url).await?;
        let result: serde_json::Value = response.json().await?;
        
        if result["status"] == "1" {
            Ok(Some(result["result"].as_str().unwrap_or("").to_string()))
        } else {
            Ok(None)
        }
    }

    pub async fn get_contract_name(&self, address: &str) -> Result<Option<String>> {
        let url = format!(
            "{}?module=contract&action=getcontractcreation&contractaddresses={}&apikey={}",
            self.base_url, address, self.api_key
        );
        
        let response = reqwest::get(&url).await?;
        let result: serde_json::Value = response.json().await?;
        
        if result["status"] == "1" {
            let contracts = result["result"].as_array();
            if let Some(contracts) = contracts {
                if let Some(contract) = contracts.first() {
                    return Ok(contract["contractName"].as_str().map(|s| s.to_string()));
                }
            }
        }
        
        Ok(None)
    }
}

// Interface for future L2Beat-style discovery
pub trait ContractDiscovery {
    async fn get_contract_info(&self, address: &str) -> Result<Option<CachedContract>>;
    async fn get_abi(&self, address: &str) -> Result<Option<String>>;
}

impl ContractDiscovery for EtherscanClient {
    async fn get_contract_info(&self, address: &str) -> Result<Option<CachedContract>> {
        let name = self.get_contract_name(address).await?;
        let abi = self.get_contract_abi(address).await?;
        
        if name.is_some() || abi.is_some() {
            Ok(Some(CachedContract {
                address: address.to_string(),
                name: name.unwrap_or_else(|| "Unknown".to_string()),
                abi,
                cached_at: Instant::now(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_abi(&self, address: &str) -> Result<Option<String>> {
        self.get_contract_abi(address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = LruCache::new(2, 3600);
        
        cache.set_abi("key1".to_string(), "abi1".to_string(), true);
        cache.set_abi("key2".to_string(), "abi2".to_string(), false);
        
        assert!(cache.get_abi("key1").is_some());
        assert!(cache.get_abi("key2").is_some());
        
        // Should evict key1 when adding key3
        cache.set_abi("key3".to_string(), "abi3".to_string(), true);
        assert!(cache.get_abi("key1").is_none());
        assert!(cache.get_abi("key2").is_some());
        assert!(cache.get_abi("key3").is_some());
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = LruCache::new(10, 1); // 1 second TTL
        
        cache.set_abi("key1".to_string(), "abi1".to_string(), true);
        assert!(cache.get_abi("key1").is_some());
        
        // Wait for expiration
        std::thread::sleep(Duration::from_secs(2));
        assert!(cache.get_abi("key1").is_none());
    }
}
