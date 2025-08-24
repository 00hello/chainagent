use anyhow::Result;
use tracing::{debug, info};
use crate::baml::BamlFunction;
use crate::provider::{ChatProvider, ChatMessage, ChatRequest};
use crate::tools::ToolRegistry;

pub struct NlParser<P: ChatProvider> {
    provider: P,
}

impl<P: ChatProvider> NlParser<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub async fn parse_query_with_history(&self, query: &str, history: &[ChatMessage]) -> Result<BamlFunction> {
        info!("Parsing query with LLM (with history): {}", query);
        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: r#"You are an EVM toolbox agent that can help with blockchain operations or casual conversation.

Available blockchain functions:
- GetNativeBalance: Get native token balance of an address or name
- GetFungibleBalance: Get fungible token balance for a holder address  
- GetCode: Check if an address has deployed code
- SendNative: Send native token from one address to another

For blockchain-related queries, use the appropriate function with the correct parameters.
For addresses, prefer ENS names when available (e.g., "vitalik.eth").
For send operations, default to simulate=true unless explicitly requested to send.

For casual conversation (greetings, general questions), respond naturally without using any tools.
If you use a tool, return a JSON object with the function type and parameters.
If it's casual conversation, just respond normally."#.to_string(),
            },
        ];
        messages.extend_from_slice(history);
        messages.push(ChatMessage { role: "user".to_string(), content: query.to_string() });

        let request = ChatRequest {
            messages,
            model: "claude-sonnet-4-20250514".to_string(),
            temperature: Some(0.0),
            tools: Some(self.native_tools_schema()),
        };

        let response = self.provider.chat(request).await?;
        debug!("LLM response: {}", response.content);

        match self.parse_llm_response(&response.content) {
            Ok(func) => Ok(func),
            Err(_) => Ok(BamlFunction::Chat(response.content)),
        }
    }

    fn native_tools_schema(&self) -> Vec<crate::provider::ToolDef> {
        // Generate from ToolRegistry so tools can be added dynamically
        ToolRegistry::with_default_tools().tool_defs()
    }

    pub async fn parse_query(&self, query: &str) -> Result<BamlFunction> {
        info!("Parsing query with LLM: {}", query);

        // Create BAML agent prompt
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: r#"You are an EVM toolbox agent that can help with blockchain operations or casual conversation.

Available blockchain functions:
- GetNativeBalance: Get native token balance of an address or name
- GetFungibleBalance: Get fungible token balance for a holder address  
- GetCode: Check if an address has deployed code
- SendNative: Send native token from one address to another

For blockchain-related queries, use the appropriate function with the correct parameters.
For addresses, prefer ENS names when available (e.g., "vitalik.eth").
For send operations, default to simulate=true unless explicitly requested to send.

For casual conversation (greetings, general questions), respond naturally without using any tools.
If you use a tool, return a JSON object with the function type and parameters.
If it's casual conversation, just respond normally."#.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: query.to_string(),
            },
        ];

        let request = ChatRequest {
            messages,
            model: "claude-sonnet-4-20250514".to_string(),
            temperature: Some(0.0),
            // Pass native tool schemas so the LLM can select tools or decline
            tools: Some(self.native_tools_schema()),
        };

        let response = self.provider.chat(request).await?;
        debug!("LLM response: {}", response.content);

        // Parse the response. If it's not a tool call JSON, treat it as plain chat.
        match self.parse_llm_response(&response.content) {
            Ok(func) => Ok(func),
            Err(_) => Ok(BamlFunction::Chat(response.content)),
        }
    }

    fn parse_llm_response(&self, response: &str) -> Result<BamlFunction> {
        // Try to parse as JSON first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(function) = json.get("function") {
                return self.parse_function_json(function);
            }
        }

        // No tool JSON found → treat as plain chat
        anyhow::bail!("No tool JSON found")
    }

    fn parse_function_json(&self, function: &serde_json::Value) -> Result<BamlFunction> {
        let function_type = function.get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing function type"))?;

        // Prefer BAML bindings (schema-first) to validate and map
        if let Ok(mapped) = crate::baml_bindings::validate_and_to_baml_function(function_type, function) {
            return Ok(mapped);
        }

        match function_type {
            // New chain-neutral name
            "GetNativeBalance" | "GetEthBalance" => {
                let who_opt = function.get("who").and_then(|w| w.as_str());
                if who_opt.is_none() {
                    let msg = format!("I need 'who' to get a balance. Please provide an address or ENS/name.\n[[PARTIAL_INTENT]]\n{}\n[[/PARTIAL_INTENT]]", function.to_string());
                    return Ok(BamlFunction::Chat(msg));
                }
                let who = who_opt.unwrap();
                let input = who.to_string();
                let addr_or_ens = if input.ends_with(".eth") {
                    domain::AddressOrEns::from_ens(input)
                } else {
                    domain::AddressOrEns::from_address(input)
                };
                Ok(BamlFunction::Balance(
                    domain::BalanceRequest::new(addr_or_ens)
                ))
            }
            // New chain-neutral name
            "GetCode" | "IsDeployed" => {
                let addr_opt = function.get("addr").and_then(|a| a.as_str());
                if addr_opt.is_none() {
                    let msg = format!("I need 'addr' to check code. Please provide an address.\n[[PARTIAL_INTENT]]\n{}\n[[/PARTIAL_INTENT]]", function.to_string());
                    return Ok(BamlFunction::Chat(msg));
                }
                let addr = addr_opt.unwrap();
                Ok(BamlFunction::Code(
                    domain::CodeRequest::new(domain::Address::new(addr.to_string()))
                ))
            }
            // New chain-neutral name
            "GetFungibleBalance" | "GetErc20Balance" => {
                let token_opt = function.get("token").and_then(|t| t.as_str());
                let holder_opt = function.get("holder").and_then(|h| h.as_str());
                if token_opt.is_none() || holder_opt.is_none() {
                    let msg = format!("I need 'token' and 'holder' to get token balance. Please provide both.\n[[PARTIAL_INTENT]]\n{}\n[[/PARTIAL_INTENT]]", function.to_string());
                    return Ok(BamlFunction::Chat(msg));
                }
                let token = token_opt.unwrap();
                let holder = holder_opt.unwrap();
                Ok(BamlFunction::Erc20Balance(
                    domain::Erc20BalanceRequest::new(
                        domain::Address::new(token.to_string()),
                        domain::Address::new(holder.to_string()),
                    )
                ))
            }
            // New chain-neutral name
            "SendNative" | "SendEth" => {
                let from_opt = function.get("from").and_then(|f| f.as_str());
                let to_opt = function.get("to").and_then(|t| t.as_str());
                let amount_opt = function.get("amount_eth").and_then(|a| a.as_str());
                if from_opt.is_none() || to_opt.is_none() || amount_opt.is_none() {
                    let msg = format!("I need 'from', 'to', and 'amount_eth' to send. Please provide missing fields.\n[[PARTIAL_INTENT]]\n{}\n[[/PARTIAL_INTENT]]", function.to_string());
                    return Ok(BamlFunction::Chat(msg));
                }
                let from = from_opt.unwrap();
                let to = to_opt.unwrap();
                let amount_eth = amount_opt.unwrap();
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
            let addr_or_ens = if who.ends_with(".eth") {
                domain::AddressOrEns::from_ens(who)
            } else {
                domain::AddressOrEns::from_address(who)
            };
            return Ok(BamlFunction::Balance(
                domain::BalanceRequest::new(addr_or_ens)
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

        // New: Balance tool selection by 0x address
        let function = parser.parse_query("What's 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266's balance?").await.unwrap();
        assert!(matches!(function, BamlFunction::Balance(_)));
        if let BamlFunction::Balance(req) = function {
            match req.who() {
                domain::AddressOrEns::Address(addr) => assert_eq!(addr.as_str(), "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"),
                _ => panic!("Expected 0x address"),
            }
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

        // Small talk should return Chat
        let function = parser.parse_query("hello").await.unwrap();
        assert!(matches!(function, BamlFunction::Chat(_)));
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
