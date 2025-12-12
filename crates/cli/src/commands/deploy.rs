use x_core::config;
use x_core::gas::GasStrategy;
use x_signature;
use x_deploy;

pub async fn handle_deploy(
    network: String,
    contract: String,
    gas_strategy: String,
) -> anyhow::Result<()> {
    let private_key = config::load_private_key()?;
    let networks = x_core::networks::load_networks()?;
    let network_obj = x_core::networks::get_network_by_id(&networks, &network)
        .ok_or_else(|| anyhow::anyhow!("Network '{}' not found", network))?;

    let strategy = match gas_strategy.as_str() {
        "low" => GasStrategy::Low,
        "standard" => GasStrategy::Standard,
        "fast" => GasStrategy::Fast,
        "instant" => GasStrategy::Instant,
        _ => return Err(anyhow::anyhow!("Invalid gas strategy: {}", gas_strategy)),
    };

    let artifact_path = format!("artifacts/{}.sol/{}.json", contract, contract);
    
    println!("Loading contract artifact from {}...", artifact_path);
    let artifact = x_deploy::ArtifactLoader::load_artifact(&artifact_path)?;

    println!("Deploying {} to {} with {:?} strategy...", contract, network_obj.name, strategy);

    let rpc_url = network_obj.rpc.first()
        .ok_or_else(|| anyhow::anyhow!("No RPC URL available for network"))?;

    let deployer = x_deploy::ContractDeployer::new(rpc_url, &private_key, network_obj.clone())
        .await?;

    let result = deployer.deploy(&artifact, None, strategy).await?;

    let deployer_address = x_signature::get_address_from_private_key(&private_key)?;
    
    x_deploy::MetadataManager::save_deployment(
        &contract,
        &format!("{:#x}", result.contract_address),
        &network,
        &format!("{:#x}", result.tx_hash),
        &format!("{:#x}", deployer_address),
    )?;

    println!("\n✓ Deployment successful!");
    println!("Contract Address: {:#x}", result.contract_address);
    println!("Transaction Hash: {:#x}", result.tx_hash);
    println!("Gas Used: {} ({} gwei)", 
        result.gas_used,
        result.gas_estimate.max_fee_per_gas.unwrap_or(result.gas_estimate.gas_price) / 1_000_000_000u64
    );

    let tx_explorer_url = format!(
        "{}/tx/{:#x}",
        network_obj.block_explorer.url,
        result.tx_hash
    );
    let contract_explorer_url = format!(
        "{}/address/{:#x}",
        network_obj.block_explorer.url,
        result.contract_address
    );
    println!("View Transaction: {}", tx_explorer_url);
    println!("View Contract: {}", contract_explorer_url);
    
    Ok(())
}
