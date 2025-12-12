use x_signature;

pub async fn handle_verify(message: String, signature: String, address: String) -> anyhow::Result<()> {
    let expected_addr = x_core::crypto::normalize_address(&address)?;
    let addr_bytes = x_core::crypto::hex_to_bytes(&expected_addr)?;
    let expected_address = ethers::types::Address::from_slice(&addr_bytes);
    
    match x_signature::verify_message(&signature, &message, expected_address) {
        Ok(_) => {
            println!("valid");
            Ok(())
        }
        Err(_) => {
            println!("invalid");
            Ok(())
        }
    }
}
