use ethers::prelude::*;
use std::convert::TryFrom;
use crate::core::networks::Network;
use anyhow::Result;

#[derive(Debug)]
pub struct TransferResult {
    pub tx_hash: String,
}

pub async fn transfer_eth_async(
    private_key: &str,
    to_address: &str,
    amount_eth: f64,
    network: &Network,
    notes: Option<&str>,
) -> Result<TransferResult> {
    // Normalize private key
    let private_key = crate::core::config::normalize_private_key(private_key);

    // Create wallet
    let wallet = private_key
        .parse::<LocalWallet>()
        .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;

    // Get RPC URL (use first one)
    let rpc_url = network.rpc.first()
        .ok_or_else(|| anyhow::anyhow!("No RPC URL available for network"))?;

    // Create provider
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| anyhow::anyhow!("Failed to create provider: {}", e))?;

    // Connect wallet to provider
    let client = SignerMiddleware::new(provider, wallet.with_chain_id(network.chain_id));

    // Normalize recipient address
    let to_address = crate::core::crypto::normalize_address(to_address)?;
    let to_addr_bytes = crate::core::crypto::hex_to_bytes(&to_address)?;
    let to_addr = Address::from_slice(&to_addr_bytes);

    // Convert amount to wei
    let amount_wei = ethers::utils::parse_ether(amount_eth)
        .map_err(|e| anyhow::anyhow!("Failed to parse amount: {}", e))?;

    // Build transaction
    let mut tx = TransactionRequest::new()
        .to(to_addr)
        .value(amount_wei)
        .chain_id(network.chain_id);

    // Add notes as data if provided
    if let Some(notes) = notes {
        if !notes.trim().is_empty() {
            tx = tx.data(notes.as_bytes().to_vec());
        }
    }

    // Send transaction
    let pending_tx = client.send_transaction(tx, None).await
        .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

    // Wait for confirmation
    let receipt = pending_tx.confirmations(1).await
        .map_err(|e| anyhow::anyhow!("Failed to confirm transaction: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("Transaction confirmation timeout"))?;

    let tx_hash = format!("{:?}", receipt.transaction_hash);

    Ok(TransferResult { tx_hash })
}

pub fn transfer_eth(
    private_key: &str,
    to_address: &str,
    amount_eth: f64,
    network: &Network,
    notes: Option<&str>,
) -> Result<TransferResult> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| anyhow::anyhow!("Failed to create runtime: {}", e))?;

    rt.block_on(transfer_eth_async(private_key, to_address, amount_eth, network, notes))
}