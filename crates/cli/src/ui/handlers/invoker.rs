use colored::Colorize;
use inquire::{Text, Select};
use x_core as core;
use x_core::invoker::{DeploymentManager, ContractInvoker, Codec};
use x_core::stress::{StressExecutor, StressConfig};
use alloy_dyn_abi::DynSolValue;
use std::sync::{Arc, Mutex};

use super::utils::{clear_screen, print_separator, print_line};
use crate::ui::loading::{create_spinner, finish_spinner};

pub fn handle_smart_contract_invoker(network_id: &str) -> anyhow::Result<()> {
    println!("{}", "📋 SMART CONTRACT INVOKER".cyan().bold());
    println!();

    let deployments_file = format!("deployments/{}.json", network_id);
    let artifact_dir = "artifacts";

    print!("{}", "Loading deployed contracts... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    let records = DeploymentManager::load_deployments(&deployments_file)?;
    let filtered_records: Vec<_> = records.iter()
        .filter(|r| r.network == network_id)
        .collect();

    if filtered_records.is_empty() {
        anyhow::bail!("No deployed contracts found for this network");
    }

    println!("{}", "✓".green().bold());
    println!();

    let mut display_contracts = Vec::new();
    for (i, record) in filtered_records.iter().enumerate() {
        let short_addr = if record.address.len() > 10 {
            format!("{}...{}", &record.address[..6], &record.address[record.address.len()-4..])
        } else {
            record.address.clone()
        };
        display_contracts.push(format!(
            "{}. {} - {}",
            i + 1,
            record.contract_name,
            short_addr
        ));
    }
    let back_num = filtered_records.len() + 1;
    let quit_num = filtered_records.len() + 2;
    display_contracts.push(format!("{}. Back", back_num));
    display_contracts.push(format!("{}. Quit", quit_num));

    let selected_display = Select::new("Select a contract:", display_contracts)
        .prompt()
        .map_err(|_| anyhow::anyhow!("Contract selection cancelled"))?;

    if selected_display.contains("Back") {
        return Err(anyhow::anyhow!("__BACK__"));
    }

    if selected_display.contains("Quit") {
        clear_screen();
        println!("{}", "👋 Goodbye!".green().bold());
        std::process::exit(0);
    }

    let selected_idx = selected_display
        .split(". ")
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| anyhow::anyhow!("Invalid selection"))?
        - 1;

    let selected_record = filtered_records[selected_idx];

    println!();
    println!("{}", format!("Selected: {} ({})", selected_record.contract_name, selected_record.address).cyan().bold());
    println!();

    let function_type_options = vec!["1. Read (View/Pure)", "2. Write (State Changing)", "3. Stress Mode", "4. Back"];
    let function_type_selected = Select::new("Select function type:", function_type_options)
        .prompt()
        .map_err(|_| anyhow::anyhow!("Function type selection cancelled"))?;

    if function_type_selected.contains("Back") {
        return Err(anyhow::anyhow!("__BACK__"));
    }

    let is_read = function_type_selected.contains("Read");
    let is_stress = function_type_selected.contains("Stress");

    let invoker = ContractInvoker::new(&deployments_file, artifact_dir);
    let contract_invoker = invoker.get_contract_by_address(&selected_record.contract_name, &selected_record.address, network_id)?;

    println!();
    let all_functions = contract_invoker.get_all_functions()?;

    let mut display_functions = Vec::new();
    for (i, func) in all_functions.iter().enumerate() {
        display_functions.push(format!("{}. {}", i + 1, func));
    }
    display_functions.push(format!("{}. Back", all_functions.len() + 1));

    let selected_func_display = Select::new("Select a function:", display_functions)
        .prompt()
        .map_err(|_| anyhow::anyhow!("Function selection cancelled"))?;

    if selected_func_display.contains("Back") {
        return Err(anyhow::anyhow!("__BACK__"));
    }

    let selected_func = selected_func_display
        .split(". ")
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Invalid function selection"))?
        .to_string();

    let (inputs, outputs) = contract_invoker.get_function_info(&selected_func)?;

    println!();
    println!("{}", format!("Function: {}", selected_func).cyan().bold());
    println!("{}", "Inputs:".cyan().bold());
    for (name, ty) in &inputs {
        println!("  - {}: {}", name, ty);
    }
    println!("{}", "Outputs:".cyan().bold());
    for (name, ty) in &outputs {
        println!("  - {}: {}", name, ty);
    }
    println!();

    let mut dyn_args = Vec::new();

    if !inputs.is_empty() {
        println!("{}", "Enter function arguments (or leave empty for no args):".cyan().bold());

        let mut args = Vec::new();
        for (name, ty) in &inputs {
            let input_val = Text::new(&format!("Enter {} ({}): ", name, ty))
                .prompt()
                .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

            if !input_val.trim().is_empty() {
                args.push((name.clone(), ty.clone(), input_val));
            }
        }

        if !args.is_empty() {
            println!();
            println!("{}", "Arguments entered:".cyan().bold());
            for (name, ty, val) in &args {
                println!("  - {}: {} = {}", name, ty, val);
            }

            for (_name, ty, val) in &args {
                let parsed_val = Codec::parse_value(val, ty)?;
                dyn_args.push(parsed_val);
            }
        }
    }

    println!();
    if is_read {
        handle_read_function(&contract_invoker, &selected_record, &selected_func, &dyn_args, network_id)?;
    } else if is_stress {
        handle_stress_mode(&contract_invoker, &selected_record, &selected_func, &dyn_args, network_id)?;
    } else {
        handle_write_function(&contract_invoker, &selected_record, &selected_func, &dyn_args, network_id)?;
    }

    Ok(())
}

