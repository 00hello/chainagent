mod dto;
mod facade;
mod toolbox;
#[cfg(feature = "bonus_uniswap_v2")]
mod uniswap_v2;
mod external_api;

use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::post,
    Router,
};
use dto::{BalanceIn, CodeIn, Erc20BalanceIn, SendIn, TokenLookupIn, TokenLookupOut};
use foundry_adapter::FoundryAdapter;
use serde_json::{json, Value};
use std::sync::Arc;
use toolbox::ServerToolbox;
use domain::Toolbox;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let bonus_enabled = std::env::var("BONUS").ok().map(|v| v == "1").unwrap_or(false);
    if bonus_enabled { 
        info!("mcp_server: BONUS features enabled"); 
    }
    
    info!("mcp_server starting HTTP server on :3000");
    
    // Initialize adapter with RPC URL from env or default to Anvil
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());
    let adapter = FoundryAdapter::new(&rpc_url).await?;
    let toolbox = Arc::new(ServerToolbox::new(adapter));
    
    let app = Router::new()
        .route("/balance", post(handle_balance))
        .route("/code", post(handle_code))
        .route("/erc20_balance_of", post(handle_erc20_balance))
        .route("/send", post(handle_send))
        .route("/token_lookup", post(handle_token_lookup))
        .with_state(toolbox);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

// HTTP Handlers
async fn handle_balance(
    axum::extract::State(toolbox): axum::extract::State<Arc<ServerToolbox>>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let balance_in: BalanceIn = serde_json::from_value(payload).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match balance_in.try_into() {
        Ok(req) => {
            match toolbox.balance(req).await {
                Ok(response) => Ok(ResponseJson(json!({ "balance": response.wei() }))),
                Err(e) => {
                    error!("Balance error: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Invalid balance request: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn handle_code(
    axum::extract::State(toolbox): axum::extract::State<Arc<ServerToolbox>>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let code_in: CodeIn = serde_json::from_value(payload).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match code_in.try_into() {
        Ok(req) => {
            match toolbox.code(req).await {
                Ok(response) => Ok(ResponseJson(json!({ 
                    "deployed": response.deployed(), 
                    "bytecode_len": response.bytecode_len() 
                }))),
                Err(e) => {
                    error!("Code error: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Invalid code request: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn handle_erc20_balance(
    axum::extract::State(toolbox): axum::extract::State<Arc<ServerToolbox>>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let erc20_in: Erc20BalanceIn = serde_json::from_value(payload).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match erc20_in.try_into() {
        Ok(req) => {
            match toolbox.erc20_balance_of(req).await {
                Ok(response) => Ok(ResponseJson(json!({ "amount": response.amount() }))),
                Err(e) => {
                    error!("ERC20 balance error: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Invalid ERC20 balance request: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn handle_send(
    axum::extract::State(toolbox): axum::extract::State<Arc<ServerToolbox>>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let send_in: SendIn = serde_json::from_value(payload).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match send_in.try_into() {
        Ok(req) => {
            match toolbox.send(req).await {
                Ok(result) => Ok(ResponseJson(json!({ 
                    "tx_hash": result.tx_hash(),
                    "success": result.status().unwrap_or(false)
                }))),
                Err(e) => {
                    error!("Send error: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Invalid send request: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn handle_token_lookup(
    axum::extract::State(_toolbox): axum::extract::State<Arc<ServerToolbox>>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token_in: TokenLookupIn = serde_json::from_value(payload).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let mut client = external_api::TokenLookupClient::new("http://localhost:8080", 60);
    match client.lookup_by_symbol(&token_in.symbol, &token_in.chain).await {
        Ok(result) => {
            let response = TokenLookupOut {
                address: result.map(|t| t.address),
                symbol: token_in.symbol,
                chain: token_in.chain,
            };
            Ok(ResponseJson(json!({ 
                "address": response.address,
                "symbol": response.symbol,
                "chain": response.chain
            })))
        }
        Err(e) => {
            error!("Token lookup error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

