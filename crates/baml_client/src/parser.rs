use anyhow::Result;
use tracing::{debug, info};
use crate::baml::{BamlFunction, BamlTool};

pub struct NlParser {
    tools: Vec<BamlTool>,
}

impl NlParser {
    pub fn new() -> Self {
        Self {
            tools: crate::baml::available_tools(),
        }
    }

    pub fn parse_query(&self, query: &str) -> Result<BamlFunction> {
        let query_lower = query.to_lowercase();
        info!("Parsing query: {}", query);

        // Simple keyword-based parsing for now
        // TODO: Replace with actual LLM call to Anthropic
        if query_lower.contains("balance") && query_lower.contains("eth") {
            let who = self.extract_address_or_ens(query)?;
            debug!("Parsed balance request for: {}", who);
            return Ok(BamlFunction::Balance(
                domain::BalanceRequest::new(domain::AddressOrEns::from_ens(who))
            ));
        }

        if query_lower.contains("code") || query_lower.contains("deployed") {
            let addr = self.extract_address(query)?;
            debug!("Parsed code request for: {}", addr);
            return Ok(BamlFunction::Code(
                domain::CodeRequest::new(domain::Address::new(addr))
            ));
        }

        if query_lower.contains("erc20") || query_lower.contains("token") {
            let (token, holder) = self.extract_token_and_holder(query)?;
            debug!("Parsed ERC20 balance request: token={}, holder={}", token, holder);
            return Ok(BamlFunction::Erc20Balance(
                domain::Erc20BalanceRequest::new(
                    domain::Address::new(token),
                    domain::Address::new(holder),
                )
            ));
        }

        if query_lower.contains("send") || query_lower.contains("transfer") {
            let (from, to, amount) = self.extract_send_params(query)?;
            debug!("Parsed send request: {} -> {} ({} ETH)", from, to, amount);
            return Ok(BamlFunction::Send(
                domain::SendRequest::builder()
                    .from(domain::Address::new(from))
                    .to(domain::Address::new(to))
                    .amount_eth(amount)
                    .simulate(true) // Default to simulation
                    .build().map_err(|e| anyhow::anyhow!("{}", e))?
            ));
        }

        anyhow::bail!("Could not parse query: {}", query)
    }

    fn extract_address_or_ens(&self, query: &str) -> Result<String> {
        // Simple regex-like extraction
        if query.contains("vitalik") {
            return Ok("vitalik.eth".to_string());
        }
        if query.contains("0x") {
            return Ok(self.extract_address(query)?);
        }
        anyhow::bail!("No address or ENS found in query")
    }

    fn extract_address(&self, query: &str) -> Result<String> {
        // Extract first 0x-prefixed string
        let words: Vec<&str> = query.split_whitespace().collect();
        for word in words {
            if word.starts_with("0x") && word.len() == 42 {
                return Ok(word.to_string());
            }
        }
        anyhow::bail!("No valid address found in query")
    }

    fn extract_token_and_holder(&self, _query: &str) -> Result<(String, String)> {
        // For now, return USDC and a default holder
        Ok((
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string(), // Anvil account 0
        ))
    }

    fn extract_send_params(&self, _query: &str) -> Result<(String, String, String)> {
        // For now, return default Anvil accounts and 0.1 ETH
        Ok((
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string(), // Anvil account 0
            "0x70997970c51812dc3a010c7d01b50e0d17dc79c8".to_string(), // Anvil account 1
            "0.1".to_string(),
        ))
    }
}
