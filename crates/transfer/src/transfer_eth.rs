use ethers::prelude::*;
use ethers::types::Eip1559TransactionRequest;
use x_core::networks::Network;
use x_core::gas::{GasCalculator, GasStrategy};
use x_core::network::{HttpClient, WebSocketClient};
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
    transfer_eth_with_strategy_async(private_key, to_address, amount_eth, network, notes, GasStrategy::Standard).await
}

pub async fn transfer_eth_with_strategy_async(
    private_key: &str,
    to_address: &str,
    amount_eth: f64,
    network: &Network,
    notes: Option<&str>,
    gas_strategy: GasStrategy,
) -> Result<TransferResult> {
    let private_key = x_core::config::normalize_private_key(private_key);

    let wallet = private_key
        .parse::<LocalWallet>()
        .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;

    let rpc_url = network.rpc.first()
        .ok_or_else(|| anyhow::anyhow!("No RPC URL available for network"))?;

    let http_client = HttpClient::new(rpc_url).await?;
    let provider = http_client.get_provider();

    let ws_client = if !network.ws_rpc.is_empty() {
        Some(WebSocketClient::new(&network.ws_rpc[0]))
    } else {
        None
    };

    let client = SignerMiddleware::new(provider.clone(), wallet.with_chain_id(network.chain_id));

    let to_address = x_core::crypto::normalize_address(to_address)?;
    let to_addr_bytes = x_core::crypto::hex_to_bytes(&to_address)?;
    let to_addr = Address::from_slice(&to_addr_bytes);

    let from_addr = client.address();

    let amount_wei = ethers::utils::parse_ether(amount_eth)
        .map_err(|e| anyhow::anyhow!("Failed to parse amount: {}", e))?;

    let data = notes.and_then(|n| {
        if n.trim().is_empty() {
            None
        } else {
            Some(n.as_bytes().to_vec())
        }
    });

    let is_eip1559 = check_eip1559_support(&client).await.unwrap_or(true);

    if is_eip1559 {
        let gas_estimate = GasCalculator::estimate_gas(
            &client,
            from_addr,
            to_addr,
            amount_wei,
            data.clone(),
            gas_strategy,
            Some(from_addr),
        )
        .await?;

        let mut tx = Eip1559TransactionRequest::new()
            .to(to_addr)
            .value(amount_wei)
            .gas(gas_estimate.gas_limit)
            .chain_id(network.chain_id);

        if let Some(d) = data {
            tx = tx.data(d);
        }

        if let (Some(max_priority_fee), Some(max_fee_per_gas)) =
            (gas_estimate.max_priority_fee, gas_estimate.max_fee_per_gas)
        {
            tx = tx
                .max_priority_fee_per_gas(max_priority_fee)
                .max_fee_per_gas(max_fee_per_gas);
        }

        let pending_tx = client.send_transaction(tx, None).await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

        let tx_hash = pending_tx.tx_hash();

        if let Some(ws_client) = &ws_client {
            match ws_client.wait_for_transaction_confirmation(tx_hash).await {
                Ok(_) => {
                    let tx_hash_str = format!("{:?}", tx_hash);
                    return Ok(TransferResult { tx_hash: tx_hash_str });
                }
                Err(e) => {
                    eprintln!("Warning: WebSocket confirmation failed: {}, falling back to HTTP polling", e);
                }
            }
        }

        let receipt = pending_tx.confirmations(1).await
            .map_err(|e| anyhow::anyhow!("Failed to confirm transaction: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("Transaction confirmation timeout"))?;

        let tx_hash_str = format!("{:?}", receipt.transaction_hash);
        Ok(TransferResult { tx_hash: tx_hash_str })
    } else {
        let gas_estimate = GasCalculator::estimate_gas_legacy(
            &client,
            from_addr,
            to_addr,
            amount_wei,
            data.clone(),
            gas_strategy,
            Some(from_addr),
        )
        .await?;

        let tx = TransactionRequest::new()
            .to(to_addr)
            .value(amount_wei)
            .gas(gas_estimate.gas_limit)
            .gas_price(gas_estimate.gas_price)
            .chain_id(network.chain_id)
            .data(data.unwrap_or_default());

        let pending_tx = client.send_transaction(tx, None).await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

        let tx_hash = pending_tx.tx_hash();

        if let Some(ws_client) = &ws_client {
            match ws_client.wait_for_transaction_confirmation(tx_hash).await {
                Ok(_) => {
                    let tx_hash_str = format!("{:?}", tx_hash);
                    return Ok(TransferResult { tx_hash: tx_hash_str });
                }
                Err(e) => {
                    eprintln!("Warning: WebSocket confirmation failed: {}, falling back to HTTP polling", e);
                }
            }
        }

        let receipt = pending_tx.confirmations(1).await
            .map_err(|e| anyhow::anyhow!("Failed to confirm transaction: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("Transaction confirmation timeout"))?;

        let tx_hash_str = format!("{:?}", receipt.transaction_hash);

        Ok(TransferResult { tx_hash: tx_hash_str })
    }
}

async fn check_eip1559_support<M: Middleware>(client: &M) -> Result<bool> {
    match client.get_block(BlockNumber::Latest).await {
        Ok(Some(block)) => Ok(block.base_fee_per_gas.is_some()),
        _ => Ok(true),
    }
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
