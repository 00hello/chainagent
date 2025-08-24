use serde::{Deserialize, Serialize};
use domain::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum BamlFunction {
    /// Plain chat response from the LLM; no tool to call
    Chat(String),
    Balance(BalanceRequest),
    Code(CodeRequest),
    Erc20Balance(Erc20BalanceRequest),
    Send(SendRequest),
}

impl BamlFunction {
    pub fn name(&self) -> &'static str {
        match self {
            BamlFunction::Chat(_) => "chat",
            BamlFunction::Balance(_) => "balance",
            BamlFunction::Code(_) => "code",
            BamlFunction::Erc20Balance(_) => "erc20_balance_of",
            BamlFunction::Send(_) => "send",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BamlFunction::Chat(_) => "Plain chat response",
            BamlFunction::Balance(_) => "Get ETH balance of an address or ENS name",
            BamlFunction::Code(_) => "Check if address has deployed code",
            BamlFunction::Erc20Balance(_) => "Get ERC-20 token balance for holder",
            BamlFunction::Send(_) => "Send ETH from one address to another",
        }
    }
}

// Removed legacy BamlTool infrastructure in favor of ToolRegistry
