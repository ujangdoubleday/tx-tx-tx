use crate::crypto;
use anyhow::Result;
use ethers::types::Address;
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId, Signature};
use secp256k1::{Message, Secp256k1};
use sha3::{Digest, Keccak256};

/// Verifies a message signature against an expected address
///
/// # Arguments
/// * `signature_hex` - Signature as hex string (0x{r}{s}{v}, 132 chars)
/// * `message` - Original message that was signed
/// * `expected_address` - Expected Ethereum address that should have signed this
///
/// # Returns
/// The address that signed the message (same as expected_address if valid)
pub fn verify_message(signature_hex: &str, message: &str, expected_address: Address) -> Result<Address> {
    let signature_bytes = crypto::hex_to_bytes(signature_hex)?;

    if signature_bytes.len() != 65 {
        anyhow::bail!("Signature must be 65 bytes (130 hex chars)");
    }

    let message_payload = crypto::prepare_message_for_signing(message);
    let digest = crypto::keccak256(&message_payload);

    let v_byte = signature_bytes[64];
    let recovery_id = match v_byte {
        27 => RecoveryId::from_i32(0).map_err(|e| anyhow::anyhow!("Invalid recovery id: {}", e))?,
        28 => RecoveryId::from_i32(1).map_err(|e| anyhow::anyhow!("Invalid recovery id: {}", e))?,
        0 => RecoveryId::from_i32(0).map_err(|e| anyhow::anyhow!("Invalid recovery id: {}", e))?,
        1 => RecoveryId::from_i32(1).map_err(|e| anyhow::anyhow!("Invalid recovery id: {}", e))?,
        _ => anyhow::bail!("Invalid recovery id: {}", v_byte),
    };

    let message = Message::from_slice(&digest)
        .map_err(|e| anyhow::anyhow!("Invalid message digest: {}", e))?;

    let sig_bytes: [u8; 64] = signature_bytes[0..64]
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid signature bytes"))?;

    let sig = RecoverableSignature::from_compact(&sig_bytes, recovery_id)
        .map_err(|e| anyhow::anyhow!("Invalid signature: {}", e))?;

    let secp = Secp256k1::new();
    let public_key = secp
        .recover_ecdsa(&message, &sig)
        .map_err(|e| anyhow::anyhow!("Key recovery failed: {}", e))?;

    let regular_sig = Signature::from_compact(&sig_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid signature format: {}", e))?;

    secp.verify_ecdsa(&message, &regular_sig, &public_key)
        .map_err(|_| anyhow::anyhow!("Signature verification failed"))?;

    let public_key_bytes = public_key.serialize_uncompressed();
    let public_key_uncompressed = &public_key_bytes[1..];

    let mut hasher = Keccak256::new();
    hasher.update(public_key_uncompressed);
    let hash = hasher.finalize();

    let address_bytes: [u8; 20] = hash.as_slice()[12..32]
        .try_into()
        .map_err(|_| anyhow::anyhow!("Address derivation failed"))?;

    let recovered_address = Address::from(address_bytes);
    
    if recovered_address != expected_address {
        anyhow::bail!("Signature does not match expected address");
    }

    Ok(recovered_address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_message_with_valid_signature() {
        use ethers::types::Address;
        use std::str::FromStr;
        
        let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let message = "hello";
        let expected_address = Address::from_str("0x7e5f4552091a69125d5dfcb7b8c2659029395bdf").unwrap();

        let signature = crate::evm::sign::sign_message(private_key, message).unwrap();
        let address = verify_message(&signature, message, expected_address);

        assert!(address.is_ok());
        assert_eq!(address.unwrap(), expected_address);
    }

    #[test]
    fn test_verify_message_invalid_signature() {
        use ethers::types::Address;
        use std::str::FromStr;
        
        let message = "hello";
        let invalid_sig = format!("0x{}", "00".repeat(65));
        let expected_address = Address::from_str("0x7e5f4552091a69125d5dfcb7b8c2659029395bdf").unwrap();

        let address = verify_message(&invalid_sig, message, expected_address);
        assert!(address.is_err());
    }

    #[test]
    fn test_verify_message_wrong_message() {
        use ethers::types::Address;
        use std::str::FromStr;
        
        let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let message1 = "hello";
        let message2 = "world";
        let expected_address = Address::from_str("0x7e5f4552091a69125d5dfcb7b8c2659029395bdf").unwrap();

        let signature = crate::evm::sign::sign_message(private_key, message1).unwrap();
        let result_correct = verify_message(&signature, message1, expected_address);
        let result_wrong = verify_message(&signature, message2, expected_address);

        assert!(result_correct.is_ok());
        assert!(result_wrong.is_err());
    }
}
