use serde::{Deserialize, Serialize};
use domain::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum BamlFunction {
    Balance(BalanceRequest),
    Code(CodeRequest),
    Erc20Balance(Erc20BalanceRequest),
    Send(SendRequest),
}

impl BamlFunction {
    pub fn name(&self) -> &'static str {
        match self {
            BamlFunction::Balance(_) => "balance",
            BamlFunction::Code(_) => "code",
            BamlFunction::Erc20Balance(_) => "erc20_balance_of",
            BamlFunction::Send(_) => "send",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BamlFunction::Balance(_) => "Get ETH balance of an address or ENS name",
            BamlFunction::Code(_) => "Check if address has deployed code",
            BamlFunction::Erc20Balance(_) => "Get ERC-20 token balance for holder",
            BamlFunction::Send(_) => "Send ETH from one address to another",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BamlTool {
    pub name: String,
    pub description: String,
    pub function: BamlFunction,
}

impl BamlTool {
    pub fn balance(who: String) -> Self {
        Self {
            name: "balance".to_string(),
            description: "Get ETH balance of an address or ENS name".to_string(),
            function: BamlFunction::Balance(BalanceRequest::new(AddressOrEns::from_ens(who))),
        }
    }

    pub fn code(addr: String) -> Self {
        Self {
            name: "code".to_string(),
            description: "Check if address has deployed code".to_string(),
            function: BamlFunction::Code(CodeRequest::new(Address::new(addr))),
        }
    }

    pub fn erc20_balance(token: String, holder: String) -> Self {
        Self {
            name: "erc20_balance_of".to_string(),
            description: "Get ERC-20 token balance for holder".to_string(),
            function: BamlFunction::Erc20Balance(Erc20BalanceRequest::new(
                Address::new(token),
                Address::new(holder),
            )),
        }
    }

    pub fn send(from: String, to: String, amount_eth: String, simulate: Option<bool>) -> Self {
        Self {
            name: "send".to_string(),
            description: "Send ETH from one address to another".to_string(),
            function: BamlFunction::Send(
                SendRequest::builder()
                    .from(Address::new(from))
                    .to(Address::new(to))
                    .amount_eth(amount_eth)
                    .simulate(simulate.unwrap_or(true))
                    .build()
                    .expect("valid send request"),
            ),
        }
    }
}

pub fn available_tools() -> Vec<BamlTool> {
    vec![
        BamlTool::balance("vitalik.eth".to_string()),
        BamlTool::code("0x0000000000000000000000000000000000000000".to_string()),
        BamlTool::erc20_balance(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string(),
        ),
        BamlTool::send(
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string(),
            "0x70997970c51812dc3a010c7d01b50e0d17dc79c8".to_string(),
            "0.1".to_string(),
            Some(true),
        ),
    ]
}
