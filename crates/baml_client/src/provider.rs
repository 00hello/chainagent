use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub temperature: Option<f32>,
    /// Optional: native tool registration (Claude/OpenAI-style)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDef>>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Minimal tool definition for native tool registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    /// JSON schema for inputs
    pub input_schema: serde_json::Value,
}

#[async_trait]
pub trait ChatProvider: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
}

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ChatProvider for AnthropicProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Separate system message from user messages
        let (system_message, user_messages): (Option<String>, Vec<ChatMessage>) = {
            let mut system = None;
            let mut users = Vec::new();
            
            for msg in request.messages {
                if msg.role == "system" {
                    system = Some(msg.content);
                } else {
                    users.push(msg);
                }
            }
            (system, users)
        };

        let mut body = serde_json::json!({
            "model": request.model,
            "max_tokens": 1000,
            "messages": user_messages,
            "temperature": request.temperature.unwrap_or(0.0),
            // Native tool registration (if provided)
            "tools": request.tools.unwrap_or_default(),
        });

        // Add system message as separate parameter if present
        if let Some(system) = system_message {
            body["system"] = serde_json::Value::String(system);
        }

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        
        // Prefer native tool_use blocks if present and convert them into the
        // function JSON our parser already understands: { "function": { "type": name, ...input } }
        if let Some(content_blocks) = result.get("content").and_then(|c| c.as_array()) {
            if let Some(tool_block) = content_blocks.iter().find(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use")) {
                let name = tool_block.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let input = tool_block.get("input").cloned().unwrap_or(serde_json::json!({}));
                let function_json = serde_json::json!({
                    "function": {
                        "type": name,
                        // Merge input fields directly under function
                    }
                });
                // Manually merge input object into function_json["function"]
                let mut function_obj = function_json["function"].as_object().cloned().unwrap_or_default();
                if let Some(map) = input.as_object() {
                    for (k, v) in map.iter() { function_obj.insert(k.clone(), v.clone()); }
                }
                let final_json = serde_json::json!({ "function": serde_json::Value::Object(function_obj) });

                return Ok(ChatResponse {
                    content: final_json.to_string(),
                    usage: Some(Usage {
                        prompt_tokens: result["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
                        completion_tokens: result["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
                        total_tokens: result["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32
                            + result["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
                    }),
                });
            }
        }

        // Fallback: treat first text block as plain chat
        let text = result["content"][0]["text"].as_str().unwrap_or("").to_string();
        Ok(ChatResponse {
            content: text,
            usage: Some(Usage {
                prompt_tokens: result["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: result["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: result["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32
                    + result["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            }),
        })
    }
}

pub struct OpenAIProvider {
    api_key: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key, client: reqwest::Client::new() }
    }
}

#[async_trait]
impl ChatProvider for OpenAIProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Map ToolDef → OpenAI tool schema
        let oai_tools = request.tools.unwrap_or_default().into_iter().map(|t| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.input_schema
                }
            })
        }).collect::<Vec<_>>();

        let body = serde_json::json!({
            "model": request.model,
            "temperature": request.temperature.unwrap_or(0.0),
            "messages": request.messages,
            "tools": oai_tools,
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        // If tool call present, normalize to our JSON shape
        if let Some(choice) = result["choices"].as_array().and_then(|arr| arr.first()) {
            let msg = &choice["message"];
            if let Some(tool_calls) = msg["tool_calls"].as_array() {
                if let Some(tc) = tool_calls.first() {
                    let name = tc["function"]["name"].as_str().unwrap_or("");
                    let args = tc["function"]["arguments"].as_str().unwrap_or("{}");
                    let parsed_args: serde_json::Value = serde_json::from_str(args).unwrap_or(serde_json::json!({}));
                    let mut function_obj = serde_json::Map::new();
                    function_obj.insert("type".to_string(), serde_json::Value::String(name.to_string()));
                    if let Some(map) = parsed_args.as_object() {
                        for (k, v) in map.iter() { function_obj.insert(k.clone(), v.clone()); }
                    }
                    let final_json = serde_json::json!({ "function": serde_json::Value::Object(function_obj) });
                    return Ok(ChatResponse { content: final_json.to_string(), usage: None });
                }
            }
            // Fallback: plain content
            let text = msg["content"].as_str().unwrap_or("").to_string();
            return Ok(ChatResponse { content: text, usage: None });
        }

        // Ultimate fallback
        Ok(ChatResponse { content: "".to_string(), usage: None })
    }
}
pub struct MockProvider {
    responses: std::collections::HashMap<String, String>,
}

impl MockProvider {
    pub fn new() -> Self {
        let mut responses = std::collections::HashMap::new();
        responses.insert(
            "balance".to_string(),
            r#"{"function": {"type": "GetNativeBalance", "who": "vitalik.eth"}}"#.to_string(),
        );
        responses.insert(
            "hello".to_string(),
            r#"{"message": "hello"}"#.to_string(),
        );
        responses.insert(
            "code".to_string(),
            r#"{"function": {"type": "IsDeployed", "addr": "0x0000000000000000000000000000000000000000"}}"#.to_string(),
        );
        responses.insert(
            "send".to_string(),
            r#"{"function": {"type": "SendEth", "from": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266", "to": "0x70997970c51812dc3a010c7d01b50e0d17dc79c8", "amount_eth": "0.1", "simulate": true}}"#.to_string(),
        );
        
        Self { responses }
    }
}

#[async_trait]
impl ChatProvider for MockProvider {
    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        // Simple keyword-based response for testing
        let last = &_request.messages.last().unwrap().content;

        // First check for exact small talk matches
        if last.trim().eq_ignore_ascii_case("hello") || last.trim().eq_ignore_ascii_case("hi") {
            let content = self.responses.get("hello").unwrap();
            return Ok(ChatResponse {
                content: content.clone(),
                usage: None,
            });
        }

        // Prefer code/deployed queries first → IsDeployed
        let lower = last.to_lowercase();
        if lower.contains("code") || lower.contains("deployed") {
            if let Some(addr) = extract_first_address(last) {
                let json = serde_json::json!({
                    "function": { "type": "IsDeployed", "addr": addr }
                });
                return Ok(ChatResponse { content: json.to_string(), usage: None });
            }
        }

        // If the query includes a 0x address, default to GetNativeBalance for testing
        if let Some(addr) = extract_first_address(last) {
            let json = serde_json::json!({
                "function": { "type": "GetNativeBalance", "who": addr }
            });
            return Ok(ChatResponse { content: json.to_string(), usage: None });
        }

        // If the query includes an ENS-like token, pass it as who (balance)
        if let Some(ens) = extract_first_ens(last) {
            let json = serde_json::json!({
                "function": { "type": "GetNativeBalance", "who": ens }
            });
            return Ok(ChatResponse { content: json.to_string(), usage: None });
        }

        // Fallback keyword matching
        let content = if last.contains("send") {
            self.responses.get("send").unwrap()
        } else if last.contains("code") {
            self.responses.get("code").unwrap()
        } else if last.contains("balance") {
            self.responses.get("balance").unwrap()
        } else {
            // For unknown queries, return a simple chat response instead of defaulting to balance
            "Hello! I'm here to help with blockchain operations. You can ask me to check balances, send ETH, or check if addresses have deployed code."
        };

        Ok(ChatResponse {
            content: content.to_string(),
            usage: None,
        })
    }
}

// Helpers for MockProvider only
fn extract_first_address(text: &str) -> Option<String> {
    for word in text.split_whitespace() {
        let trimmed = word.trim_matches(|c: char|
            c == '?' || c == '"' || c == '\'' || c == ',' || c == '.' || c == ')' || c == '(' || c == ':' || c == ';'
        );
        let mut candidate = trimmed.to_string();
        if candidate.ends_with("'s") || candidate.ends_with("’s") {
            candidate.truncate(candidate.len().saturating_sub(2));
        }
        if candidate.starts_with("0x") && candidate.len() == 42 { return Some(candidate); }
    }
    None
}

fn extract_first_ens(text: &str) -> Option<String> {
    for word in text.split_whitespace() {
        let trimmed = word.trim_matches(|c: char|
            c == '?' || c == '"' || c == '\'' || c == ',' || c == '.' || c == ')' || c == '(' || c == ':' || c == ';'
        );
        let mut candidate = trimmed.to_string();
        if candidate.ends_with("'s") || candidate.ends_with("’s") {
            candidate.truncate(candidate.len().saturating_sub(2));
        }
        if candidate.ends_with(".eth") { return Some(candidate); }
    }
    None
}
