use crate::crypto;
use anyhow::Result;
use ethers::types::Address;
use secp256k1::{Message, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};

/// Signs a message according to EIP-191 personal_sign standard and returns hex signature
///
/// The signature format is: 0x{r}{s}{v} where:
/// - r and s are 32 bytes each (64 hex chars)
/// - v is the recovery id (27 or 28)
///
/// # Arguments
/// * `private_key` - Ethereum private key (with or without 0x prefix)
/// * `message` - Message to sign
///
/// # Returns
/// Signature as hex string with 0x prefix, total 132 characters (0x + 64 + 64 + 2)
pub fn sign_message(private_key: &str, message: &str) -> Result<String> {
    let private_key_str = crate::config::normalize_private_key(private_key);

    let key_bytes = crypto::hex_to_bytes(&private_key_str)?;
    if key_bytes.len() != 32 {
        anyhow::bail!("Private key must be 32 bytes");
    }

    let secret_key = SecretKey::from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;

    let message_payload = crypto::prepare_message_for_signing(message);
    let digest = crypto::keccak256(&message_payload);

    let message = Message::from_slice(&digest)
        .map_err(|e| anyhow::anyhow!("Invalid message digest: {}", e))?;

    let secp = Secp256k1::new();
    let sig = secp.sign_ecdsa_recoverable(&message, &secret_key);

    let (recovery_id, sig_bytes) = sig.serialize_compact();

    let mut signature = Vec::new();
    signature.extend_from_slice(&sig_bytes);
    signature.push(recovery_id.to_i32() as u8 + 27);

    Ok(crypto::bytes_to_hex(&signature))
}

/// Derives the Ethereum address from a private key
pub fn get_address_from_private_key(private_key: &str) -> Result<Address> {
    let private_key_str = crate::config::normalize_private_key(private_key);

    let key_bytes = crypto::hex_to_bytes(&private_key_str)?;
    if key_bytes.len() != 32 {
        anyhow::bail!("Private key must be 32 bytes");
    }

    let secret_key = SecretKey::from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;

    let secp = Secp256k1::new();
    let public_key = secret_key.public_key(&secp);
    let public_key_bytes = public_key.serialize_uncompressed();
    let public_key_uncompressed = &public_key_bytes[1..];

    let mut hasher = Keccak256::new();
    hasher.update(public_key_uncompressed);
    let hash = hasher.finalize();

    let address_bytes: [u8; 20] = hash.as_slice()[12..32]
        .try_into()
        .map_err(|_| anyhow::anyhow!("Address derivation failed"))?;

    Ok(Address::from(address_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_message_with_valid_key() {
        let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let message = "hello";

        let signature = sign_message(private_key, message);
        assert!(signature.is_ok());

        let sig = signature.unwrap();
        assert!(sig.starts_with("0x"));
        assert_eq!(sig.len(), 132);
    }

    #[test]
    fn test_sign_message_without_prefix() {
        let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let message = "test";

        let signature = sign_message(private_key, message);
        assert!(signature.is_ok());
    }

    #[test]
    fn test_sign_message_invalid_key() {
        let private_key = "0xinvalid";
        let message = "hello";

        let signature = sign_message(private_key, message);
        assert!(signature.is_err());
    }
}
