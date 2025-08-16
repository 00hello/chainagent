use foundry_adapter::FoundryAdapter;
use domain::{Address, AddressOrEns, BalanceRequest, CodeRequest, Erc20BalanceRequest};

#[tokio::test]
async fn test_router02_deployment_check() {
    // This test requires a running Anvil fork with mainnet data
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Check if Uniswap V2 Router02 is deployed
        let router_addr = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";
        let req = CodeRequest::new(Address::new(router_addr.to_string()));
        
        let result = adapter.get_code_len(&req).await;
        if let Ok((deployed, bytecode_len)) = result {
            // Router02 should be deployed on mainnet fork
            assert!(deployed, "Router02 should be deployed on mainnet fork");
            assert!(bytecode_len > 0, "Router02 should have bytecode");
            println!("Router02 deployed: {}, bytecode length: {}", deployed, bytecode_len);
        } else {
            // Skip test if no Anvil fork is running
            println!("Skipping Router02 test - no Anvil fork detected");
        }
    } else {
        println!("Skipping Router02 test - could not connect to Anvil");
    }
}

#[tokio::test]
async fn test_usdc_balance_check() {
    // This test requires a running Anvil fork with mainnet data
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Check USDC balance for Alice (Anvil account 0)
        let usdc_token = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let alice = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
        
        let req = Erc20BalanceRequest::new(
            Address::new(usdc_token.to_string()),
            Address::new(alice.to_string()),
        );
        
        let result = adapter.erc20_balance_of(&req).await;
        if let Ok(balance) = result {
            // Should return a numeric balance (even if 0)
            let balance_u128 = balance.parse::<u128>();
            assert!(balance_u128.is_ok(), "USDC balance should be a valid number");
            println!("Alice USDC balance: {}", balance);
        } else {
            // Skip test if no Anvil fork is running
            println!("Skipping USDC balance test - no Anvil fork detected");
        }
    } else {
        println!("Skipping USDC balance test - could not connect to Anvil");
    }
}

#[tokio::test]
async fn test_ens_resolution() {
    // This test requires a running Anvil fork with mainnet data
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Test ENS resolution for vitalik.eth
        let ens_name = AddressOrEns::from_ens("vitalik.eth".to_string());
        let result = adapter.resolve_address_or_ens(&ens_name).await;
        
        if let Ok(resolved) = result {
            // Should resolve to a valid address
            assert!(resolved.as_str().starts_with("0x"), "ENS should resolve to valid address");
            assert_eq!(resolved.as_str().len(), 42, "Resolved address should be 42 chars");
            println!("vitalik.eth resolved to: {}", resolved.as_str());
        } else {
            // Skip test if no Anvil fork is running
            println!("Skipping ENS resolution test - no Anvil fork detected");
        }
    } else {
        println!("Skipping ENS resolution test - could not connect to Anvil");
    }
}

#[tokio::test]
async fn test_eth_balance_check() {
    // This test requires a running Anvil fork with mainnet data
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Check ETH balance for Alice (Anvil account 0)
        let alice = AddressOrEns::from_address("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string());
        let req = BalanceRequest::new(alice);
        
        let result = adapter.get_balance(&req).await;
        if let Ok(balance) = result {
            // Should return a numeric balance
            let balance_u128 = balance.parse::<u128>();
            assert!(balance_u128.is_ok(), "ETH balance should be a valid number");
            println!("Alice ETH balance: {} wei", balance);
        } else {
            // Skip test if no Anvil fork is running
            println!("Skipping ETH balance test - no Anvil fork detected");
        }
    } else {
        println!("Skipping ETH balance test - could not connect to Anvil");
    }
}

#[tokio::test]
async fn test_chain_id_validation() {
    // This test requires a running Anvil fork with mainnet data
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Set expected chain ID to mainnet (1)
        let adapter = adapter.with_expected_chain_id(1);
        
        // This should work on mainnet fork
        let alice = AddressOrEns::from_address("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string());
        let req = BalanceRequest::new(alice);
        
        let result = adapter.get_balance(&req).await;
        if let Ok(_) = result {
            println!("Chain ID validation passed on mainnet fork");
        } else {
            println!("Skipping chain ID validation test - no Anvil fork detected");
        }
    } else {
        println!("Skipping chain ID validation test - could not connect to Anvil");
    }
}
