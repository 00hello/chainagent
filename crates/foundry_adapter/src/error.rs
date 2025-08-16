use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("unexpected chain id: got {got} expected {expected}")]
    ChainIdMismatch { got: u64, expected: u64 },

    #[error("estimated gas {estimated} exceeds cap {cap}")]
    GasCapExceeded { estimated: u64, cap: u64 },

    #[error("no local key for from address {0}")]
    MissingLocalKey(String),

    #[error(transparent)]
    Provider(#[from] ethers_providers::ProviderError),

    #[error(transparent)]
    Abi(#[from] ethers_core::abi::Error),

    #[error(transparent)]
    Contract(#[from] ethers_contract::ContractError<ethers_providers::Provider<ethers_providers::Http>>),

    #[error(transparent)]
    Signer(#[from] ethers_signers::WalletError),

    #[error("invalid address: {0}")]
    AddrParse(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

