use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum GasStrategy {
    Low,
    Standard,
    Fast,
    Instant,
}

#[derive(Debug, Clone)]
pub struct GasEstimate {
    pub gas_price: U256,
    pub gas_limit: U256,
    pub max_priority_fee: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
}

pub struct GasCalculator;

impl GasCalculator {
    pub async fn estimate_gas<M: Middleware>(
        client: &M,
        from: Address,
        to: Address,
        value: U256,
        data: Option<Vec<u8>>,
        strategy: GasStrategy,
    ) -> Result<GasEstimate> {
        let tx = TransactionRequest::new()
            .from(from)
            .to(to)
            .value(value)
            .data(data.unwrap_or_default());

        let typed_tx = TypedTransaction::Legacy(tx.into());

        let gas_limit = client
            .estimate_gas(&typed_tx, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to estimate gas: {}", e))?;

        let gas_limit = (gas_limit.as_u128() as f64 * 1.2) as u128;
        let gas_limit = U256::from(gas_limit);

        let fee_history = client
            .fee_history(10u64, BlockNumber::Latest, &[50.0])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch fee history: {}", e))?;

        let base_fee = fee_history
            .base_fee_per_gas
            .last()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("No base fee available"))?;

        let priority_fee = calculate_priority_fee(&fee_history, strategy);

        let max_fee_per_gas = base_fee + priority_fee;

        Ok(GasEstimate {
            gas_price: U256::zero(),
            gas_limit,
            max_priority_fee: Some(priority_fee),
            max_fee_per_gas: Some(max_fee_per_gas),
        })
    }

    pub async fn estimate_gas_legacy<M: Middleware>(
        client: &M,
        from: Address,
        to: Address,
        value: U256,
        data: Option<Vec<u8>>,
        strategy: GasStrategy,
    ) -> Result<GasEstimate> {
        let tx = TransactionRequest::new()
            .from(from)
            .to(to)
            .value(value)
            .data(data.unwrap_or_default());

        let typed_tx = TypedTransaction::Legacy(tx.into());

        let gas_limit = client
            .estimate_gas(&typed_tx, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to estimate gas: {}", e))?;

        let gas_limit = (gas_limit.as_u128() as f64 * 1.2) as u128;
        let gas_limit = U256::from(gas_limit);

        let gas_price = client
            .get_gas_price()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch gas price: {}", e))?;

        let gas_price = apply_strategy_multiplier(gas_price, strategy);

        Ok(GasEstimate {
            gas_price,
            gas_limit,
            max_priority_fee: None,
            max_fee_per_gas: None,
        })
    }
}

fn calculate_priority_fee(fee_history: &FeeHistory, strategy: GasStrategy) -> U256 {
    let priority_fees: Vec<U256> = fee_history
        .reward
        .iter()
        .flatten()
        .copied()
        .collect();

    if priority_fees.is_empty() {
        return match strategy {
            GasStrategy::Low => U256::from_dec_str("1000000000").unwrap_or(U256::from(1_000_000_000u64)),
            GasStrategy::Standard => U256::from_dec_str("2000000000").unwrap_or(U256::from(2_000_000_000u64)),
            GasStrategy::Fast => U256::from_dec_str("5000000000").unwrap_or(U256::from(5_000_000_000u64)),
            GasStrategy::Instant => U256::from_dec_str("10000000000").unwrap_or(U256::from(10_000_000_000u64)),
        };
    }

    let percentile = match strategy {
        GasStrategy::Low => 25,
        GasStrategy::Standard => 50,
        GasStrategy::Fast => 75,
        GasStrategy::Instant => 95,
    };

    let index = (priority_fees.len() * percentile) / 100;
    priority_fees[index.min(priority_fees.len() - 1)]
}

fn apply_strategy_multiplier(gas_price: U256, strategy: GasStrategy) -> U256 {
    let multiplier = match strategy {
        GasStrategy::Low => 1.0,
        GasStrategy::Standard => 1.1,
        GasStrategy::Fast => 1.5,
        GasStrategy::Instant => 2.0,
    };

    let adjusted = (gas_price.as_u128() as f64 * multiplier) as u128;
    U256::from(adjusted)
}
