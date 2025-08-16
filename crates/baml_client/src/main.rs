use clap::Parser;
use tracing::info;

mod baml;
mod mcp;
mod parser;

use baml::BamlFunction;
use mcp::McpClient;
use parser::NlParser;

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.debug {
        tracing_subscriber::fmt::init();
    } else {
        tracing_subscriber::fmt::init();
    }

    info!("Processing query: {}", cli.query);
    info!("MCP server: {}", cli.server);

    // 3.1 Parse NL input and choose BAML function
    let parser = NlParser::new();
    let function = parser.parse_query(&cli.query)?;
    info!("Selected function: {}", function.name());

    // 3.2 Validate via BAML schema (implicit in our type system)
    info!("Function validated: {}", function.description());

    // 3.3 Invoke MCP server
    let client = McpClient::new(cli.server);
    let result = match function {
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
            let tx_result = client.send(req).await?;
            serde_json::json!({
                "tx_hash": tx_result.tx_hash(),
                "success": tx_result.status().unwrap_or(false)
            })
        }
    };

    // 3.4 Echo typed call and pretty-print JSON response
    println!("Function: {}", function.name());
    println!("Response: {}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

