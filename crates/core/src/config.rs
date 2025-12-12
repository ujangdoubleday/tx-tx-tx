use anyhow::Result;

/// Loads the Ethereum private key from Docker secrets or .env file
///
/// Priority order:
/// 1. Docker secret file (ETH_PRIVATE_KEY_FILE)
/// 2. Environment variable (ETH_PRIVATE_KEY)
/// 3. Fallback environment variable (PRIVATE_KEY)
/// The private key can be with or without `0x` prefix.
pub fn load_private_key() -> Result<String> {
    // Try Docker secret first
    if let Ok(secret_path) = std::env::var("ETH_PRIVATE_KEY_FILE") {
        let key = std::fs::read_to_string(&secret_path)
            .map_err(|e| anyhow::anyhow!("Failed to read secret file {}: {}", secret_path, e))?;
        
        let trimmed = key.trim();
        if trimmed.is_empty() {
            anyhow::bail!("Private key from secret file is empty");
        }
        return Ok(trimmed.to_string());
    }

    // Fallback to environment variables
    dotenvy::dotenv().ok();

    let private_key = std::env::var("ETH_PRIVATE_KEY")
        .or_else(|_| std::env::var("PRIVATE_KEY"))
        .map_err(|_| anyhow::anyhow!("ETH_PRIVATE_KEY or PRIVATE_KEY not found in environment variables"))?;

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
