use crate::networks::Network;
use alloy_dyn_abi::DynSolValue;
use alloy_primitives::Address;
use anyhow::{anyhow, Result};
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;

use super::abi::DynAbiFunction;
use super::codec::Codec;

pub struct ContractExecutor {
    client: Provider<Http>,
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
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| anyhow!("Failed to create provider: {}", e))?;

        let wallet: LocalWallet = private_key
            .parse()
            .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;

        let wallet = wallet.with_chain_id(network.chain_id);

        Ok(ContractExecutor {
            client: provider,
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
            .client
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
        
        // eprintln!("DEBUG: Function name: {}", function_name);
        // eprintln!("DEBUG: Calldata hex: 0x{}", hex::encode(calldata.as_ref()));
        // eprintln!("DEBUG: Calldata len: {}", calldata.len());
        
        let from = self.wallet.address();
        let to_addr = H160::from_slice(&contract_address.to_vec()[..20]);

        let gas_price = self
            .client
            .get_gas_price()
            .await
            .map_err(|e| anyhow!("Failed to get gas price: {}", e))?;

        let tx_request = TransactionRequest::new()
            .from(from)
            .to(to_addr)
            .data(ethers::types::Bytes::from(calldata.to_vec()));

        let typed_tx = TypedTransaction::Legacy(tx_request.into());
        
        let estimated_gas = self
            .client
            .estimate_gas(&typed_tx, None)
            .await
            .map_err(|e| anyhow!("Failed to estimate gas: {}", e))?;
        
        let gas_limit = (estimated_gas.as_u128() as f64 * 1.2) as u128;
        let gas_limit = U256::from(gas_limit);

        let tx = TransactionRequest::new()
            .from(from)
            .to(to_addr)
            .data(ethers::types::Bytes::from(calldata.to_vec()))
            .gas(gas_limit)
            .gas_price(gas_price)
            .chain_id(self.network.chain_id);

        let client = self.client.clone();
        let wallet_client = SignerMiddleware::new(client, self.wallet.clone());

        let pending_tx = wallet_client
            .send_transaction(tx, None)
            .await
            .map_err(|e| anyhow!("Failed to send transaction: {}", e))?;

        let receipt = pending_tx
            .confirmations(1)
            .await
            .map_err(|e| anyhow!("Failed to confirm transaction: {}", e))?
            .ok_or_else(|| anyhow!("Transaction confirmation timeout"))?;

        let tx_hash = format!("{:?}", receipt.transaction_hash);

        Ok(ExecutionResult { tx_hash })
    }
}
