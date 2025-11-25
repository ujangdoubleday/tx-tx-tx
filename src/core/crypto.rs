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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak256() {
        let input = b"hello";
        let hash = keccak256(input);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_bytes_to_hex() {
        let bytes = vec![0x01, 0x02, 0x03];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "0x010203");
    }

    #[test]
    fn test_hex_to_bytes() {
        let hex = "0x010203";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_hex_to_bytes_without_prefix() {
        let hex = "010203";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_prepare_message_for_signing() {
        let message = "hello";
        let payload = prepare_message_for_signing(message);
        let expected_prefix = "\x19Ethereum Signed Message:\n5";
        let mut expected = expected_prefix.as_bytes().to_vec();
        expected.extend_from_slice(b"hello");
        assert_eq!(payload, expected);
    }
}
