use x_core::config;
use x_signature;

pub async fn handle_sign(message: String, private_key: Option<String>) -> anyhow::Result<()> {
    let key = if let Some(pk) = private_key {
        pk
    } else {
        config::load_private_key()?
    };

    let signature = x_signature::sign_message(&key, &message)?;
    let address = x_signature::get_address_from_private_key(&key)?;
    println!("Signature: {}", signature);
    println!("Address: {:#x}", address);
    Ok(())
}
