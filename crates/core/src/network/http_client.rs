use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::providers::Middleware;
use anyhow::{anyhow, Result};

#[derive(Clone)]
pub struct HttpClient {
    provider: Provider<Http>,
}

impl HttpClient {
    pub async fn new(rpc_url: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| anyhow!("Failed to create HTTP provider: {}", e))?;

        Ok(HttpClient { provider })
    }

    pub fn get_provider(&self) -> &Provider<Http> {
        &self.provider
    }

    pub async fn get_gas_price(&self) -> Result<U256> {
        Middleware::get_gas_price(&self.provider)
            .await
            .map_err(|e| anyhow!("Failed to get gas price: {}", e))
    }

    pub async fn estimate_gas(&self, tx: &TypedTransaction) -> Result<U256> {
        Middleware::estimate_gas(&self.provider, tx, None)
            .await
            .map_err(|e| anyhow!("Failed to estimate gas: {}", e))
    }

    pub async fn get_transaction_receipt(&self, tx_hash: H256) -> Result<Option<TransactionReceipt>> {
        Middleware::get_transaction_receipt(&self.provider, tx_hash)
            .await
            .map_err(|e| anyhow!("Failed to get transaction receipt: {}", e))
    }

    pub async fn send_raw_transaction(&self, tx: &Bytes) -> Result<TxHash> {
        Middleware::send_raw_transaction(&self.provider, tx.clone())
            .await
            .map(|pending_tx| *pending_tx)
            .map_err(|e| anyhow!("Failed to send raw transaction: {}", e))
    }
}
