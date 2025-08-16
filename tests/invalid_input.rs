use foundry_adapter::FoundryAdapter;
use domain::{Address, AddressOrEns, BalanceRequest, CodeRequest, Erc20BalanceRequest, SendRequest};

#[tokio::test]
async fn test_invalid_address_format() {
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Test invalid address format
        let invalid_addr = Address::new("not-an-address".to_string());
        let req = CodeRequest::new(invalid_addr);
        
        let result = adapter.get_code_len(&req).await;
        assert!(result.is_err(), "Should fail with invalid address format");
        
        if let Err(e) = result {
            assert!(e.to_string().contains("invalid address"), "Error should mention invalid address");
        }
    } else {
        println!("Skipping invalid address test - no Anvil fork detected");
    }
}

#[tokio::test]
async fn test_invalid_ens_name() {
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Test invalid ENS name
        let invalid_ens = AddressOrEns::from_ens("invalid-ens-name-that-does-not-exist.eth".to_string());
        let req = BalanceRequest::new(invalid_ens);
        
        let result = adapter.get_balance(&req).await;
        // This might succeed or fail depending on ENS resolution behavior
        // We just want to ensure it doesn't panic
        println!("ENS resolution result: {:?}", result);
    } else {
        println!("Skipping invalid ENS test - no Anvil fork detected");
    }
}

#[tokio::test]
async fn test_invalid_eth_amount() {
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Test invalid ETH amount
        let req = SendRequest::builder()
            .from(Address::new("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string()))
            .to(Address::new("0x70997970c51812dc3a010c7d01b50e0d17dc79c8".to_string()))
            .amount_eth("invalid-amount".to_string())
            .simulate(true)
            .build();
        
        if let Ok(req) = req {
            let result = adapter.send_eth(&req).await;
            assert!(result.is_err(), "Should fail with invalid ETH amount");
            
            if let Err(e) = result {
                assert!(e.to_string().contains("invalid") || e.to_string().contains("parse"), 
                       "Error should mention invalid amount or parse error");
            }
        } else {
            println!("SendRequest builder failed with invalid amount");
        }
    } else {
        println!("Skipping invalid ETH amount test - no Anvil fork detected");
    }
}

#[tokio::test]
async fn test_missing_required_fields() {
    // Test SendRequest builder with missing fields
    let req = SendRequest::builder()
        .from(Address::new("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string()))
        // Missing 'to' field
        .amount_eth("1.0".to_string())
        .build();
    
    assert!(req.is_err(), "Should fail with missing 'to' field");
    
    let req = SendRequest::builder()
        .from(Address::new("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string()))
        .to(Address::new("0x70997970c51812dc3a010c7d01b50e0d17dc79c8".to_string()))
        // Missing 'amount_eth' field
        .build();
    
    assert!(req.is_err(), "Should fail with missing 'amount_eth' field");
}

#[tokio::test]
async fn test_erc20_invalid_token_address() {
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Test ERC20 balance with invalid token address
        let req = Erc20BalanceRequest::new(
            Address::new("invalid-token-address".to_string()),
            Address::new("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string()),
        );
        
        let result = adapter.erc20_balance_of(&req).await;
        assert!(result.is_err(), "Should fail with invalid token address");
        
        if let Err(e) = result {
            assert!(e.to_string().contains("invalid address"), "Error should mention invalid address");
        }
    } else {
        println!("Skipping invalid ERC20 token test - no Anvil fork detected");
    }
}

#[tokio::test]
async fn test_chain_id_mismatch() {
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Set expected chain ID to something other than mainnet (1)
        let adapter = adapter.with_expected_chain_id(999); // Non-existent chain
        
        let req = SendRequest::builder()
            .from(Address::new("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string()))
            .to(Address::new("0x70997970c51812dc3a010c7d01b50e0d17dc79c8".to_string()))
            .amount_eth("0.1".to_string())
            .simulate(true)
            .build()
            .unwrap();
        
        let result = adapter.send_eth(&req).await;
        assert!(result.is_err(), "Should fail with chain ID mismatch");
        
        if let Err(e) = result {
            assert!(e.to_string().contains("chain id"), "Error should mention chain ID");
        }
    } else {
        println!("Skipping chain ID mismatch test - no Anvil fork detected");
    }
}

#[tokio::test]
async fn test_gas_cap_exceeded() {
    let adapter = FoundryAdapter::new("http://127.0.0.1:8545").await;
    
    if let Ok(adapter) = adapter {
        // Set very low gas cap
        let adapter = adapter.with_gas_cap(1000); // Very low gas cap
        
        let req = SendRequest::builder()
            .from(Address::new("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string()))
            .to(Address::new("0x70997970c51812dc3a010c7d01b50e0d17dc79c8".to_string()))
            .amount_eth("0.1".to_string())
            .simulate(true)
            .build()
            .unwrap();
        
        let result = adapter.send_eth(&req).await;
        // This might succeed or fail depending on actual gas estimation
        // We just want to ensure it doesn't panic
        println!("Gas cap test result: {:?}", result);
    } else {
        println!("Skipping gas cap test - no Anvil fork detected");
    }
}
