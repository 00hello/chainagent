mod error;
mod constants;
mod validation;
mod cache;
use anyhow::anyhow;
use error::AdapterError;
use constants::*;

use domain::{Address, AddressOrEns, BalanceRequest, CodeRequest, Erc20BalanceRequest, SendRequest, TxResult};
use ethers_contract::Contract;
use ethers_core::abi::parse_abi_str;
use ethers_core::types::{transaction::eip2718::TypedTransaction, Address as EthAddress, Bytes, TransactionRequest, U256};
use ethers_core::utils::parse_ether;
use ethers_middleware::SignerMiddleware;
use ethers_providers::{Http, Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};
use std::collections::HashMap;
use std::str::FromStr;

pub fn is_checksum_address(_addr: &str) -> bool {
    // TODO: implement EIP-55 validation
    true
}

#[derive(Clone, Debug)]
pub struct FoundryAdapter {
    rpc_url: String,
    provider: Provider<Http>,
    gas_cap: u64,
    expected_chain_id: Option<u64>,
    known_wallets: HashMap<String, LocalWallet>,
}

impl FoundryAdapter {
    pub async fn new(rpc_url: impl Into<String>) -> Result<Self, AdapterError> {
        let rpc_url = rpc_url.into();
        let provider = Provider::<Http>::try_from(rpc_url.clone()).map_err(|e| AdapterError::Other(e.into()))?;
        let mut known_wallets = HashMap::new();
        let accounts = get_anvil_accounts();
        let private_keys = vec![
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
            "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
            "0x7c852118e8d7e3b58184ae9b0c2aa26a2d4f9b6c3b6b6b6b6b6b6b6b6b6b6b6b",
            "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a",
        ];
        
        for (addr, key) in accounts.iter().zip(private_keys.iter()) {
            let wallet = LocalWallet::from_str(key)?;
            known_wallets.insert(normalize(&addr.to_string()), wallet);
        }
        
        Ok(Self { rpc_url, provider, gas_cap: DEFAULT_GAS_CAP, expected_chain_id: None, known_wallets })
    }

    pub fn with_expected_chain_id(mut self, chain_id: u64) -> Self {
        self.expected_chain_id = Some(chain_id);
        self
    }

    pub fn with_gas_cap(mut self, gas_cap: u64) -> Self {
        self.gas_cap = gas_cap;
        self
    }

    pub async fn resolve_address_or_ens(&self, input: &AddressOrEns) -> Result<Address, AdapterError> {
        match input {
            AddressOrEns::Address(addr) => {
                let parsed = EthAddress::from_str(addr.as_str()).map_err(|_| AdapterError::AddrParse(addr.as_str().into()))?;
                Ok(Address::new(parsed.to_string()))
            }
            AddressOrEns::Ens(name) => {
                let resolved: EthAddress = self.provider.resolve_name(name.as_str()).await?;
                Ok(Address::new(resolved.to_string()))
            }
        }
    }

    pub async fn get_balance(&self, req: &BalanceRequest) -> Result<String, AdapterError> {
        let addr = self.resolve_address_or_ens(req.who()).await?;
        let addr = EthAddress::from_str(addr.as_str()).map_err(|_| AdapterError::AddrParse(addr.as_str().into()))?;
        let bal: U256 = self.provider.get_balance(addr, None).await?;
        Ok(bal.to_string())
    }

    pub async fn get_code_len(&self, req: &CodeRequest) -> Result<(bool, u64), AdapterError> {
        let addr = EthAddress::from_str(req.addr().as_str()).map_err(|_| AdapterError::AddrParse(req.addr().as_str().into()))?;
        let code: Bytes = self.provider.get_code(addr, None).await?;
        let len = code.0.len() as u64;
        Ok((len > 0, len))
    }

    pub async fn erc20_balance_of(&self, req: &Erc20BalanceRequest) -> Result<String, AdapterError> {
        let token = EthAddress::from_str(req.token().as_str()).map_err(|_| AdapterError::AddrParse(req.token().as_str().into()))?;
        let holder = EthAddress::from_str(req.holder().as_str()).map_err(|_| AdapterError::AddrParse(req.holder().as_str().into()))?;
        let abi = parse_abi_str("[function balanceOf(address) view returns (uint256)]").map_err(|e| AdapterError::Other(e.into()))?;
        let contract = Contract::new(token, abi, self.provider.clone().into());
        let method = contract.method::<_, U256>("balanceOf", holder).map_err(|e| AdapterError::Other(e.into()))?;
        let amount: U256 = method.call().await.map_err(|e| AdapterError::Other(e.into()))?;
        Ok(amount.to_string())
    }

    pub async fn send_eth(&self, req: &SendRequest) -> Result<TxResult, AdapterError> {
        if let Some(expected) = self.expected_chain_id {
            let chain_id = self.provider.get_chainid().await?.as_u64();
            if chain_id != expected {
                return Err(AdapterError::ChainIdMismatch { got: chain_id, expected });
            }
        }
        let from_addr = EthAddress::from_str(req.from().as_str()).map_err(|_| AdapterError::AddrParse(req.from().as_str().into()))?;
        let to_addr = EthAddress::from_str(req.to().as_str()).map_err(|_| AdapterError::AddrParse(req.to().as_str().into()))?;
        let value = parse_ether(req.amount_eth()).map_err(|e| AdapterError::Other(e.into()))?;
        let base = TransactionRequest::new().from(from_addr).to(to_addr).value(value);
        let mut typed: TypedTransaction = base.into();
        let est = self.provider.estimate_gas(&typed, None).await?;
        if est.as_u64() > self.gas_cap {
            return Err(AdapterError::GasCapExceeded { estimated: est.as_u64(), cap: self.gas_cap });
        }
        typed.set_gas(est);
        let _sim = self.provider.call(&typed, None).await?;
        if req.simulate() {
            return Ok(TxResult::new(String::new(), Some(est.as_u64()), None));
        }
        let key = normalize(req.from().as_str());
        let wallet = self.known_wallets.get(&key).cloned().ok_or_else(|| AdapterError::MissingLocalKey(req.from().as_str().to_string()))?;
        let chain_id = self.provider.get_chainid().await?.as_u64();
        let wallet = wallet.with_chain_id(chain_id);
        let client = SignerMiddleware::new(self.provider.clone(), wallet);
        let pending = client
            .send_transaction(typed, None)
            .await
            .map_err(|e| AdapterError::Other(e.into()))?;
        let tx_hash = *pending;
        let receipt = pending.await?;
        if let Some(rcpt) = receipt {
            let status = rcpt.status.map(|s| s.as_u64() == 1);
            let gas_used = rcpt.gas_used.map(|g| g.as_u64());
            Ok(TxResult::new(format!("0x{:x}", rcpt.transaction_hash), gas_used, status))
        } else {
            Ok(TxResult::new(format!("0x{:x}", tx_hash), Some(est.as_u64()), None))
        }
    }
}

pub fn placeholder_adapter() {}

fn normalize(addr: &str) -> String { validation::normalize(addr) }
