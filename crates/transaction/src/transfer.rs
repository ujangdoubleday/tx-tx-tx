use ethers::prelude::*;
use std::convert::TryFrom;
use x_core::networks::Network;
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
    let private_key = x_core::config::normalize_private_key(private_key);

    let wallet = private_key
        .parse::<LocalWallet>()
        .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;

    let rpc_url = network.rpc.first()
        .ok_or_else(|| anyhow::anyhow!("No RPC URL available for network"))?;

    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| anyhow::anyhow!("Failed to create provider: {}", e))?;

    let client = SignerMiddleware::new(provider, wallet.with_chain_id(network.chain_id));

    let to_address = x_core::crypto::normalize_address(to_address)?;
    let to_addr_bytes = x_core::crypto::hex_to_bytes(&to_address)?;
    let to_addr = Address::from_slice(&to_addr_bytes);

    let amount_wei = ethers::utils::parse_ether(amount_eth)
        .map_err(|e| anyhow::anyhow!("Failed to parse amount: {}", e))?;

    let mut tx = TransactionRequest::new()
        .to(to_addr)
        .value(amount_wei)
        .chain_id(network.chain_id);

    if let Some(notes) = notes {
        if !notes.trim().is_empty() {
            tx = tx.data(notes.as_bytes().to_vec());
        }
    }

    let pending_tx = client.send_transaction(tx, None).await
        .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

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
