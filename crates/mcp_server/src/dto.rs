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

