use crate::networks::Network;
use crate::network::{HttpClient, WebSocketClient};
use crate::invoker::abi::DynAbiFunction;
use alloy_dyn_abi::DynSolValue;
use alloy_primitives::Address;
use anyhow::{anyhow, Result};
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct StressExecutor {
    http_client: HttpClient,
    ws_client: Option<WebSocketClient>,
    wallet: LocalWallet,
    network: Network,
}

#[derive(Debug, Clone)]
pub struct StressExecutionResult {
    pub tx_hash: String,
    pub index: usize,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StressConfig {
    pub total_transactions: Option<usize>,
    pub interval_ms: u64,
}

impl StressExecutor {
    pub async fn new(
        rpc_url: &str,
        private_key: &str,
        network: Network,
    ) -> Result<Self> {
        let http_client = HttpClient::new(rpc_url).await?;

        let ws_client = if !network.ws_rpc.is_empty() {
            Some(WebSocketClient::new(&network.ws_rpc[0]))
        } else {
            None
        };

        let wallet: LocalWallet = private_key
            .parse()
            .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;

        let wallet = wallet.with_chain_id(network.chain_id);

        Ok(StressExecutor {
            http_client,
            ws_client,
            wallet,
            network,
        })
    }

    pub async fn execute_stress_test(
        &self,
        contract_address: Address,
        function: &DynAbiFunction,
        args: &[DynSolValue],
        function_name: &str,
        config: StressConfig,
        on_progress: impl Fn(&StressExecutionResult),
    ) -> Result<Vec<StressExecutionResult>> {
        let mut results = Vec::new();
        let start_time = Instant::now();
        let mut transaction_index = 0;

        loop {
            if let Some(total) = config.total_transactions {
                if transaction_index >= total {
                    break;
                }
            }

            let result = self
                .execute_single_write_transaction(
                    contract_address,
                    function,
                    args,
                    function_name,
                    transaction_index,
                )
                .await;

            let stress_result = match result {
                Ok(tx_hash) => StressExecutionResult {
                    tx_hash,
                    index: transaction_index,
                    success: true,
                    error: None,
                },
                Err(e) => StressExecutionResult {
                    tx_hash: String::new(),
                    index: transaction_index,
                    success: false,
                    error: Some(e.to_string()),
                },
            };

            on_progress(&stress_result);
            results.push(stress_result);

            transaction_index += 1;

            if config.interval_ms > 0 && (config.total_transactions.is_none() || transaction_index < config.total_transactions.unwrap()) {
                sleep(Duration::from_millis(config.interval_ms)).await;
            }
        }

        let elapsed = start_time.elapsed();
        println!(
            "\n✅ Stress test completed: {} transactions in {:.2}s",
            results.len(),
            elapsed.as_secs_f64()
        );

        Ok(results)
    }

    async fn execute_single_write_transaction(
        &self,
        contract_address: Address,
        function: &DynAbiFunction,
        args: &[DynSolValue],
        function_name: &str,
        _tx_index: usize,
    ) -> Result<String> {
        let calldata = function.encode_call(function_name, args)?;

        let from = self.wallet.address();
        let to_addr = H160::from_slice(&contract_address.to_vec()[..20]);

        let gas_price = self.http_client.get_gas_price().await?;

        let tx_request = TransactionRequest::new()
            .from(from)
            .to(to_addr)
            .data(ethers::types::Bytes::from(calldata.to_vec()));

        let typed_tx = TypedTransaction::Legacy(tx_request.into());

        let estimated_gas = self.http_client.estimate_gas(&typed_tx).await?;

        let gas_limit = (estimated_gas.as_u128() as f64 * 1.2) as u128;
        let gas_limit = U256::from(gas_limit);

        let tx = TransactionRequest::new()
            .from(from)
            .to(to_addr)
            .data(ethers::types::Bytes::from(calldata.to_vec()))
            .gas(gas_limit)
            .gas_price(gas_price)
            .chain_id(self.network.chain_id);

        let wallet_client =
            SignerMiddleware::new(self.http_client.get_provider().clone(), self.wallet.clone());

        let pending_tx = wallet_client
            .send_transaction(tx, None)
            .await
            .map_err(|e| anyhow!("Failed to send transaction: {}", e))?;

        let tx_hash = pending_tx.tx_hash();

        if let Some(ws_client) = &self.ws_client {
            match ws_client.wait_for_transaction_confirmation(tx_hash).await {
                Ok(_) => {
                    return Ok(format!("{:?}", tx_hash));
                }
                Err(e) => {
                    eprintln!(
                        "Warning: WebSocket confirmation failed: {}, falling back to HTTP polling",
                        e
                    );
                }
            }
        }

        let receipt = pending_tx
            .confirmations(1)
            .await
            .map_err(|e| anyhow!("Failed to confirm transaction: {}", e))?
            .ok_or_else(|| anyhow!("Transaction confirmation timeout"))?;

        Ok(format!("{:?}", receipt.transaction_hash))
    }
}
