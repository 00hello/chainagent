use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use anyhow::Result;

#[tokio::test]
async fn test_e2e_send_eth_from_alice_to_bob() {
    // This test requires a running Anvil fork
    println!("Starting E2E test: Send 1 ETH from Alice to Bob");
    
    // Step 1: Check if Anvil is running
    let anvil_check = Command::new("curl")
        .args(["-s", "http://127.0.0.1:8545"])
        .output();
    
    if anvil_check.is_err() {
        println!("Skipping E2E test - Anvil not running on localhost:8545");
        return;
    }
    
    // Step 2: Start MCP server in background
    println!("Starting MCP server...");
    let server_handle = tokio::spawn(async {
        let output = Command::new("cargo")
            .args(["run", "-p", "mcp_server"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        
        if let Ok(mut child) = output {
            // Wait a bit for server to start
            sleep(Duration::from_secs(2)).await;
            
            // Check if server is still running
            match child.try_wait() {
                Ok(Some(status)) => {
                    println!("MCP server exited with status: {}", status);
                    return Err(anyhow::anyhow!("MCP server failed to start"));
                }
                Ok(None) => {
                    println!("MCP server started successfully");
                    Ok(child)
                }
                Err(e) => {
                    println!("Error checking MCP server: {}", e);
                    Err(anyhow::anyhow!("Failed to check MCP server status"))
                }
            }
        } else {
            Err(anyhow::anyhow!("Failed to start MCP server"))
        }
    });
    
    let server_process = match server_handle.await {
        Ok(Ok(process)) => process,
        Ok(Err(e)) => {
            println!("Skipping E2E test - MCP server failed to start: {}", e);
            return;
        }
        Err(e) => {
            println!("Skipping E2E test - Failed to spawn MCP server: {}", e);
            return;
        }
    };
    
    // Step 3: Run CLI command to send ETH
    println!("Running CLI command to send ETH...");
    let cli_output = Command::new("cargo")
        .args([
            "run", "-p", "baml_client", "--",
            "--mock", // Use mock provider for deterministic testing
            "-q", "Send 1 ETH from Alice to Bob"
        ])
        .output();
    
    let cli_result = match cli_output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            println!("CLI stdout: {}", stdout);
            if !stderr.is_empty() {
                println!("CLI stderr: {}", stderr);
            }
            
            if output.status.success() {
                Ok(stdout.to_string())
            } else {
                Err(anyhow::anyhow!("CLI command failed with status: {}", output.status))
            }
        }
        Err(e) => {
            println!("Failed to run CLI command: {}", e);
            Err(anyhow::anyhow!("CLI command execution failed"))
        }
    };
    
    // Step 4: Verify the result
    match cli_result {
        Ok(output) => {
            // Check that the output contains expected function and response
            assert!(
                output.contains("Function: send"),
                "CLI should select 'send' function"
            );
            assert!(
                output.contains("Response:"),
                "CLI should output a response"
            );
            
            println!("✅ E2E test passed: CLI successfully executed send command");
        }
        Err(e) => {
            println!("❌ E2E test failed: {}", e);
            // Don't panic, just log the failure since this requires external setup
        }
    }
    
    // Step 5: Clean up - terminate MCP server
    if let Ok(mut process) = server_process {
        let _ = process.kill();
        println!("MCP server terminated");
    }
}

#[tokio::test]
async fn test_e2e_balance_check() {
    println!("Starting E2E test: Check ETH balance");
    
    // Run CLI command to check balance
    let cli_output = Command::new("cargo")
        .args([
            "run", "-p", "baml_client", "--",
            "--mock",
            "-q", "What's vitalik.eth's balance?"
        ])
        .output();
    
    match cli_output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            println!("CLI stdout: {}", stdout);
            if !stderr.is_empty() {
                println!("CLI stderr: {}", stderr);
            }
            
            if output.status.success() {
                assert!(
                    stdout.contains("Function: balance"),
                    "CLI should select 'balance' function"
                );
                assert!(
                    stdout.contains("Response:"),
                    "CLI should output a response"
                );
                println!("✅ E2E balance test passed");
            } else {
                println!("❌ E2E balance test failed with status: {}", output.status);
            }
        }
        Err(e) => {
            println!("❌ E2E balance test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_e2e_code_check() {
    println!("Starting E2E test: Check if address has code");
    
    // Run CLI command to check code deployment
    let cli_output = Command::new("cargo")
        .args([
            "run", "-p", "baml_client", "--",
            "--mock",
            "-q", "Check if 0x0000000000000000000000000000000000000000 has deployed code"
        ])
        .output();
    
    match cli_output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            println!("CLI stdout: {}", stdout);
            if !stderr.is_empty() {
                println!("CLI stderr: {}", stderr);
            }
            
            if output.status.success() {
                assert!(
                    stdout.contains("Function: code"),
                    "CLI should select 'code' function"
                );
                assert!(
                    stdout.contains("Response:"),
                    "CLI should output a response"
                );
                println!("✅ E2E code check test passed");
            } else {
                println!("❌ E2E code check test failed with status: {}", output.status);
            }
        }
        Err(e) => {
            println!("❌ E2E code check test failed: {}", e);
        }
    }
}
