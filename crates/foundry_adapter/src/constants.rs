use ethers_core::types::Address;

// Mainnet contract addresses
pub const USDC_MAINNET: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub const UNISWAP_V2_ROUTER: &str = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";

// Anvil default accounts (seeded with 10000 ETH each)
pub const ANVIL_ACCOUNT_0: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"; // Alice
pub const ANVIL_ACCOUNT_1: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8"; // Bob
pub const ANVIL_ACCOUNT_2: &str = "0x3c44cdddb6a900fa2b585dd299e03d12fa4293bc"; // Charlie
pub const ANVIL_ACCOUNT_3: &str = "0x90f79bf6eb2c4f870365e785982e1f101e93b906"; // David
pub const ANVIL_ACCOUNT_4: &str = "0x15d34aaf54267db7d7c367839aaf71a00a2c6a65"; // Eve

// Default configuration
pub const DEFAULT_GAS_CAP: u64 = 30_000_000; // 30M gas
pub const DEFAULT_CHAIN_ID: u64 = 1; // Mainnet
pub const DEFAULT_RPC_URL: &str = "http://127.0.0.1:8545"; // Anvil default

// ENS resolution
pub const ENS_REGISTRY: &str = "0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e";
pub const ENS_RESOLVER: &str = "0x4976fb03C32e5B8cfe2b6cCB31c09Ba78EBaBa41";

// Cache configuration
pub const LRU_CACHE_SIZE: usize = 1000;
pub const CACHE_TTL_SECONDS: u64 = 3600; // 1 hour

pub fn get_anvil_accounts() -> Vec<Address> {
    vec![
        ANVIL_ACCOUNT_0.parse().unwrap(),
        ANVIL_ACCOUNT_1.parse().unwrap(),
        ANVIL_ACCOUNT_2.parse().unwrap(),
        ANVIL_ACCOUNT_3.parse().unwrap(),
        ANVIL_ACCOUNT_4.parse().unwrap(),
    ]
}

pub fn get_anvil_account_aliases() -> std::collections::HashMap<String, Address> {
    let mut aliases = std::collections::HashMap::new();
    aliases.insert("Alice".to_string(), ANVIL_ACCOUNT_0.parse().unwrap());
    aliases.insert("Bob".to_string(), ANVIL_ACCOUNT_1.parse().unwrap());
    aliases.insert("Charlie".to_string(), ANVIL_ACCOUNT_2.parse().unwrap());
    aliases.insert("David".to_string(), ANVIL_ACCOUNT_3.parse().unwrap());
    aliases.insert("Eve".to_string(), ANVIL_ACCOUNT_4.parse().unwrap());
    aliases
}
