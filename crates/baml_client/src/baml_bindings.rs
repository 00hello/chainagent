use anyhow::Result;
use crate::baml::BamlFunction;

// Minimal BAML bindings shim: validates inputs and normalizes, then produces
// a strongly-typed BamlFunction. Names mirror .baml function surfaces.

// Accepted function names (BAML-era and legacy), mapped to chain-neutral tools
fn canonical_name(name: &str) -> &str {
    match name {
        "GetEthBalance" => "GetNativeBalance",
        "IsDeployed" => "GetCode",
        "GetErc20Balance" => "GetFungibleBalance",
        "SendEth" => "SendNative",
        other => other,
    }
}

pub fn validate_and_to_baml_function(name: &str, input: &serde_json::Value) -> Result<BamlFunction> {
    let name = canonical_name(name);
    match name {
        "GetNativeBalance" => {
            let who = input.get("who").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'who' parameter"))?;
            let who_s = who.to_string();
            let addr_or_ens = if who_s.ends_with(".eth") { domain::AddressOrEns::from_ens(who_s) } else { domain::AddressOrEns::from_address(who_s) };
            Ok(BamlFunction::Balance(domain::BalanceRequest::new(addr_or_ens)))
        }
        "GetCode" => {
            let addr = input.get("addr").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'addr' parameter"))?;
            Ok(BamlFunction::Code(domain::CodeRequest::new(domain::Address::new(addr.to_string()))))
        }
        "GetFungibleBalance" => {
            let token = input.get("token").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'token' parameter"))?;
            let holder = input.get("holder").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'holder' parameter"))?;
            Ok(BamlFunction::Erc20Balance(domain::Erc20BalanceRequest::new(
                domain::Address::new(token.to_string()),
                domain::Address::new(holder.to_string()),
            )))
        }
        "SendNative" => {
            let from = input.get("from").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'from' parameter"))?;
            let to = input.get("to").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'to' parameter"))?;
            let amount_eth = input.get("amount_eth").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'amount_eth' parameter"))?;
            let simulate = input.get("simulate").and_then(|v| v.as_bool()).unwrap_or(true);
            Ok(BamlFunction::Send(
                domain::SendRequest::builder()
                    .from(domain::Address::new(from.to_string()))
                    .to(domain::Address::new(to.to_string()))
                    .amount_eth(amount_eth.to_string())
                    .simulate(simulate)
                    .build().map_err(|e| anyhow::anyhow!("{}", e))?
            ))
        }
        other => anyhow::bail!("Unknown function type: {}", other),
    }
}


