use colored::Colorize;
use inquire::{Text, Select};
use x_core as core;
use x_core::invoker::{DeploymentManager, ContractInvoker, Codec};
use alloy_dyn_abi::DynSolValue;

use super::utils::{clear_screen, print_separator, print_line};

pub fn handle_smart_contract_invoker(network_id: &str) -> anyhow::Result<()> {
    println!("{}", "📋 SMART CONTRACT INVOKER".cyan().bold());
    println!();

    let deployments_file = "deployments/testnet_sepolia.json";
    let artifact_dir = "artifacts";

    print!("{}", "Loading deployed contracts... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    let records = DeploymentManager::load_deployments(deployments_file)?;
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

    let function_type_options = vec!["1. Read (View/Pure)", "2. Write (State Changing)", "3. Back"];
    let function_type_selected = Select::new("Select function type:", function_type_options)
        .prompt()
        .map_err(|_| anyhow::anyhow!("Function type selection cancelled"))?;

    if function_type_selected.contains("Back") {
        return Err(anyhow::anyhow!("__BACK__"));
    }

    let is_read = function_type_selected.contains("Read");

    let invoker = ContractInvoker::new(deployments_file, artifact_dir);
    let contract_invoker = invoker.get_contract(&selected_record.contract_name, network_id)?;

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
    
    print!("{}", "Calling contract... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

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

    println!("{}", "✓".green().bold());

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
    
    print!("{}", "Sending transaction... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

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

    println!("{}", "✓".green().bold());

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
