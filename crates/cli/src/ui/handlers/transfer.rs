use colored::Colorize;
use inquire::Text;
use x_core as core;

use super::utils::{print_separator, print_line};

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
