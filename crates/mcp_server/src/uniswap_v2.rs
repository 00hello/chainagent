#[cfg(feature = "bonus_uniswap_v2")]
pub mod uniswap_v2 {
    use domain::{Address, UniswapV2SwapRequest, UniswapV2SwapResponse};

    pub fn build_swap_exact_eth_for_tokens_calldata(req: &UniswapV2SwapRequest, weth: &Address, token_out: &Address, recipient: &Address) -> Vec<u8> {
        // Placeholder: Return empty calldata; real implementation would encode ABI
        let _ = (req, weth, token_out, recipient);
        vec![]
    }

    pub async fn simulate_or_send_swap(_req: UniswapV2SwapRequest) -> anyhow::Result<UniswapV2SwapResponse> {
        // Placeholder: return simulated success
        Ok(UniswapV2SwapResponse::new(
            "0xSIMULATED".to_string(),
            None,
            vec![],
            None,
            Some(true),
        ))
    }
}
