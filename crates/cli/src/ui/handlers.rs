use colored::Colorize;
use inquire::{Text, Select};
use std::io::{self, Write};
use x_core as core;
use x_core::compiler::SmartContractCompiler;
use x_signature;
use x_transfer;
use x_gate;
use x_deploy;
use x_core::gas::GasStrategy;
use x_core::invoker::{DeploymentManager, ContractInvoker, Codec};
use alloy_dyn_abi::DynSolValue;

const WIDTH: usize = 80;

pub fn clear_screen() {
    print!("\x1B[2J\x1B[3J\x1B[1;1H");
    std::io::Write::flush(&mut std::io::stdout()).ok();
}

fn print_separator() {
    println!("{}", "─".repeat(WIDTH));
}

fn print_line(label: &str, value: &str, color_fn: fn(&str) -> colored::ColoredString) {
    let label_width = 10;
    println!("  {:<width$} {}", format!("{}:", label).bold(), color_fn(value), width = label_width);
}

fn read_input_line(prompt: &str) -> anyhow::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn handle_sign() -> anyhow::Result<()> {
    println!("{}", "📝 SIGN MESSAGE".cyan().bold());

    let message = Text::new("Enter message to sign:")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    if message.trim().is_empty() {
        anyhow::bail!("Message cannot be empty");
    }

    print!("{}", "Generating signature... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    let key = x_core::config::load_private_key()
        .map_err(|_| anyhow::anyhow!("Failed to load private key from .env"))?;

    let signature = x_signature::sign_message(&key, message.trim())?;
    let address = x_signature::get_address_from_private_key(&key)?;

    println!("{}", "✓".green().bold());

    println!("\n{}", "✅ SIGNATURE GENERATED".green().bold());
    print_line("Message", message.trim(), |s| s.normal());
    print_line("Signature", &signature, |s| s.yellow());
    print_line("Address", &format!("{:#x}", address), |s| s.yellow());
    print_separator();
    println!();

    Ok(())
}

pub fn handle_verify() -> anyhow::Result<()> {
    println!("{}", "✓ VERIFY MESSAGE".cyan().bold());

    let message = Text::new("Enter the message:")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    if message.trim().is_empty() {
        anyhow::bail!("Message cannot be empty");
    }

    let signature = read_input_line("Enter the signature (with or without 0x): ")?;

    if signature.is_empty() {
        anyhow::bail!("Signature cannot be empty");
    }

    let address = Text::new("Enter the address (with or without 0x):")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    if address.trim().is_empty() {
        anyhow::bail!("Address cannot be empty");
    }

    print!("{}", "Verifying signature... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    let expected_addr = core::crypto::normalize_address(address.trim())?;
    let addr_bytes = core::crypto::hex_to_bytes(&expected_addr)?;
    let expected_address = ethers::types::Address::from_slice(&addr_bytes);

    let is_valid = match x_signature::verify_message(&signature, message.trim(), expected_address) {
        Ok(_) => true,
        Err(_) => false,
    };

    if is_valid {
        println!("{}", "✓".green().bold());
        println!("\n{}", "✅ SIGNATURE IS VALID".green().bold());
    } else {
        println!("{}", "✗".red().bold());
        println!("\n{}", "❌ SIGNATURE IS INVALID".red().bold());
    }

    print_line("Message", message.trim(), |s| s.cyan());
    print_line("Signature", &signature, |s| s.yellow());
    print_line("Address", &format!("{:#x}", expected_address), |s| s.cyan());
    print_separator();
    println!();

    Ok(())
}

pub fn handle_transfer_sepolia() -> anyhow::Result<()> {
    println!("{}", "💸 TRANSFER ETH ON SEPOLIA".cyan().bold());

    let amount = Text::new("Enter amount in ETH (e.g., 1 for 1 ETH, 0.01 for 0.01 ETH):")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    let amount: f64 = amount.trim().parse()
        .map_err(|_| anyhow::anyhow!("Invalid amount format"))?;

    if amount <= 0.0 {
        anyhow::bail!("Amount must be greater than 0");
    }

    let to_address = Text::new("Enter recipient address (with or without 0x):")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    if to_address.trim().is_empty() {
        anyhow::bail!("Recipient address cannot be empty");
    }

    let notes = Text::new("Enter notes (optional):")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    print!("{}", "Processing transfer... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    let key = core::config::load_private_key()
        .map_err(|_| anyhow::anyhow!("Failed to load private key from .env"))?;

    let networks = core::networks::load_networks()?;
    let network = core::networks::get_network_by_id(&networks, "testnet_sepolia")
        .ok_or_else(|| anyhow::anyhow!("Sepolia network not found"))?;

    let notes_opt = if notes.trim().is_empty() { None } else { Some(notes.as_str()) };
    let result = x_transfer::transfer_eth(&key, &to_address, amount, network, notes_opt)?;

    println!("{}", "✓".green().bold());

    println!("\n{}", "✅ TRANSFER SUCCESSFUL".green().bold());
    print_line("Amount", &format!("{} ETH", amount), |s| s.normal());
    print_line("To", &to_address, |s| s.yellow());
    print_line("Network", &network.name, |s| s.cyan());
    print_line("Tx Hash", &result.tx_hash, |s| s.green());
    print_line("Block Explorer", &format!("{}/tx/{}", network.block_explorer.url, result.tx_hash), |s| s.blue());
    if !notes.trim().is_empty() {
        print_line("Notes", notes.trim(), |s| s.normal());
    }
    print_separator();
    println!();

    Ok(())
}

pub fn handle_compile_smart_contracts() -> anyhow::Result<()> {
    println!("{}", "🔨 COMPILE SMART CONTRACTS".cyan().bold());
    println!();

    print!("{}", "Compiling all smart contracts... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    SmartContractCompiler::compile_all()?;

    println!("\n{}", "✅ COMPILATION SUCCESSFUL".green().bold());
    print_separator();
    println!();

    Ok(())
}

pub fn handle_gate_mainnet() -> anyhow::Result<()> {
    println!("{}", "🌐 THE GATE - ETHEREUM MAINNET".cyan().bold());
    println!();

    let networks = core::networks::load_networks()?;
    let network = core::networks::get_network_by_id(&networks, "ethereum_mainnet")
        .ok_or_else(|| anyhow::anyhow!("Ethereum Mainnet network not found"))?;

    println!("{}", "Available Features:".cyan().bold());
    for feature in x_gate::Gate::get_features().iter() {
        println!("  - {}", feature.to_string());
    }
    
    print_line("Network", &network.name, |s| s.cyan());
    print_separator();
    println!();

    Ok(())
}

pub fn handle_gate_sepolia() -> anyhow::Result<()> {
    println!("{}", "🌐 THE GATE - ETHEREUM SEPOLIA".cyan().bold());
    println!();

    let networks = core::networks::load_networks()?;
    let network = core::networks::get_network_by_id(&networks, "testnet_sepolia")
        .ok_or_else(|| anyhow::anyhow!("Ethereum Sepolia network not found"))?;

    println!("{}", "Available Features:".cyan().bold());
    for feature in x_gate::Gate::get_features().iter() {
        println!("  - {}", feature.to_string());
    }
    
    print_line("Network", &network.name, |s| s.cyan());
    print_separator();
    println!();

    Ok(())
}

pub fn handle_gate_deploy(network_id: &str) -> anyhow::Result<()> {
    println!("{}", "🚀 DEPLOY SMART CONTRACT".cyan().bold());
    println!();

    let networks = core::networks::load_networks()?;
    let network = core::networks::get_network_by_id(&networks, network_id)
        .ok_or_else(|| anyhow::anyhow!("Network '{}' not found", network_id))?;

    print!("{}", "Loading available contracts... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    let contracts = x_gate::Gate::get_available_contracts()?;
    println!("{}", "✓".green().bold());

    let mut display_contracts = Vec::new();
    for (i, contract) in contracts.iter().enumerate() {
        display_contracts.push(format!("{}. {}", i + 1, contract));
    }
    let back_num = contracts.len() + 1;
    let quit_num = contracts.len() + 2;
    display_contracts.push(format!("{}. Back", back_num));
    display_contracts.push(format!("{}. Quit", quit_num));

    let selected_display = Select::new("Select a contract to deploy:", display_contracts)
        .prompt()
        .map_err(|_| anyhow::anyhow!("Contract selection cancelled"))?;

    let selected_contract = if selected_display.contains("Back") {
        "Back".to_string()
    } else if selected_display.contains("Quit") {
        "Quit".to_string()
    } else {
        selected_display
            .split(". ")
            .nth(1)
            .unwrap_or("")
            .to_string()
    };

    if selected_contract == "Back" {
        return Err(anyhow::anyhow!("__BACK__"));
    }
    
    if selected_contract == "Quit" {
        clear_screen();
        println!("{}", "👋 Goodbye!".green().bold());
        std::process::exit(0);
    }

    let gas_strategy_str = Select::new(
        "Select gas strategy:",
        vec!["low", "standard", "fast", "instant"],
    )
    .prompt()
    .map_err(|_| anyhow::anyhow!("Gas strategy selection cancelled"))?;

    let gas_strategy = match gas_strategy_str {
        "low" => GasStrategy::Low,
        "standard" => GasStrategy::Standard,
        "fast" => GasStrategy::Fast,
        "instant" => GasStrategy::Instant,
        _ => GasStrategy::Standard,
    };

    let private_key = core::config::load_private_key()
        .map_err(|_| anyhow::anyhow!("Failed to load private key from .env"))?;

    let artifact_path = format!("artifacts/{}.sol/{}.json", selected_contract, selected_contract);
    
    print!("{}", "Loading contract artifact... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;
    
    let artifact = x_deploy::ArtifactLoader::load_artifact(&artifact_path)?;
    println!("{}", "✓".green().bold());

    println!();
    print!("{}", "Deploying contract... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(async {
        let rpc_url = network.rpc.first()
            .ok_or_else(|| anyhow::anyhow!("No RPC URL available for network"))?;

        let deployer = x_deploy::ContractDeployer::new(rpc_url, &private_key, network.clone())
            .await?;

        deployer.deploy(&artifact, None, gas_strategy).await
    })?;

    println!("{}", "✓".green().bold());

    let deployer_address = x_signature::get_address_from_private_key(&private_key)?;
    
    x_deploy::MetadataManager::save_deployment(
        &selected_contract,
        &format!("{:#x}", result.contract_address),
        network_id,
        &format!("{:#x}", result.tx_hash),
        &format!("{:#x}", deployer_address),
    )?;

    println!("\n{}", "✅ DEPLOYMENT SUCCESSFUL".green().bold());
    print_line("Contract", &selected_contract, |s| s.normal());
    print_line("Network", &network.name, |s| s.cyan());
    print_line("Contract Address", &format!("{:#x}", result.contract_address), |s| s.yellow());
    print_line("Transaction Hash", &format!("{:#x}", result.tx_hash), |s| s.green());
    print_line("Gas Used", &result.gas_used.to_string(), |s| s.normal());
    print_line("Block Explorer", &format!("{}/address/{:#x}", network.block_explorer.url, result.contract_address), |s| s.blue());
    print_separator();
    println!();

    Ok(())
}

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
        display_contracts.push(format!(
            "{}. {} - {}",
            i + 1,
            record.contract_name,
            &record.address[..10]
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
    let encoded = dyn_func.encode_call(selected_func, &dyn_args)?;
    
    // println!();
    // println!("{}", "📝 Debug Info:".cyan().bold());
    // print_line("Contract Address", &selected_record.address, |s| s.cyan());
    // print_line("Network", &network.id, |s| s.cyan());
    // print_line("RPC URL", rpc_url, |s| s.cyan());
    // print_line("Encoded Calldata", &format!("0x{}", hex::encode(encoded.as_ref())), |s| s.yellow());
    // print_line("Calldata Length", &encoded.len().to_string(), |s| s.yellow());
    // println!();

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
    let encoded = dyn_func.encode_call(selected_func, &dyn_args)?;
    
    // println!();
    // println!("{}", "📝 Debug Info:".cyan().bold());
    // print_line("Contract Address", &selected_record.address, |s| s.cyan());
    // print_line("Network", &network.id, |s| s.cyan());
    // print_line("RPC URL", rpc_url, |s| s.cyan());
    // print_line("Encoded Calldata", &format!("0x{}", hex::encode(encoded.as_ref())), |s| s.yellow());
    // print_line("Calldata Length", &encoded.len().to_string(), |s| s.yellow());
    // println!();

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
