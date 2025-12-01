use colored::Colorize;
use inquire::Text;
use std::io::{self, Write};
use x_core as core;
use x_core::compiler::SmartContractCompiler;
use x_signature;
use x_transfer;

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
