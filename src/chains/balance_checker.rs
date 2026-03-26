use anyhow::Result;
use ethers::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{info, warn};

/// Check if an address has sufficient balance for a swap
pub async fn check_balance(
    chain: &str,
    asset: &str,
    owner: &str,
    required_amount: &str,
    rpc_url: &str,
) -> Result<bool> {
    // Parse the required amount
    let required: u128 = required_amount.parse().unwrap_or(0);
    
    if required == 0 {
        return Ok(true); // No amount required
    }

    // Extract chain and token from asset (e.g., "arbitrum_sepolia:usdc")
    let parts: Vec<&str> = asset.split(':').collect();
    if parts.len() != 2 {
        warn!("Invalid asset format: {}", asset);
        return Ok(false);
    }
    
    let token_symbol = parts[1];

    // Check balance based on chain type
    if chain.contains("sepolia") || chain.contains("base") || chain.contains("arbitrum") 
        || chain.contains("ethereum") || chain.contains("evm") {
        // Add timeout to prevent hanging
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            check_evm_balance(token_symbol, owner, required, rpc_url)
        ).await {
            Ok(result) => result,
            Err(_) => {
                warn!("Balance check timed out for {}, assuming sufficient", asset);
                Ok(true) // Assume sufficient on timeout
            }
        }
    } else if chain.contains("solana") {
        check_solana_balance(token_symbol, owner, required, rpc_url).await
    } else {
        // For other chains (Bitcoin, Starknet, Tron, etc.), assume balance is sufficient
        // These require manual checking or different APIs
        info!("Skipping balance check for chain: {} (not implemented)", chain);
        Ok(true)
    }
}

/// Check EVM token balance
async fn check_evm_balance(
    token_symbol: &str,
    owner: &str,
    required: u128,
    rpc_url: &str,
) -> Result<bool> {
    // Create provider with timeout
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| anyhow::anyhow!("Failed to create provider: {}", e))?;
    
    let address = Address::from_str(owner)
        .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

    // Check native token (ETH) balance
    if token_symbol == "eth" {
        match provider.get_balance(address, None).await {
            Ok(balance) => {
                let balance_u128: u128 = balance.as_u128();
                
                info!(
                    "ETH balance: {} wei, required: {} wei, sufficient: {}",
                    balance_u128,
                    required,
                    balance_u128 >= required
                );
                
                return Ok(balance_u128 >= required);
            }
            Err(e) => {
                warn!("Failed to get ETH balance: {}, assuming sufficient", e);
                return Ok(true);
            }
        }
    }

    // For ERC20 tokens, try to get balance
    match get_token_address(token_symbol, rpc_url) {
        Ok(token_address) => {
            // ERC20 balanceOf ABI
            let abi_json = r#"[{"constant":true,"inputs":[{"name":"_owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"type":"function"}]"#;
            let abi: ethers::abi::Abi = serde_json::from_str(abi_json)?;
            let contract = Contract::new(token_address, abi, Arc::new(provider));
            
            match contract.method::<_, U256>("balanceOf", address)?.call().await {
                Ok(balance) => {
                    let balance_u128: u128 = balance.as_u128();
                    
                    info!(
                        "{} balance: {}, required: {}, sufficient: {}",
                        token_symbol.to_uppercase(),
                        balance_u128,
                        required,
                        balance_u128 >= required
                    );
                    
                    Ok(balance_u128 >= required)
                }
                Err(e) => {
                    warn!("Failed to get {} balance: {}, assuming sufficient", token_symbol, e);
                    Ok(true)
                }
            }
        }
        Err(e) => {
            warn!("Token address not found for {}: {}, assuming sufficient", token_symbol, e);
            Ok(true)
        }
    }
}

/// Check Solana token balance
async fn check_solana_balance(
    _token_symbol: &str,
    _owner: &str,
    _required: u128,
    _rpc_url: &str,
) -> Result<bool> {
    // Solana balance checking would require solana-client
    // For now, assume sufficient balance
    info!("Solana balance check not implemented, assuming sufficient");
    Ok(true)
}

/// Get token contract address for a given symbol and chain
fn get_token_address(symbol: &str, rpc_url: &str) -> Result<Address> {
    // Token addresses for Sepolia testnets (devnet/testnet)
    let address_str = if rpc_url.contains("arbitrum") {
        match symbol.to_lowercase().as_str() {
            "usdc" => "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d", // Arbitrum Sepolia USDC
            "wbtc" => "0xb5ae9785349186069c48794a763db39ec756b1cf", // Arbitrum Sepolia WBTC
            _ => return Err(anyhow::anyhow!("Unknown token: {}", symbol)),
        }
    } else if rpc_url.contains("base") {
        match symbol.to_lowercase().as_str() {
            "usdc" => "0x036CbD53842c5426634e7929541eC2318f3dCF7e", // Base Sepolia USDC
            "wbtc" => "0x4200000000000000000000000000000000000006", // Base Sepolia Wrapped ETH (placeholder)
            "weth" => "0x4200000000000000000000000000000000000006", // Base Sepolia Wrapped ETH
            _ => return Err(anyhow::anyhow!("Unknown token: {}", symbol)),
        }
    } else if rpc_url.contains("sepolia") || rpc_url.contains("ethereum") {
        match symbol.to_lowercase().as_str() {
            "usdc" => "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238", // Ethereum Sepolia USDC
            "wbtc" => "0x29f2D40B0605204364af54EC677bD022dA425d03", // Ethereum Sepolia WBTC
            _ => return Err(anyhow::anyhow!("Unknown token: {}", symbol)),
        }
    } else {
        return Err(anyhow::anyhow!("Unknown chain RPC: {}", rpc_url));
    };

    Address::from_str(address_str).map_err(|e| anyhow::anyhow!("Invalid address: {}", e))
}
