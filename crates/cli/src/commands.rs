use x_core::config;
use x_signature;
use x_transaction;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "my_evm_signer")]
#[command(about = "EVM message signing and verification tool", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Sign a message with private key from .env
    Sign {
        /// Message to sign
        #[arg(long)]
        message: String,

        /// Private key (optional, uses .env if not provided)
        #[arg(long)]
        private_key: Option<String>,
    },

    /// Verify a message signature against an expected address
    Verify {
        /// Message that was signed
        #[arg(long)]
        message: String,

        /// Signature in hex format (with or without 0x prefix)
        #[arg(long)]
        signature: String,

        /// Expected Ethereum address (with or without 0x prefix)
        #[arg(long)]
        address: String,
    },

    /// Transfer ETH to an address
    TransferEth {
        /// Network ID (e.g., ethereum_mainnet, testnet_sepolia)
        #[arg(long)]
        network: String,

        /// Amount in ETH (e.g., 0.01)
        #[arg(long)]
        amount: f64,

        /// Recipient Ethereum address
        #[arg(long)]
        address: String,

        /// Transaction notes (optional)
        #[arg(long)]
        notes: Option<String>,
    },
}

impl Cli {
    pub fn execute(&self) -> anyhow::Result<()> {
        match &self.command {
            Commands::Sign {
                message,
                private_key,
            } => {
                let key = if let Some(pk) = private_key {
                    pk.clone()
                } else {
                    config::load_private_key()?
                };

                let signature = x_signature::sign_message(&key, message)?;
                let address = x_signature::get_address_from_private_key(&key)?;
                println!("Signature: {}", signature);
                println!("Address: {:#x}", address);
                Ok(())
            }

            Commands::Verify { message, signature, address } => {
                let expected_addr = x_core::crypto::normalize_address(address)?;
                let addr_bytes = x_core::crypto::hex_to_bytes(&expected_addr)?;
                let expected_address = ethers::types::Address::from_slice(&addr_bytes);
                
                match x_signature::verify_message(signature, message, expected_address) {
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

            Commands::TransferEth {
                network,
                amount,
                address,
                notes,
            } => {
                let private_key = config::load_private_key()?;
                let networks = x_core::networks::load_networks()?;
                let network_obj = x_core::networks::get_network_by_id(&networks, network)
                    .ok_or_else(|| anyhow::anyhow!("Network '{}' not found", network))?;

                println!("Sending {:.4} ETH to {}...", amount, address);

                let result = x_transaction::transfer_eth(
                    &private_key,
                    address,
                    *amount,
                    network_obj,
                    notes.as_deref(),
                )?;

                let tx_hash = result.tx_hash.trim_matches('"').to_string();
                let explorer_url = format!("{}/tx/{}", network_obj.block_explorer.url, tx_hash);

                println!("Transaction successful!");
                println!("TX Hash: {}", tx_hash);
                println!("View on Explorer: {}", explorer_url);
                Ok(())
            }
        }
    }
}
