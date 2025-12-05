use ethers::prelude::*;
use anyhow::{anyhow, Result};
use tokio::time::{sleep, Duration};

pub struct WebSocketClient {
    ws_url: String,
}

impl WebSocketClient {
    pub fn new(ws_url: &str) -> Self {
        WebSocketClient {
            ws_url: ws_url.to_string(),
        }
    }

    pub async fn wait_for_transaction_confirmation(&self, tx_hash: H256) -> Result<()> {
        let mut retries = 0;
        let max_retries = 120;
        let poll_interval = Duration::from_millis(300);

        loop {
            match self.get_transaction_receipt(tx_hash).await {
                Ok(Some(_receipt)) => {
                    return Ok(());
                }
                Ok(None) => {
                    retries += 1;
                    if retries >= max_retries {
                        return Err(anyhow!("Transaction confirmation timeout"));
                    }
                    sleep(poll_interval).await;
                }
                Err(e) => {
                    retries += 1;
                    if retries >= max_retries {
                        return Err(anyhow!("Failed to get transaction receipt: {}", e));
                    }
                    sleep(poll_interval).await;
                }
            }
        }
    }

    async fn get_transaction_receipt(&self, tx_hash: H256) -> Result<Option<TransactionReceipt>> {
        self.try_ws_request(tx_hash).await.or_else(|_| {
            Err(anyhow!("WebSocket request failed"))
        })
    }

    async fn try_ws_request(&self, tx_hash: H256) -> Result<Option<TransactionReceipt>> {
        let http_provider = Provider::<Http>::try_from(&self.ws_url.replace("wss://", "https://").replace("ws://", "http://"))
            .map_err(|e| anyhow!("Failed to create fallback HTTP provider: {}", e))?;

        http_provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|e| anyhow!("Failed to get transaction receipt: {}", e))
    }

    pub fn get_ws_url(&self) -> &str {
        &self.ws_url
    }
}
