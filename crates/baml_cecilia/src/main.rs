use anyhow::Result;
use clap::Parser;
use tracing::{info, error};
use baml_client_cecilia::{
    apis::{configuration::Configuration, default_api as baml_api},
    models::{
        GetCodeRequest, GetNativeBalanceRequest, GetFungibleBalanceRequest,
        NativeBalance, FungibleBalance, Check
    }
};

#[derive(Parser)]
#[command(name = "baml-cecilia")]
#[command(about = "Proper BAML implementation for EVM blockchain operations")]
struct Args {
    /// Natural language query to execute
    #[arg(short, long)]
    query: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Address or ENS name for balance queries
    #[arg(short, long)]
    address: Option<String>,

    /// Token contract address for ERC-20 balance queries
    #[arg(short, long)]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    let _ = dotenvy::dotenv();
    
    let args = Args::parse();

    // Initialize logging
    if args.debug {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    } else {
        tracing_subscriber::fmt().init();
    }

    info!("Starting BAML Cecilia client");
    info!("Processing query: {}", args.query);

    // Initialize BAML configuration
    // The Configuration::default() uses environment variables for API keys
    let config = Configuration::default();
    
    // Demonstrate proper BAML usage with different function calls
    // This shows how to properly initialize and use BAML-generated clients
    
    match args.query.to_lowercase().as_str() {
        query if query.contains("balance") && query.contains("eth") => {
            handle_native_balance(&config, args.address).await?;
        },
        query if query.contains("balance") && (query.contains("token") || query.contains("erc20")) => {
            handle_fungible_balance(&config, args.address, args.token).await?;
        },
        query if query.contains("code") || query.contains("deployed") => {
            handle_code_check(&config, args.address).await?;
        },
        _ => {
            info!("Query doesn't match specific patterns, trying general processing");
            // For more complex queries, you would use BAML's NL parsing functions
            println!("Query processed: {}", args.query);
            println!("For more complex NL processing, implement BAML functions in baml_src/");
        }
    }

    Ok(())
}

async fn handle_native_balance(config: &Configuration, who: Option<String>) -> Result<()> {
    let who = who.unwrap_or_else(|| "vitalik.eth".to_string());
    
    info!("Getting native balance for: {}", who);
    
    let request = GetNativeBalanceRequest {
        who: who.clone(),
        __baml_options__: None,
    };

    match baml_api::get_native_balance(config, request).await {
        Ok(balance) => {
            println!("✅ Native balance for {}: {}", who, balance);
            info!("Successfully retrieved balance: {}", balance);
        },
        Err(e) => {
            error!("Failed to get native balance: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn handle_fungible_balance(
    config: &Configuration, 
    who: Option<String>, 
    token: Option<String>
) -> Result<()> {
    let who = who.unwrap_or_else(|| "vitalik.eth".to_string());
    let token = token.unwrap_or_else(|| {
        // USDC contract address on mainnet
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()
    });
    
    info!("Getting fungible balance for address: {}, token: {}", who, token);
    
    let request = GetFungibleBalanceRequest {
        token: token.clone(),
        who: who.clone(),
        __baml_options__: None,
    };

    match baml_api::get_fungible_balance(config, request).await {
        Ok(balance) => {
            println!("✅ Token balance for {} (token: {}): {}", who, token, balance);
            info!("Successfully retrieved token balance: {}", balance);
        },
        Err(e) => {
            error!("Failed to get fungible balance: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn handle_code_check(config: &Configuration, address: Option<String>) -> Result<()> {
    let address = address.unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());
    
    info!("Checking if address has deployed code: {}", address);
    
    let request = GetCodeRequest {
        addr: address.clone(),
        __baml_options__: None,
    };

    match baml_api::get_code(config, request).await {
        Ok(check_result) => {
            println!("✅ Code check for {}: deployed={}", 
                    address, check_result);
            info!("Code check result - deployed: {}", 
                 check_result);
        },
        Err(e) => {
            error!("Failed to check code: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_configuration_creation() {
        // Test that we can create a BAML configuration
        let config = Configuration::default();
        // This should not panic and should use environment variables
        assert_eq!(config.base_path, "http://localhost:3000");
    }

    #[test]
    fn test_args_parsing() {
        // Test that our CLI args parse correctly
        let args = Args::parse_from(vec!["baml-cecilia", "-q", "test query", "-d"]);
        assert_eq!(args.query, "test query");
        assert!(args.debug);
    }
}