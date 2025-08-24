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
        
        Ok(ChatResponse {
            content: result["content"][0]["text"].as_str().unwrap_or("").to_string(),
            usage: Some(Usage {
                prompt_tokens: result["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: result["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: result["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32 
                    + result["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            }),
        })
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
            r#"{"function": {"type": "GetEthBalance", "who": "vitalik.eth"}}"#.to_string(),
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
        let content = if last.trim().eq_ignore_ascii_case("hello") || last.trim().eq_ignore_ascii_case("hi") {
            self.responses.get("hello").unwrap()
        } else if last.contains("send") {
            self.responses.get("send").unwrap()
        } else if last.contains("code") {
            self.responses.get("code").unwrap()
        } else if last.contains("balance") {
            self.responses.get("balance").unwrap()
        } else {
            self.responses.get("balance").unwrap() // default
        };

        Ok(ChatResponse {
            content: content.clone(),
            usage: None,
        })
    }
}
