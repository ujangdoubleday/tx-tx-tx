use crate::networks::Network;
use crate::network::{HttpClient, WebSocketClient};
use alloy_dyn_abi::DynSolValue;
use alloy_primitives::Address;
use anyhow::{anyhow, Result};
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;

use super::abi::DynAbiFunction;
use super::codec::Codec;

pub struct ContractExecutor {
    http_client: HttpClient,
    ws_client: Option<WebSocketClient>,
    wallet: LocalWallet,
    network: Network,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub tx_hash: String,
}

#[derive(Debug, Clone)]
pub struct ReadResult {
    pub outputs: Vec<(String, String)>,
}

impl ContractExecutor {
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

        Ok(ContractExecutor {
            http_client,
            ws_client,
            wallet,
            network,
        })
    }

    pub async fn call_read_function(
        &self,
        contract_address: Address,
        function: &DynAbiFunction,
        args: &[DynSolValue],
        function_name: &str,
    ) -> Result<ReadResult> {
        let calldata = function.encode_call(function_name, args)?;
        
        let from_addr = self.wallet.address();
        let to_addr = H160::from_slice(&contract_address.to_vec()[..20]);
        
        let tx_request = TransactionRequest::new()
            .from(from_addr)
            .to(to_addr)
            .data(ethers::types::Bytes::from(calldata.to_vec()));
        
        let typed_tx = TypedTransaction::Legacy(tx_request.into());

        let result = self
            .http_client
            .get_provider()
            .call(&typed_tx, None)
            .await
            .map_err(|e| anyhow!("Failed to call contract: {}", e))?;

        let decoded_outputs = function.decode_output(&result)?;
        let outputs = function.get_outputs();

        let formatted_outputs = Codec::format_values(&decoded_outputs, &outputs);

        Ok(ReadResult {
            outputs: formatted_outputs,
        })
    }

    pub async fn call_write_function(
        &self,
        contract_address: Address,
        function: &DynAbiFunction,
        args: &[DynSolValue],
        function_name: &str,
    ) -> Result<ExecutionResult> {
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

        let wallet_client = SignerMiddleware::new(self.http_client.get_provider().clone(), self.wallet.clone());

        let pending_tx = wallet_client
            .send_transaction(tx, None)
            .await
            .map_err(|e| anyhow!("Failed to send transaction: {}", e))?;

        let tx_hash = pending_tx.tx_hash();

        if let Some(ws_client) = &self.ws_client {
            match ws_client.wait_for_transaction_confirmation(tx_hash).await {
                Ok(_) => {
                    let tx_hash_str = format!("{:?}", tx_hash);
                    return Ok(ExecutionResult { tx_hash: tx_hash_str });
                }
                Err(e) => {
                    eprintln!("Warning: WebSocket confirmation failed: {}, falling back to HTTP polling", e);
                }
            }
        }

        let receipt = pending_tx
            .confirmations(1)
            .await
            .map_err(|e| anyhow!("Failed to confirm transaction: {}", e))?
            .ok_or_else(|| anyhow!("Transaction confirmation timeout"))?;

        let tx_hash_str = format!("{:?}", receipt.transaction_hash);

        Ok(ExecutionResult { tx_hash: tx_hash_str })
    }
}
