use crate::wallet::Wallet;
use anyhow::Result;
use std::fs;
use std::path::Path;
use x_signature::get_address_from_private_key;

pub struct WalletGenerator;

impl WalletGenerator {
    fn generate_wallet_with_id(id: usize) -> Result<Wallet> {
        let secret_key = secp256k1::SecretKey::new(&mut rand::thread_rng());
        let private_key_bytes = secret_key.as_ref();
        let private_key = format!("0x{}", hex::encode(private_key_bytes));

        let address = get_address_from_private_key(&private_key)?;
        let wallet = Wallet::new(
            id.to_string(),
            private_key,
            format!("0x{:x}", address),
        );

        Ok(wallet)
    }

    pub fn generate_wallets(count: usize) -> Result<Vec<Wallet>> {
        let mut wallets = Vec::new();
        for i in 1..=count {
            wallets.push(Self::generate_wallet_with_id(i)?);
        }
        Ok(wallets)
    }

    pub fn save_wallets_to_json(wallets: &[Wallet], output_path: &str) -> Result<()> {
        let directory = Path::new(output_path).parent().unwrap_or_else(|| Path::new(""));

        if !directory.as_os_str().is_empty() && !directory.exists() {
            fs::create_dir_all(directory)?;
        }

        let json_data = serde_json::to_string_pretty(&wallets)?;
        fs::write(output_path, json_data)?;

        Ok(())
    }

    pub fn generate_and_save(count: usize, output_path: &str) -> Result<Vec<Wallet>> {
        let wallets = Self::generate_wallets(count)?;
        Self::save_wallets_to_json(&wallets, output_path)?;
        Ok(wallets)
    }
}
