use anyhow::{anyhow, Result};
use domain::{Address, AddressOrEns, BalanceRequest, CodeRequest, Erc20BalanceRequest, SendRequest, TxResult};
use ethers_contract::Contract;
use ethers_core::abi::parse_abi_str;
use ethers_core::types::{transaction::eip2718::TypedTransaction, Address as EthAddress, Bytes, TransactionRequest, U256};
use ethers_core::utils::{parse_ether, to_checksum};
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
    pub async fn new(rpc_url: impl Into<String>) -> Result<Self> {
        let rpc_url = rpc_url.into();
        let provider = Provider::<Http>::try_from(rpc_url.clone())?;
        let mut known_wallets = HashMap::new();
        known_wallets.insert(
            normalize("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"),
            LocalWallet::from_str("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")?,
        );
        known_wallets.insert(
            normalize("0x70997970c51812dc3a010c7d01b50e0d17dc79c8"),
            LocalWallet::from_str("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")?,
        );
        Ok(Self { rpc_url, provider, gas_cap: 500_000, expected_chain_id: None, known_wallets })
    }

    pub fn with_expected_chain_id(mut self, chain_id: u64) -> Self {
        self.expected_chain_id = Some(chain_id);
        self
    }

    pub fn with_gas_cap(mut self, gas_cap: u64) -> Self {
        self.gas_cap = gas_cap;
        self
    }

    pub async fn resolve_address_or_ens(&self, input: &AddressOrEns) -> Result<Address> {
        match input {
            AddressOrEns::Address(addr) => {
                let parsed = EthAddress::from_str(addr.as_str())?;
                let checksum = to_checksum(&parsed, None);
                Ok(Address::new(format!("0x{}", checksum.trim_start_matches("0x"))))
            }
            AddressOrEns::Ens(name) => {
                let resolved: EthAddress = self.provider.resolve_name(name.as_str()).await?;
                let checksum = to_checksum(&resolved, None);
                Ok(Address::new(format!("0x{}", checksum.trim_start_matches("0x"))))
            }
        }
    }

    pub async fn get_balance(&self, req: &BalanceRequest) -> Result<String> {
        let addr = self.resolve_address_or_ens(req.who()).await?;
        let addr = EthAddress::from_str(addr.as_str())?;
        let bal: U256 = self.provider.get_balance(addr, None).await?;
        Ok(bal.to_string())
    }

    pub async fn get_code_len(&self, req: &CodeRequest) -> Result<(bool, u64)> {
        let addr = EthAddress::from_str(req.addr().as_str())?;
        let code: Bytes = self.provider.get_code(addr, None).await?;
        let len = code.0.len() as u64;
        Ok((len > 0, len))
    }

    pub async fn erc20_balance_of(&self, req: &Erc20BalanceRequest) -> Result<String> {
        let token = EthAddress::from_str(req.token().as_str())?;
        let holder = EthAddress::from_str(req.holder().as_str())?;
        let abi = parse_abi_str("[function balanceOf(address) view returns (uint256)]")?;
        let contract = Contract::new(token, abi, self.provider.clone().into());
        let amount: U256 = contract.method::<_, U256>("balanceOf", holder)?.call().await?;
        Ok(amount.to_string())
    }

    pub async fn send_eth(&self, req: &SendRequest) -> Result<TxResult> {
        if let Some(expected) = self.expected_chain_id {
            let chain_id = self.provider.get_chainid().await?.as_u64();
            if chain_id != expected {
                return Err(anyhow!("unexpected chain id: got {} expected {}", chain_id, expected));
            }
        }
        let from_addr = EthAddress::from_str(req.from().as_str())?;
        let to_addr = EthAddress::from_str(req.to().as_str())?;
        let value = parse_ether(req.amount_eth())?;
        let base = TransactionRequest::new().from(from_addr).to(to_addr).value(value);
        let mut typed: TypedTransaction = base.into();
        let est = self.provider.estimate_gas(&typed, None).await?;
        if est.as_u64() > self.gas_cap {
            return Err(anyhow!("estimated gas {} exceeds cap {}", est, self.gas_cap));
        }
        typed.set_gas(est);
        let _sim = self.provider.call(&typed, None).await?;
        if req.simulate() {
            return Ok(TxResult::new(String::new(), Some(est.as_u64()), None));
        }
        let key = normalize(req.from().as_str());
        let wallet = self.known_wallets.get(&key).cloned().ok_or_else(|| anyhow!("no local key for from address {}", req.from().as_str()))?;
        let chain_id = self.provider.get_chainid().await?.as_u64();
        let wallet = wallet.with_chain_id(chain_id);
        let client = SignerMiddleware::new(self.provider.clone(), wallet);
        let pending = client.send_transaction(typed, None).await?;
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

fn normalize(addr: &str) -> String { addr.trim().to_lowercase() }
