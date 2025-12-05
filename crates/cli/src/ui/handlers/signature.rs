use colored::Colorize;
use inquire::Text;
use x_core as core;

use super::utils::{print_separator, print_line, read_input_line};

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
