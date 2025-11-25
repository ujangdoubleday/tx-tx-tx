use anyhow::Result;

/// Loads the Ethereum private key from .env file
///
/// Tries to load `ETH_PRIVATE_KEY` first, then falls back to `PRIVATE_KEY`.
/// The private key can be with or without `0x` prefix.
pub fn load_private_key() -> Result<String> {
    dotenvy::dotenv().ok();

    let private_key = std::env::var("ETH_PRIVATE_KEY")
        .or_else(|_| std::env::var("PRIVATE_KEY"))
        .map_err(|_| anyhow::anyhow!("ETH_PRIVATE_KEY or PRIVATE_KEY not found in .env"))?;

    let trimmed = private_key.trim();

    if trimmed.is_empty() {
        anyhow::bail!("Private key is empty");
    }

    Ok(trimmed.to_string())
}

/// Normalizes a hex private key (adds 0x prefix if missing)
pub fn normalize_private_key(key: &str) -> String {
    let trimmed = key.trim();
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        trimmed.to_lowercase()
    } else {
        format!("0x{}", trimmed.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_private_key_with_prefix() {
        let key = "0xabcd1234";
        assert_eq!(normalize_private_key(key), "0xabcd1234");
    }

    #[test]
    fn test_normalize_private_key_without_prefix() {
        let key = "abcd1234";
        assert_eq!(normalize_private_key(key), "0xabcd1234");
    }
}
