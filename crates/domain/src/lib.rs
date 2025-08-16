use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Represents an Ethereum address as a checksummed string.
/// Fields are private; use constructors and getters.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Address {
    value: String,
}

impl Address {
    pub fn new(value: String) -> Self {
        // Validation is deferred to adapter layer; keep domain minimal and testable.
        Self { value }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// ENS name wrapper
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EnsName {
    value: String,
}

impl EnsName {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Input that can be an address or an ENS name.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AddressOrEns {
    Address(Address),
    Ens(EnsName),
}

impl AddressOrEns {
    pub fn from_address(address: String) -> Self { Self::Address(Address::new(address)) }
    pub fn from_ens(name: String) -> Self { Self::Ens(EnsName::new(name)) }
}

/// Request/Response types for tools

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BalanceRequest {
    who: AddressOrEns,
}

impl BalanceRequest {
    pub fn new(who: AddressOrEns) -> Self { Self { who } }
    pub fn who(&self) -> &AddressOrEns { &self.who }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BalanceResponse {
    wei: String,
}

impl BalanceResponse {
    pub fn new(wei: String) -> Self { Self { wei } }
    pub fn wei(&self) -> &str { &self.wei }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodeRequest {
    addr: Address,
}

impl CodeRequest {
    pub fn new(addr: Address) -> Self { Self { addr } }
    pub fn addr(&self) -> &Address { &self.addr }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodeResponse {
    deployed: bool,
    bytecode_len: u64,
}

impl CodeResponse {
    pub fn new(deployed: bool, bytecode_len: u64) -> Self { Self { deployed, bytecode_len } }
    pub fn deployed(&self) -> bool { self.deployed }
    pub fn bytecode_len(&self) -> u64 { self.bytecode_len }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Erc20BalanceRequest {
    token: Address,
    holder: Address,
}

impl Erc20BalanceRequest {
    pub fn new(token: Address, holder: Address) -> Self { Self { token, holder } }
    pub fn token(&self) -> &Address { &self.token }
    pub fn holder(&self) -> &Address { &self.holder }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Erc20BalanceResponse {
    amount: String,
}

impl Erc20BalanceResponse {
    pub fn new(amount: String) -> Self { Self { amount } }
    pub fn amount(&self) -> &str { &self.amount }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SendRequest {
    from: Address,
    to: Address,
    amount_eth: String,
    simulate: bool,
    fork_block: Option<u64>,
}

impl SendRequest {
    pub fn builder() -> SendRequestBuilder { SendRequestBuilder::default() }
    pub fn from(&self) -> &Address { &self.from }
    pub fn to(&self) -> &Address { &self.to }
    pub fn amount_eth(&self) -> &str { &self.amount_eth }
    pub fn simulate(&self) -> bool { self.simulate }
    pub fn fork_block(&self) -> Option<u64> { self.fork_block }
}

#[derive(Default)]
pub struct SendRequestBuilder {
    from: Option<Address>,
    to: Option<Address>,
    amount_eth: Option<String>,
    simulate: Option<bool>,
    fork_block: Option<u64>,
}

impl SendRequestBuilder {
    pub fn from(mut self, from: Address) -> Self { self.from = Some(from); self }
    pub fn to(mut self, to: Address) -> Self { self.to = Some(to); self }
    pub fn amount_eth(mut self, amount_eth: impl Into<String>) -> Self { self.amount_eth = Some(amount_eth.into()); self }
    pub fn simulate(mut self, simulate: bool) -> Self { self.simulate = Some(simulate); self }
    pub fn fork_block(mut self, fork_block: Option<u64>) -> Self { self.fork_block = fork_block; self }
    pub fn build(self) -> Result<SendRequest, &'static str> {
        Ok(SendRequest {
            from: self.from.ok_or("from required")?,
            to: self.to.ok_or("to required")?,
            amount_eth: self.amount_eth.ok_or("amount_eth required")?,
            simulate: self.simulate.unwrap_or(true),
            fork_block: self.fork_block,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TxResult {
    tx_hash: String,
    gas_used: Option<u64>,
    status: Option<bool>,
}

impl TxResult {
    pub fn new(tx_hash: String, gas_used: Option<u64>, status: Option<bool>) -> Self {
        Self { tx_hash, gas_used, status }
    }
    pub fn tx_hash(&self) -> &str { &self.tx_hash }
    pub fn gas_used(&self) -> Option<u64> { self.gas_used }
    pub fn status(&self) -> Option<bool> { self.status }
}

#[async_trait]
pub trait Toolbox: Send + Sync {
    async fn balance(&self, req: BalanceRequest) -> anyhow::Result<BalanceResponse>;
    async fn code(&self, req: CodeRequest) -> anyhow::Result<CodeResponse>;
    async fn erc20_balance_of(&self, req: Erc20BalanceRequest) -> anyhow::Result<Erc20BalanceResponse>;
    async fn send(&self, req: SendRequest) -> anyhow::Result<TxResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_request_builder_defaults_to_simulate_true() {
        let req = SendRequest::builder()
            .from(Address::new("0xAlice".into()))
            .to(Address::new("0xBob".into()))
            .amount_eth("1.0")
            .build()
            .unwrap();
        assert!(req.simulate());
    }
}
pub fn placeholder_domain() {}

