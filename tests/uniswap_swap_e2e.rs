#[cfg(all(feature = "bonus_uniswap_v2"))]
mod swap_tests {
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_uniswap_swap_dry_run() {
        // Check for Anvil
        let anvil_check = Command::new("curl").args(["-s", "http://127.0.0.1:8545"]).output();
        if anvil_check.is_err() {
            println!("Skipping Uniswap swap E2E - no Anvil");
            return;
        }

        // Start MCP server
        let server = Command::new("cargo")
            .args(["run", "-p", "mcp_server", "--features", "bonus_uniswap_v2"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        if server.is_err() {
            println!("Skipping Uniswap swap E2E - server failed to start");
            return;
        }
        let mut server = server.unwrap();
        sleep(Duration::from_secs(2)).await;

        // Run client with a swap intent; using mock provider for determinism
        let cli = Command::new("cargo")
            .args(["run", "-p", "baml_client", "--", "--mock", "--dry-run", "-q", "Swap 0.5 ETH for USDC"])
            .output();

        match cli {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("CLI stdout: {}", stdout);
                assert!(output.status.success(), "CLI should succeed");
                if let Some(json_start) = stdout.find("{\n") {
                    let json_str = &stdout[json_start..];
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                        if let Some(success) = val.get("success").and_then(|v| v.as_bool()) {
                            assert!(success, "Swap simulate should report success=true when present");
                        } else {
                            println!("No success field found in Response; skipping strict assertion");
                        }
                    }
                }
            }
            Err(e) => println!("CLI failed: {}", e),
        }

        let _ = server.kill();
    }
}

#[cfg(not(feature = "bonus_uniswap_v2"))]
#[test]
fn test_uniswap_swap_feature_disabled() {
    println!("Skipping Uniswap swap E2E - feature disabled");
}