fn handle_read_function(
    contract_invoker: &x_core::invoker::DeployedContractInvoker,
    selected_record: &x_core::invoker::DeploymentRecord,
    selected_func: &str,
    dyn_args: &[DynSolValue],
    network_id: &str,
) -> anyhow::Result<()> {
    let networks = core::networks::load_networks()?;
    let network = core::networks::get_network_by_id(&networks, network_id)
        .ok_or_else(|| anyhow::anyhow!("Network not found"))?;

    let private_key = core::config::load_private_key()
        .map_err(|_| anyhow::anyhow!("Failed to load private key from .env"))?;

    let rpc_url = network.rpc.first()
        .ok_or_else(|| anyhow::anyhow!("No RPC URL available for network"))?;

    let dyn_func = contract_invoker.get_function_abi(selected_func)?;
    let _encoded = dyn_func.encode_call(selected_func, &dyn_args)?;
    
    let spinner = create_spinner("Calling contract...");

    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(async {
        contract_invoker.execute_read_function(
            rpc_url,
            &private_key,
            network,
            selected_func,
            dyn_args,
        ).await
    })?;

    finish_spinner(spinner, "Calling contract... ");

    println!();
    println!("{}", "✅ READ FUNCTION RESULT".green().bold());
    print_line("Contract", &selected_record.contract_name, |s| s.normal());
    print_line("Function", selected_func, |s| s.yellow());
    print_line("Address", &selected_record.address, |s| s.cyan());
    
    println!("{}", "Returns:".cyan().bold());
    for (name, val) in &result.outputs {
        println!("  - {}: {}", name, val.green());
    }

    print_separator();
    println!();

    Ok(())
}

fn handle_write_function(
    contract_invoker: &x_core::invoker::DeployedContractInvoker,
    selected_record: &x_core::invoker::DeploymentRecord,
    selected_func: &str,
    dyn_args: &[DynSolValue],
    network_id: &str,
) -> anyhow::Result<()> {
    let networks = core::networks::load_networks()?;
    let network = core::networks::get_network_by_id(&networks, network_id)
        .ok_or_else(|| anyhow::anyhow!("Network not found"))?;

    let private_key = core::config::load_private_key()
        .map_err(|_| anyhow::anyhow!("Failed to load private key from .env"))?;

    let rpc_url = network.rpc.first()
        .ok_or_else(|| anyhow::anyhow!("No RPC URL available for network"))?;

    let dyn_func = contract_invoker.get_function_abi(selected_func)?;
    let _encoded = dyn_func.encode_call(selected_func, &dyn_args)?;
    
    let spinner = create_spinner("Sending transaction...");

    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(async {
        contract_invoker.execute_write_function(
            rpc_url,
            &private_key,
            network,
            selected_func,
            dyn_args,
        ).await
    })?;

    finish_spinner(spinner, "Sending transaction... ");

    println!();
    println!("{}", "✅ TRANSACTION SUCCESSFUL".green().bold());
    print_line("Contract", &selected_record.contract_name, |s| s.normal());
    print_line("Function", selected_func, |s| s.yellow());
    print_line("Address", &selected_record.address, |s| s.cyan());
    print_line("Tx Hash", &result.tx_hash, |s| s.green());
    print_line("Block Explorer", &format!("{}/tx/{}", network.block_explorer.url, result.tx_hash), |s| s.blue());

    print_separator();
    println!();

    Ok(())
}

