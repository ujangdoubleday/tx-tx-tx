use colored::Colorize;
use inquire::Select;
use x_core as core;
use x_core::gas::GasStrategy;
use x_signature;
use x_deploy;
use x_gate;

use super::utils::{clear_screen, print_separator, print_line};

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
