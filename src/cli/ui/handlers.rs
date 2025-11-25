use colored::Colorize;
use inquire::Text;
use crate::{core, features};

pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    std::io::Write::flush(&mut std::io::stdout()).ok();
}

pub fn handle_sign() -> anyhow::Result<()> {
    clear_screen();
    println!("{}", "📝 SIGN MESSAGE".cyan().bold());

    let message = Text::new("Enter message to sign:")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    if message.trim().is_empty() {
        anyhow::bail!("Message cannot be empty");
    }

    print!("{}", "Generating signature... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    let key = core::config::load_private_key()
        .map_err(|_| anyhow::anyhow!("Failed to load private key from .env"))?;

    let signature = features::sign_message(&key, message.trim())?;
    let address = features::get_address_from_private_key(&key)?;

    println!("{}", "✓".green().bold());

    println!("\n{}", "✅ SIGNATURE GENERATED".green().bold());
    println!("  {}: {}", "Message".bold(), message.trim());
    println!("  {}: {}", "Signature".bold(), signature.yellow());
    println!("  {}: {}", "Address".bold(), format!("{:#x}", address).yellow());

    let _ = Text::new("\nPress Enter to continue...")
        .prompt();

    Ok(())
}

pub fn handle_verify() -> anyhow::Result<()> {
    clear_screen();
    println!("{}", "✓ VERIFY MESSAGE".cyan().bold());

    let message = Text::new("Enter the message:")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    if message.trim().is_empty() {
        anyhow::bail!("Message cannot be empty");
    }

    let signature = Text::new("Enter the signature (with or without 0x):")
        .prompt()
        .map_err(|_| anyhow::anyhow!("Input cancelled"))?;

    if signature.trim().is_empty() {
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

    let is_valid = match features::verify_message(signature.trim(), message.trim(), expected_address) {
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

    println!("  {}: {}", "Message".bold(), message.trim().cyan());
    println!("  {}: {}", "Address".bold(), format!("{:#x}", expected_address).cyan());

    let _ = Text::new("\nPress Enter to continue...")
        .prompt();

    Ok(())
}