fn handle_stress_mode(
    contract_invoker: &x_core::invoker::DeployedContractInvoker,
    selected_record: &x_core::invoker::DeploymentRecord,
    selected_func: &str,
    dyn_args: &[DynSolValue],
    network_id: &str,
) -> anyhow::Result<()> {
    let networks = core::networks::load_networks()?;
    let network = core::networks::get_network_by_id(&networks, network_id)
        .ok_or_else(|| anyhow::anyhow!("Network not found"))?;

    let private_key = core::config::load_private_key()
        .map_err(|_| anyhow::anyhow!("Failed to load private key from .env"))?;

    let rpc_url = network.rpc.first()
        .ok_or_else(|| anyhow::anyhow!("No RPC URL available for network"))?;

    println!();
    println!("{}", "⚙️  STRESS MODE CONFIGURATION".cyan().bold());
    println!();

    let total_tx_str = Text::new("Enter number of transactions (leave empty for unlimited): ")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    let total_transactions = if total_tx_str.trim().is_empty() {
        None
    } else {
        Some(total_tx_str.trim().parse::<usize>()
            .map_err(|_| anyhow::anyhow!("Invalid number format"))?)
    };

    let interval_str = Text::new("Enter interval between transactions in milliseconds (default: 0 - send immediately after success): ")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    let interval_ms = if interval_str.trim().is_empty() {
        0
    } else {
        interval_str.trim().parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid number format"))?
    };

    println!();
    println!("{}", format!(
        "📊 Stress Test Configuration: {} transactions, {} ms interval",
        total_transactions.map(|n| n.to_string()).unwrap_or_else(|| "unlimited".to_string()),
        if interval_ms == 0 { "immediate after success".to_string() } else { interval_ms.to_string() }
    ).cyan().bold());
    println!();

    let config = StressConfig {
        total_transactions,
        interval_ms,
    };

    let dyn_func = contract_invoker.get_function_abi(selected_func)?;
    let _encoded = dyn_func.encode_call(selected_func, &dyn_args)?;

    let rt = tokio::runtime::Runtime::new()?;

    let spinner = create_spinner("Initializing stress test...");

    let executor = rt.block_on(async {
        StressExecutor::new(rpc_url, &private_key, network.clone()).await
    })?;

    finish_spinner(spinner, "Initializing stress test... ");

    println!();

    let contract_address = contract_invoker.address()?;
    let dyn_func = contract_invoker.get_function_abi(selected_func)?;

    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = Arc::clone(&results);

    let on_progress = move |result: &x_core::stress::StressExecutionResult| {
        let status = if result.success {
            format!("✅ Tx #{}: {}", result.index + 1, &result.tx_hash[..std::cmp::min(16, result.tx_hash.len())]).green().to_string()
        } else {
            format!("❌ Tx #{}: {}", result.index + 1, result.error.as_ref().unwrap_or(&"Unknown error".to_string())).red().to_string()
        };
        println!("{}", status);

        results_clone.lock().unwrap().push(result.clone());
    };

    let spinner = create_spinner("Running stress test...");

    let stress_results = rt.block_on(async {
        executor.execute_stress_test(
            contract_address,
            &dyn_func,
            dyn_args,
            selected_func,
            config,
            on_progress,
        ).await
    })?;

    finish_spinner(spinner, "Running stress test...");

    println!();
    println!("{}", "📋 STRESS TEST SUMMARY".cyan().bold());
    println!();

    let successful = stress_results.iter().filter(|r| r.success).count();
    let failed = stress_results.iter().filter(|r| !r.success).count();

    print_line("Total Transactions", &stress_results.len().to_string(), |s| s.normal());
    print_line("Successful", &successful.to_string(), |s| s.green());
    print_line("Failed", &failed.to_string(), |s| s.red());
    print_line("Contract", &selected_record.contract_name, |s| s.normal());
    print_line("Function", selected_func, |s| s.yellow());
    print_line("Address", &selected_record.address, |s| s.cyan());

    if !stress_results.is_empty() {
        println!();
        println!("{}", "Transaction Hashes:".cyan().bold());
        for result in stress_results.iter().take(10) {
            if result.success {
                println!("  {} - {}", result.index + 1, result.tx_hash.green());
            } else {
                println!("  {} - {} ({})", result.index + 1, "Failed".red(), result.error.as_ref().unwrap_or(&"Unknown".to_string()));
            }
        }
        if stress_results.len() > 10 {
            println!("  ... and {} more transactions", stress_results.len() - 10);
        }
    }

    print_separator();
    println!();

    Ok(())
}
