use x_core::config;
use x_core::invoker::ContractInvoker;
use x_core::invoker::Codec;
use x_core::stress::{StressExecutor, StressConfig};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub async fn handle_invoke_stress(
    contract: String,
    network: String,
    function: String,
    args: String,
    transactions: usize,
    interval: u64,
) -> anyhow::Result<()> {
    let private_key = config::load_private_key()?;
    let networks = x_core::networks::load_networks()?;
    let network_obj = x_core::networks::get_network_by_id(&networks, &network)
        .ok_or_else(|| anyhow::anyhow!("Network '{}' not found", network))?;

    let deployments_file = format!("deployments/{}.json", network);
    let invoker = ContractInvoker::new(&deployments_file, "artifacts");
    let contract_invoker = invoker.get_contract(&contract, &network)
        .or_else(|_| {
            eprintln!("Attempting to use contract address directly...");
            anyhow::bail!("Contract '{}' not found in {}", contract, deployments_file)
        })?;

    println!("\n📋 Contract Invoker Stress Test");
    println!("  Contract: {}", contract_invoker.contract_name());
    println!("  Network: {}", contract_invoker.network());
    println!("  Address: {:?}", contract_invoker.address()?);
    println!("  Function: {}", function);
    println!("  Transactions: {}", transactions);
    println!("  Interval: {}ms\n", interval);

    let (inputs, _outputs) = contract_invoker.get_function_info(&function)?;

    let parsed_args = if args.is_empty() {
        Vec::new()
    } else {
        let arg_values: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
        
        if arg_values.len() != inputs.len() {
            anyhow::bail!(
                "Expected {} arguments, got {}",
                inputs.len(),
                arg_values.len()
            );
        }

        let mut parsed = Vec::new();
        for (i, (arg_val, (_, arg_type))) in arg_values.iter().zip(&inputs).enumerate() {
            match Codec::parse_value(arg_val, arg_type) {
                Ok(val) => parsed.push(val),
                Err(e) => {
                    anyhow::bail!("Failed to parse argument {}: {}", i, e);
                }
            }
        }
        parsed
    };

    let rpc_url = network_obj.rpc.first()
        .ok_or_else(|| anyhow::anyhow!("No RPC URL available for network"))?;

    let stress_executor = StressExecutor::new(rpc_url, &private_key, network_obj.clone()).await?;
    let contract_address = contract_invoker.address()?;
    let dyn_func = contract_invoker.get_function_abi(&function)?;

    let stress_config = StressConfig {
        total_transactions: Some(transactions),
        interval_ms: interval,
    };

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let results = stress_executor.execute_stress_test(
        contract_address,
        &dyn_func,
        &parsed_args,
        &function,
        stress_config,
        move |result| {
            let total = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
            if result.success {
                println!("  [✓ {}/{}] TX: {}", total, transactions, &result.tx_hash[..std::cmp::min(16, result.tx_hash.len())]);
            } else {
                println!("  [✗ {}/{}] Error: {}", total, transactions, result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
            }
        },
    ).await?;

    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    println!("\n📊 Stress Test Results:");
    println!("  Total Transactions: {}", results.len());
    println!("  Successful: {}", successful);
    println!("  Failed: {}", failed);
    println!("  Success Rate: {:.2}%", (successful as f64 / results.len() as f64) * 100.0);

    Ok(())
}
