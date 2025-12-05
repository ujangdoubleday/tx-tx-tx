use crate::artifact::{ArtifactLoader, ContractArtifact};
use anyhow::Result;
use ethers::prelude::*;
use ethers::types::Eip1559TransactionRequest;
use ethers::types::transaction::eip2718::TypedTransaction;
use x_core::gas::{GasEstimate, GasStrategy};
use x_core::networks::Network;
use x_core::network::{HttpClient, WebSocketClient};

pub struct ContractDeployer {
    http_client: HttpClient,
    ws_client: Option<WebSocketClient>,
    wallet: LocalWallet,
    network: Network,
}

#[derive(Debug, Clone)]
pub struct DeploymentResult {
    pub contract_address: Address,
    pub tx_hash: H256,
    pub gas_used: U256,
    pub gas_estimate: GasEstimate,
}

impl ContractDeployer {
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
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;

        let wallet = wallet.with_chain_id(network.chain_id);

        Ok(ContractDeployer {
            http_client,
            ws_client,
            wallet,
            network,
        })
    }

    pub async fn deploy(
        &self,
        artifact: &ContractArtifact,
        constructor_args: Option<Vec<u8>>,
        _gas_strategy: GasStrategy,
    ) -> Result<DeploymentResult> {
        let bytecode = ArtifactLoader::get_bytecode(artifact)?;
        
        let mut init_code = bytecode.to_vec();
        if let Some(args) = constructor_args {
            init_code.extend_from_slice(&args);
        }

        let init_code = Bytes::from(init_code);

        let from = self.wallet.address();

        let tx_request = TransactionRequest::new()
            .from(from)
            .data(init_code.clone());

        let typed_tx = TypedTransaction::Legacy(tx_request.into());

        let gas_limit = self.http_client
            .estimate_gas(&typed_tx)
            .await?;

        let gas_price = self.http_client
            .get_gas_price()
            .await?;

        let provider = self.http_client.get_provider();

        let mut tx = Eip1559TransactionRequest::new()
            .data(init_code)
            .gas(gas_limit)
            .chain_id(self.network.chain_id);

        let fee_history = provider
            .fee_history(1u64, BlockNumber::Latest, &[50.0])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch fee history: {}", e))?;

        if let Some(base_fee) = fee_history.base_fee_per_gas.last().copied() {
            let priority_fee = U256::from_dec_str("1000000000").unwrap_or(U256::from(1_000_000_000u64));
            let max_fee = base_fee + priority_fee;
            tx = tx
                .max_priority_fee_per_gas(priority_fee)
                .max_fee_per_gas(max_fee);
        }

        let wallet_client = SignerMiddleware::new(provider.clone(), self.wallet.clone());

        let pending_tx = wallet_client
            .send_transaction(tx, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

        let tx_hash = pending_tx.tx_hash();

        if let Some(ws_client) = &self.ws_client {
            match ws_client.wait_for_transaction_confirmation(tx_hash).await {
                Ok(_) => {
                    let receipt = provider
                        .get_transaction_receipt(tx_hash)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to get receipt: {}", e))?
                        .ok_or_else(|| anyhow::anyhow!("Receipt not found"))?;

                    let contract_address = receipt
                        .contract_address
                        .ok_or_else(|| anyhow::anyhow!("Failed to get contract address"))?;

                    let gas_used = receipt.gas_used.ok_or_else(|| {
                        anyhow::anyhow!("Failed to get gas used")
                    })?;

                    let gas_estimate = GasEstimate {
                        gas_price,
                        gas_limit,
                        max_priority_fee: None,
                        max_fee_per_gas: None,
                        invoker: Some(from),
                    };

                    return Ok(DeploymentResult {
                        contract_address,
                        tx_hash,
                        gas_used,
                        gas_estimate,
                    });
                }
                Err(e) => {
                    eprintln!("Warning: WebSocket confirmation failed: {}, falling back to HTTP polling", e);
                }
            }
        }

        let receipt = pending_tx.confirmations(1).await
            .map_err(|e| anyhow::anyhow!("Failed to confirm transaction: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("Transaction confirmation timeout"))?;

        let receipt_hash = receipt.transaction_hash;

        let contract_address = receipt
            .contract_address
            .ok_or_else(|| anyhow::anyhow!("Failed to get contract address"))?;

        let gas_used = receipt.gas_used.ok_or_else(|| {
            anyhow::anyhow!("Failed to get gas used")
        })?;

        let gas_estimate = GasEstimate {
            gas_price,
            gas_limit,
            max_priority_fee: None,
            max_fee_per_gas: None,
            invoker: Some(from),
        };

        Ok(DeploymentResult {
            contract_address,
            tx_hash: receipt_hash,
            gas_used,
            gas_estimate,
        })
    }
}
