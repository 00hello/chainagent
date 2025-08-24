use clap::Parser;
use tracing::info;

mod baml;
mod mcp;
mod parser;
mod provider;
mod tools;
mod baml_bindings;

use baml::BamlFunction;
use mcp::McpClient;
use parser::NlParser;
use provider::{MockProvider, AnthropicProvider};

#[derive(Parser)]
#[command(name = "baml-client")]
#[command(about = "BAML-driven CLI client for EVM toolbox")]
struct Cli {
    /// Natural language query to execute
    #[arg(short, long)]
    query: String,

    /// MCP server URL (default: http://localhost:3000)
    #[arg(short, long, default_value = "http://localhost:3000")]
    server: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Optional session id for conversational memory (stored on MCP server)
    #[arg(long)]
    session: Option<String>,

    /// Use mock provider instead of real LLM
    #[arg(short, long)]
    mock: bool,

    /// Simulate-only; do not broadcast state-changing transactions
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Enable bonus tools (swap, token lookup, RAG); can also set BONUS=1
    #[arg(long, default_value_t = false)]
    enable_bonus: bool,

    /// Enable BAML validation (schema-first). Can also set ENABLE_BAML=1
    #[arg(long, default_value_t = false)]
    enable_baml: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env so CLI runs without exporting variables in the shell
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Bonus flag/env
    let bonus_env = std::env::var("BONUS").ok().map(|v| v == "1").unwrap_or(false);
    let bonus_enabled = cli.enable_bonus || bonus_env;
    if bonus_enabled {
        info!("Bonus features enabled");
    }

    // BAML validation flag/env
    let baml_env = std::env::var("ENABLE_BAML").ok().map(|v| v == "1").unwrap_or(false);
    let baml_enabled = cli.enable_baml || baml_env;
    if baml_enabled {
        info!("BAML validation enabled");
    }

    info!("Processing query: {}", cli.query);
    info!("MCP server: {}", cli.server);

    // 3.0 Optional: load session history
    let mut _history: Vec<provider::ChatMessage> = Vec::new();
    if let Some(session_id) = &cli.session {
        let client = McpClient::new(cli.server.clone());
        if let Ok(h) = client.session_get(session_id).await { _history = h; }
    }

    // 3.1 Parse NL input and choose BAML function
    let function = if cli.mock {
        let provider = MockProvider::new();
        let parser = NlParser::new_with_baml(provider, baml_enabled);
        parser.parse_query(&cli.query).await?
    } else {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY environment variable required");
        let provider = AnthropicProvider::new(api_key);
        let parser = NlParser::new_with_baml(provider, baml_enabled);
        parser.parse_query(&cli.query).await?
    };
    info!("Selected function: {}", function.name());

    // 3.2 Validate via BAML schema (implicit in our type system)
    info!("Function validated: {}", function.description());

    // 3.3 Invoke MCP server
    let client = McpClient::new(cli.server.clone());
    let result = match function {
        BamlFunction::Chat(ref text) => {
            println!("Chat: {}", text);
            serde_json::json!({ "message": text })
        }
        BamlFunction::Balance(ref req) => {
            let balance = client.balance(req).await?;
            serde_json::json!({ "balance": balance })
        }
        BamlFunction::Code(ref req) => {
            let (deployed, bytecode_len) = client.code(req).await?;
            serde_json::json!({
                "deployed": deployed,
                "bytecode_len": bytecode_len
            })
        }
        BamlFunction::Erc20Balance(ref req) => {
            let amount = client.erc20_balance_of(req).await?;
            serde_json::json!({ "amount": amount })
        }
        BamlFunction::Send(ref req) => {
            // Honor --dry-run by forcing simulate=true
            let req_overridden = domain::SendRequest::builder()
                .from(req.from().clone())
                .to(req.to().clone())
                .amount_eth(req.amount_eth().to_string())
                .simulate(cli.dry_run || req.simulate())
                .fork_block(req.fork_block())
                .build()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let tx_result = client.send(&req_overridden).await?;
            serde_json::json!({
                "tx_hash": tx_result.tx_hash(),
                "success": tx_result.status().unwrap_or(false)
            })
        }
    };

    // 3.4 Echo typed call and pretty-print JSON response
    println!("Function: {}", function.name());
    println!("Response: {}", serde_json::to_string_pretty(&result)?);

    // 3.5 Append turns to session if enabled
    if let Some(session_id) = &cli.session {
        let client = McpClient::new(cli.server.clone());
        // Append user input
        let _ = client.session_append(session_id, "user", &cli.query).await;
        // Append assistant/tool reply summary
        let summary = match &function {
            BamlFunction::Chat(text) => text.clone(),
            _ => serde_json::to_string(&result).unwrap_or_default(),
        };
        let _ = client.session_append(session_id, "assistant", &summary).await;
    }

    Ok(())
}

