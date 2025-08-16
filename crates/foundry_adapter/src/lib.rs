use anyhow::{anyhow, Result};
use ethers_core::types::H160;
use domain::{Address, AddressOrEns, BalanceRequest, CodeRequest, Erc20BalanceRequest, SendRequest, TxResult};

pub fn is_checksum_address(_addr: &str) -> bool {
    // TODO: implement EIP-55 validation
    true
}

pub fn parse_h160(_addr: &str) -> Result<H160> {
    // TODO: implement strict parsing
    Err(anyhow!("not implemented"))
}

#[derive(Clone, Debug)]
pub struct FoundryAdapter {
    rpc_url: String,
    gas_cap: u64,
    expected_chain_id: Option<u64>,
}

impl FoundryAdapter {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self { rpc_url: rpc_url.into(), gas_cap: 500_000, expected_chain_id: None }
    }

    pub fn with_expected_chain_id(mut self, chain_id: u64) -> Self {
        self.expected_chain_id = Some(chain_id);
        self
    }

    pub fn with_gas_cap(mut self, gas_cap: u64) -> Self {
        self.gas_cap = gas_cap;
        self
    }

    pub async fn resolve_address_or_ens(&self, _input: &AddressOrEns) -> Result<Address> {
        Err(anyhow!("resolve_address_or_ens: not implemented"))
    }

    pub async fn get_balance(&self, _req: &BalanceRequest) -> Result<String> {
        Err(anyhow!("get_balance: not implemented"))
    }

    pub async fn get_code_len(&self, _req: &CodeRequest) -> Result<(bool, u64)> {
        Err(anyhow!("get_code_len: not implemented"))
    }

    pub async fn erc20_balance_of(&self, _req: &Erc20BalanceRequest) -> Result<String> {
        Err(anyhow!("erc20_balance_of: not implemented"))
    }

    pub async fn send_eth(&self, _req: &SendRequest) -> Result<TxResult> {
        Err(anyhow!("send_eth: not implemented"))
    }
}
pub fn placeholder_adapter() {}

