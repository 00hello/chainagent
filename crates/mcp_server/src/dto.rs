use domain::{
    Address, AddressOrEns, BalanceRequest, CodeRequest, Erc20BalanceRequest, SendRequest, SendRequestBuilder,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BalanceIn {
    pub who: String,
}

impl TryFrom<BalanceIn> for BalanceRequest {
    type Error = anyhow::Error;
    fn try_from(value: BalanceIn) -> Result<Self, Self::Error> {
        let who = if value.who.ends_with(".eth") {
            AddressOrEns::from_ens(value.who)
        } else {
            AddressOrEns::from_address(value.who)
        };
        Ok(BalanceRequest::new(who))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeIn { pub addr: String }

impl TryFrom<CodeIn> for CodeRequest {
    type Error = anyhow::Error;
    fn try_from(value: CodeIn) -> Result<Self, Self::Error> {
        Ok(CodeRequest::new(Address::new(value.addr)))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Erc20BalanceIn { pub token: String, pub holder: String }

impl TryFrom<Erc20BalanceIn> for Erc20BalanceRequest {
    type Error = anyhow::Error;
    fn try_from(value: Erc20BalanceIn) -> Result<Self, Self::Error> {
        Ok(Erc20BalanceRequest::new(Address::new(value.token), Address::new(value.holder)))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendIn {
    pub from: String,
    pub to: String,
    pub amount_eth: String,
    pub simulate: Option<bool>,
    pub fork_block: Option<u64>,
}

impl TryFrom<SendIn> for SendRequest {
    type Error = anyhow::Error;
    fn try_from(value: SendIn) -> Result<Self, Self::Error> {
        let mut b: SendRequestBuilder = SendRequest::builder()
            .from(Address::new(value.from))
            .to(Address::new(value.to))
            .amount_eth(value.amount_eth);
        if let Some(sim) = value.simulate { b = b.simulate(sim); }
        Ok(b.fork_block(value.fork_block).build().map_err(|e| anyhow::anyhow!(e))?)
    }
}

// External API lookup DTOs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenLookupIn {
    pub symbol: String,
    pub chain: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenLookupOut {
    pub address: Option<String>,
    pub symbol: String,
    pub chain: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::AddressOrEns;

    #[test]
    fn balance_in_to_request_handles_ens() {
        let b = BalanceIn { who: "vitalik.eth".into() };
        let br: BalanceRequest = b.try_into().unwrap();
        matches!(br.who(), AddressOrEns::Ens(_));
    }

    #[test]
    fn code_in_roundtrip() {
        let c = CodeIn { addr: "0x0000000000000000000000000000000000000000".into() };
        let cr: CodeRequest = c.try_into().unwrap();
        assert_eq!(cr.addr().as_str(), "0x0000000000000000000000000000000000000000");
    }

    #[test]
    fn erc20_balance_in_roundtrip() {
        let e = Erc20BalanceIn { token: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(), holder: "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".into() };
        let er: Erc20BalanceRequest = e.try_into().unwrap();
        assert!(er.token().as_str().starts_with("0x"));
    }

    #[test]
    fn send_in_defaults_simulate_true_when_missing() {
        let s = SendIn { from: "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".into(), to: "0x70997970c51812dc3a010c7d01b50e0d17dc79c8".into(), amount_eth: "1.0".into(), simulate: None, fork_block: None };
        let sr: SendRequest = s.try_into().unwrap();
        assert!(sr.simulate());
    }
}

