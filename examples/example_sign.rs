use tx_tx_tx::{core::config, sign_message, verify_message};
use ethers::types::Address;
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let expected_address = Address::from_str("0x7e5f4552091a69125d5dfcb7b8c2659029395bdf")?;
    let message = "Hello, World!";

    println!("=== EVM Message Signing Example ===\n");

    println!("Message: {}", message);
    println!("Private Key: {}...", &private_key[..10]);
    println!("Expected Address: {:#x}", expected_address);

    let signature = sign_message(private_key, message)?;
    println!("\nGenerated Signature:");
    println!("{}", signature);

    println!("\nVerifying signature...");
    let recovered_address = verify_message(&signature, message, expected_address)?;
    println!("Recovered Address: {}", recovered_address);

    println!("\n=== Testing with different message ===\n");
    let different_message = "Different message";
    let sig_different = sign_message(private_key, different_message)?;
    println!("Message: {}", different_message);
    println!("Signature: {}", sig_different);

    let recovered_addr2 = verify_message(&sig_different, different_message, expected_address)?;
    println!("Recovered Address: {}", recovered_addr2);

    println!("\n=== Loading from .env ===\n");
    let loaded_key = config::load_private_key()?;
    println!("Loaded private key from .env");
    let sig_env = sign_message(&loaded_key, "test message")?;
    println!("Signed with loaded key: {}", sig_env);

    Ok(())
}
