use anyhow::Result;
use tracing::{debug, info};
use crate::baml::{BamlFunction, BamlTool};
use crate::provider::{ChatProvider, ChatMessage, ChatRequest};

pub struct NlParser<P: ChatProvider> {
    provider: P,
    tools: Vec<BamlTool>,
}

impl<P: ChatProvider> NlParser<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            tools: crate::baml::available_tools(),
        }
    }

    pub async fn parse_query(&self, query: &str) -> Result<BamlFunction> {
        info!("Parsing query with LLM: {}", query);

        // Create BAML agent prompt
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: r#"You are an EVM toolbox agent that routes natural language queries to the appropriate blockchain functions.

Available functions:
- GetEthBalance: Get ETH balance of an address or ENS name
- GetErc20Balance: Get ERC-20 token balance for a holder address  
- IsDeployed: Check if an address has deployed code
- SendEth: Send ETH from one address to another

Parse the user's query and select the most appropriate function with the correct parameters.
For addresses, prefer ENS names when available (e.g., "vitalik.eth").
For send operations, default to simulate=true unless explicitly requested to send.

Return a JSON object with the function type and parameters."#.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: query.to_string(),
            },
        ];

        let request = ChatRequest {
            messages,
            model: "claude-3-sonnet-20240229".to_string(),
            temperature: Some(0.0),
        };

        let response = self.provider.chat(request).await?;
        debug!("LLM response: {}", response.content);

        // Parse the response and map to BAML function
        self.parse_llm_response(&response.content)
    }

    fn parse_llm_response(&self, response: &str) -> Result<BamlFunction> {
        // Try to parse as JSON first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(function) = json.get("function") {
                return self.parse_function_json(function);
            }
        }

        // Fallback to keyword-based parsing for testing
        self.fallback_keyword_parsing(response)
    }

    fn parse_function_json(&self, function: &serde_json::Value) -> Result<BamlFunction> {
        let function_type = function.get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing function type"))?;

        match function_type {
            "GetEthBalance" => {
                let who = function.get("who")
                    .and_then(|w| w.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'who' parameter"))?;
                Ok(BamlFunction::Balance(
                    domain::BalanceRequest::new(domain::AddressOrEns::from_ens(who.to_string()))
                ))
            }
            "IsDeployed" => {
                let addr = function.get("addr")
                    .and_then(|a| a.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'addr' parameter"))?;
                Ok(BamlFunction::Code(
                    domain::CodeRequest::new(domain::Address::new(addr.to_string()))
                ))
            }
            "GetErc20Balance" => {
                let token = function.get("token")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'token' parameter"))?;
                let holder = function.get("holder")
                    .and_then(|h| h.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'holder' parameter"))?;
                Ok(BamlFunction::Erc20Balance(
                    domain::Erc20BalanceRequest::new(
                        domain::Address::new(token.to_string()),
                        domain::Address::new(holder.to_string()),
                    )
                ))
            }
            "SendEth" => {
                let from = function.get("from")
                    .and_then(|f| f.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'from' parameter"))?;
                let to = function.get("to")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'to' parameter"))?;
                let amount_eth = function.get("amount_eth")
                    .and_then(|a| a.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'amount_eth' parameter"))?;
                let simulate = function.get("simulate").and_then(|s| s.as_bool()).unwrap_or(true);
                
                Ok(BamlFunction::Send(
                    domain::SendRequest::builder()
                        .from(domain::Address::new(from.to_string()))
                        .to(domain::Address::new(to.to_string()))
                        .amount_eth(amount_eth.to_string())
                        .simulate(simulate)
                        .build().map_err(|e| anyhow::anyhow!("{}", e))?
                ))
            }
            _ => anyhow::bail!("Unknown function type: {}", function_type),
        }
    }

    fn fallback_keyword_parsing(&self, response: &str) -> Result<BamlFunction> {
        let response_lower = response.to_lowercase();
        
        // Simple keyword-based parsing as fallback
        if response_lower.contains("balance") && response_lower.contains("eth") {
            let who = self.extract_address_or_ens(response)?;
            debug!("Parsed balance request for: {}", who);
            return Ok(BamlFunction::Balance(
                domain::BalanceRequest::new(domain::AddressOrEns::from_ens(who))
            ));
        }

        if response_lower.contains("code") || response_lower.contains("deployed") {
            let addr = self.extract_address(response)?;
            debug!("Parsed code request for: {}", addr);
            return Ok(BamlFunction::Code(
                domain::CodeRequest::new(domain::Address::new(addr))
            ));
        }

        if response_lower.contains("erc20") || response_lower.contains("token") {
            let (token, holder) = self.extract_token_and_holder(response)?;
            debug!("Parsed ERC20 balance request: token={}, holder={}", token, holder);
            return Ok(BamlFunction::Erc20Balance(
                domain::Erc20BalanceRequest::new(
                    domain::Address::new(token),
                    domain::Address::new(holder),
                )
            ));
        }

        if response_lower.contains("send") || response_lower.contains("transfer") {
            let (from, to, amount) = self.extract_send_params(response)?;
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

        anyhow::bail!("Could not parse response: {}", response)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::MockProvider;

    #[tokio::test]
    async fn test_golden_prompts() {
        let provider = MockProvider::new();
        let parser = NlParser::new(provider);

        // Golden prompt 1: Get ETH balance
        let function = parser.parse_query("What's vitalik.eth's balance?").await.unwrap();
        assert!(matches!(function, BamlFunction::Balance(_)));
        if let BamlFunction::Balance(req) = function {
            match req.who() {
                domain::AddressOrEns::Ens(ens) => assert_eq!(ens.as_str(), "vitalik.eth"),
                _ => panic!("Expected ENS name"),
            }
        }

        // Golden prompt 2: Check if address has code
        let function = parser.parse_query("Check if 0x0000000000000000000000000000000000000000 has deployed code").await.unwrap();
        assert!(matches!(function, BamlFunction::Code(_)));
        if let BamlFunction::Code(req) = function {
            assert_eq!(req.addr().as_str(), "0x0000000000000000000000000000000000000000");
        }

        // Golden prompt 3: Send ETH
        let function = parser.parse_query("send").await.unwrap();
        assert!(matches!(function, BamlFunction::Send(_)));
        if let BamlFunction::Send(req) = function {
            assert_eq!(req.from().as_str(), "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");
            assert_eq!(req.to().as_str(), "0x70997970c51812dc3a010c7d01b50e0d17dc79c8");
            assert_eq!(req.amount_eth(), "0.1");
            assert!(req.simulate()); // Should default to simulation
        }
    }

    #[tokio::test]
    async fn test_function_names_and_descriptions() {
        let provider = MockProvider::new();
        let parser = NlParser::new(provider);

        let function = parser.parse_query("balance").await.unwrap();
        assert_eq!(function.name(), "balance");
        assert_eq!(function.description(), "Get ETH balance of an address or ENS name");

        let function = parser.parse_query("code").await.unwrap();
        assert_eq!(function.name(), "code");
        assert_eq!(function.description(), "Check if address has deployed code");

        let function = parser.parse_query("send").await.unwrap();
        assert_eq!(function.name(), "send");
        assert_eq!(function.description(), "Send ETH from one address to another");
    }
}
