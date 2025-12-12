use x_core::config;
use x_transfer;

pub async fn handle_transfer_eth(
    network: String,
    amount: f64,
    address: String,
    notes: Option<String>,
) -> anyhow::Result<()> {
    let private_key = config::load_private_key()?;
    let networks = x_core::networks::load_networks()?;
    let network_obj = x_core::networks::get_network_by_id(&networks, &network)
        .ok_or_else(|| anyhow::anyhow!("Network '{}' not found", network))?;

    println!("Sending {:.4} ETH to {}...", amount, address);

    let result = x_transfer::transfer_eth_async(
        &private_key,
        &address,
        amount,
        &network_obj,
        notes.as_deref(),
    ).await?;

    let tx_hash = result.tx_hash.trim_matches('"').to_string();
    let explorer_url = format!("{}/tx/{}", network_obj.block_explorer.url, tx_hash);

    println!("Transaction successful!");
    println!("TX Hash: {}", tx_hash);
    println!("View on Explorer: {}", explorer_url);
    Ok(())
}
