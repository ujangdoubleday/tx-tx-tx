use crate::config;
use crate::evm;
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

                let signature = evm::sign_message(&key, message)?;
                let address = evm::get_address_from_private_key(&key)?;
                println!("Signature: {}", signature);
                println!("Address: {:#x}", address);
                Ok(())
            }

            Commands::Verify { message, signature, address } => {
                let expected_addr = crate::crypto::normalize_address(address)?;
                let addr_bytes = crate::crypto::hex_to_bytes(&expected_addr)?;
                let expected_address = ethers::types::Address::from_slice(&addr_bytes);
                
                match evm::verify_message(signature, message, expected_address) {
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
        }
    }
}
