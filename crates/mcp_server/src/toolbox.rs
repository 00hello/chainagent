use anyhow::Result;
use async_trait::async_trait;
use domain::*;
use foundry_adapter::FoundryAdapter;

pub struct ServerToolbox {
    adapter: FoundryAdapter,
}

impl ServerToolbox {
    pub fn new(adapter: FoundryAdapter) -> Self { Self { adapter } }
}

#[async_trait]
impl Toolbox for ServerToolbox {
    async fn balance(&self, req: BalanceRequest) -> Result<BalanceResponse> {
        let wei = self.adapter.get_balance(&req).await?;
        Ok(BalanceResponse::new(wei))
    }

    async fn code(&self, req: CodeRequest) -> Result<CodeResponse> {
        let (deployed, bytecode_len) = self.adapter.get_code_len(&req).await?;
        Ok(CodeResponse::new(deployed, bytecode_len))
    }

    async fn erc20_balance_of(&self, req: Erc20BalanceRequest) -> Result<Erc20BalanceResponse> {
        let amount = self.adapter.erc20_balance_of(&req).await?;
        Ok(Erc20BalanceResponse::new(amount))
    }

    async fn send(&self, req: SendRequest) -> Result<TxResult> {
        self.adapter.send_eth(&req).await
    }
}

