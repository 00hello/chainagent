use anyhow::Result;
use crate::baml::BamlFunction;

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> serde_json::Value;
    fn to_baml_function(&self, input: &serde_json::Value) -> Result<BamlFunction>;
}

pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,    
}

impl ToolRegistry {
    pub fn new() -> Self { Self { tools: Vec::new() } }

    pub fn with_default_tools() -> Self {
        let mut r = Self::new();
        // Chain-neutral names; backward-compat handled in parser
        r.register(GetNativeBalanceTool);
        r.register(GetCodeTool);
        r.register(GetFungibleBalanceTool);
        r.register(SendNativeTool);
        r
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.push(Box::new(tool));
    }

    pub fn tool_defs(&self) -> Vec<crate::provider::ToolDef> {
        self.tools.iter().map(|t| crate::provider::ToolDef {
            name: t.name().to_string(),
            description: t.description().to_string(),
            input_schema: t.input_schema(),
        }).collect()
    }

    pub fn to_baml_function(&self, tool_name: &str, input: &serde_json::Value) -> Result<BamlFunction> {
        // Find by exact name
        if let Some(tool) = self.tools.iter().find(|t| t.name() == tool_name) {
            return tool.to_baml_function(input);
        }
        anyhow::bail!("Unknown tool: {}", tool_name)
    }
}

struct GetNativeBalanceTool;
impl Tool for GetNativeBalanceTool {
    fn name(&self) -> &'static str { "GetNativeBalance" }
    fn description(&self) -> &'static str { "Get native token balance of an address or name" }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "who": {"type": "string"} },
            "required": ["who"],
        })
    }
    fn to_baml_function(&self, input: &serde_json::Value) -> Result<BamlFunction> {
        let who = input.get("who").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'who' parameter"))?;
        let who_s = who.to_string();
        let addr_or_ens = if who_s.ends_with(".eth") { domain::AddressOrEns::from_ens(who_s) } else { domain::AddressOrEns::from_address(who_s) };
        Ok(BamlFunction::Balance(domain::BalanceRequest::new(addr_or_ens)))
    }
}

struct GetCodeTool;
impl Tool for GetCodeTool {
    fn name(&self) -> &'static str { "GetCode" }
    fn description(&self) -> &'static str { "Get code/deployment status for an address" }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "addr": {"type": "string"} },
            "required": ["addr"],
        })
    }
    fn to_baml_function(&self, input: &serde_json::Value) -> Result<BamlFunction> {
        let addr = input.get("addr").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'addr' parameter"))?;
        Ok(BamlFunction::Code(domain::CodeRequest::new(domain::Address::new(addr.to_string()))))
    }
}

struct GetFungibleBalanceTool;
impl Tool for GetFungibleBalanceTool {
    fn name(&self) -> &'static str { "GetFungibleBalance" }
    fn description(&self) -> &'static str { "Get fungible token balance for holder" }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "token": {"type": "string"}, "holder": {"type": "string"} },
            "required": ["token", "holder"],
        })
    }
    fn to_baml_function(&self, input: &serde_json::Value) -> Result<BamlFunction> {
        let token = input.get("token").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'token' parameter"))?;
        let holder = input.get("holder").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'holder' parameter"))?;
        Ok(BamlFunction::Erc20Balance(domain::Erc20BalanceRequest::new(
            domain::Address::new(token.to_string()),
            domain::Address::new(holder.to_string()),
        )))
    }
}

struct SendNativeTool;
impl Tool for SendNativeTool {
    fn name(&self) -> &'static str { "SendNative" }
    fn description(&self) -> &'static str { "Send native token from one address to another" }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "from": {"type": "string"},
                "to": {"type": "string"},
                "amount_eth": {"type": "string"},
                "simulate": {"type": "boolean"}
            },
            "required": ["from", "to", "amount_eth"],
        })
    }
    fn to_baml_function(&self, input: &serde_json::Value) -> Result<BamlFunction> {
        let from = input.get("from").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'from' parameter"))?;
        let to = input.get("to").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'to' parameter"))?;
        let amount_eth = input.get("amount_eth").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'amount_eth' parameter"))?;
        let simulate = input.get("simulate").and_then(|v| v.as_bool()).unwrap_or(true);
        Ok(BamlFunction::Send(
            domain::SendRequest::builder()
                .from(domain::Address::new(from.to_string()))
                .to(domain::Address::new(to.to_string()))
                .amount_eth(amount_eth.to_string())
                .simulate(simulate)
                .build().map_err(|e| anyhow::anyhow!("{}", e))?
        ))
    }
}


