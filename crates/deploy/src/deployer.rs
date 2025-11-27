use crate::artifact::{ArtifactLoader, ContractArtifact};
use anyhow::Result;
use ethers::prelude::*;
use ethers::types::Eip1559TransactionRequest;
use ethers::types::transaction::eip2718::TypedTransaction;
use x_core::gas::{GasEstimate, GasStrategy};
use x_core::networks::Network;

pub struct ContractDeployer {
    client: Provider<Http>,
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
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| anyhow::anyhow!("Failed to create provider: {}", e))?;

        let wallet: LocalWallet = private_key
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;

        let wallet = wallet.with_chain_id(network.chain_id);

        Ok(ContractDeployer {
            client: provider,
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

        let gas_limit = self.client
            .estimate_gas(&typed_tx, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to estimate gas: {}", e))?;

        let gas_price = self.client
            .get_gas_price()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get gas price: {}", e))?;

        let mut tx = Eip1559TransactionRequest::new()
            .data(init_code)
            .gas(gas_limit)
            .chain_id(self.network.chain_id);

        let fee_history = self.client
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

        let client = self.client.clone();
        let wallet_client = SignerMiddleware::new(client, self.wallet.clone());

        let pending_tx = wallet_client
            .send_transaction(tx, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

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
        };

        Ok(DeploymentResult {
            contract_address,
            tx_hash: receipt_hash,
            gas_used,
            gas_estimate,
        })
    }
}
