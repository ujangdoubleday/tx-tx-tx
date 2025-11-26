use sha3::{Digest, Keccak256};

/// Computes Keccak-256 hash of the input bytes
pub fn keccak256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Converts bytes to lowercase hex string with 0x prefix
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Converts hex string (with or without 0x prefix) to bytes
pub fn hex_to_bytes(hex: &str) -> anyhow::Result<Vec<u8>> {
    let trimmed = hex.trim_start_matches("0x").trim_start_matches("0X");
    hex::decode(trimmed).map_err(|e| anyhow::anyhow!("Invalid hex string: {}", e))
}

/// Prepares message for signing according to EIP-191 personal_sign standard
///
/// Format: "\x19Ethereum Signed Message:\n{len}{message}"
pub fn prepare_message_for_signing(message: &str) -> Vec<u8> {
    let message_bytes = message.as_bytes();
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message_bytes.len());
    let mut payload = prefix.as_bytes().to_vec();
    payload.extend_from_slice(message_bytes);
    payload
}

/// Normalizes an Ethereum address to lowercase hex format with 0x prefix
pub fn normalize_address(address: &str) -> anyhow::Result<String> {
    let trimmed = address.trim_start_matches("0x").trim_start_matches("0X");
    if trimmed.len() != 40 {
        anyhow::bail!("Address must be 40 hex characters (20 bytes)");
    }
    Ok(format!("0x{}", trimmed.to_lowercase()))
}
