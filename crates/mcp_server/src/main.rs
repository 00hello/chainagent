mod dto;
mod facade;
mod toolbox;
#[cfg(feature = "bonus_uniswap_v2")]
mod uniswap_v2;
mod external_api;

use dto::{TokenLookupIn, TokenLookupOut};

fn main() {
    let bonus_enabled = std::env::var("BONUS").ok().map(|v| v == "1").unwrap_or(false);
    if bonus_enabled { println!("mcp_server: BONUS features enabled"); }
    println!("mcp_server starting (placeholder)");
}

#[allow(dead_code)]
async fn handle_token_lookup(req: TokenLookupIn) -> anyhow::Result<TokenLookupOut> {
    let mut client = external_api::TokenLookupClient::new("http://localhost:8080", 60);
    let result = client.lookup_by_symbol(&req.symbol, &req.chain).await?;
    Ok(TokenLookupOut {
        address: result.map(|t| t.address),
        symbol: req.symbol,
        chain: req.chain,
    })
}

