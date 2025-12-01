use x_core::config;
use x_core::gas::GasStrategy;
use x_core::compiler::SmartContractCompiler;
use x_signature;
use x_transfer;
use x_deploy;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tx-tx-tx")]
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

    /// Deploy a smart contract
    Deploy {
        /// Network ID (e.g., ethereum_mainnet, testnet_sepolia)
        #[arg(long)]
        network: String,

        /// Contract name (e.g., HelloWorld)
        #[arg(long)]
        contract: String,

        /// Gas strategy (low, standard, fast, instant)
        #[arg(long, default_value = "standard")]
        gas_strategy: String,
    },

    /// Compile smart contracts with Foundry
    #[command(name = "compile-sc")]
    CompileSc {
        /// Contract name (optional - if not provided, compiles all contracts)
        #[arg(long)]
        contract: Option<String>,
    },
}

impl Cli {
    pub async fn execute(&self) -> anyhow::Result<()> {
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

                let result = x_transfer::transfer_eth_async(
                    &private_key,
                    address,
                    *amount,
                    network_obj,
                    notes.as_deref(),
                ).await?;

                let tx_hash = result.tx_hash.trim_matches('"').to_string();
                let explorer_url = format!("{}/tx/{}", network_obj.block_explorer.url, tx_hash);

                println!("Transaction successful!");
                println!("TX Hash: {}", tx_hash);
                println!("View on Explorer: {}", explorer_url);
                Ok(())
            }

            Commands::Deploy {
                network,
                contract,
                gas_strategy,
            } => {
                let private_key = config::load_private_key()?;
                let networks = x_core::networks::load_networks()?;
                let network_obj = x_core::networks::get_network_by_id(&networks, network)
                    .ok_or_else(|| anyhow::anyhow!("Network '{}' not found", network))?;

                let strategy = match gas_strategy.as_str() {
                    "low" => GasStrategy::Low,
                    "standard" => GasStrategy::Standard,
                    "fast" => GasStrategy::Fast,
                    "instant" => GasStrategy::Instant,
                    _ => return Err(anyhow::anyhow!("Invalid gas strategy: {}", gas_strategy)),
                };

                let artifact_path = format!("artifacts/{}.sol/{}.json", contract, contract);
                
                println!("Loading contract artifact from {}...", artifact_path);
                let artifact = x_deploy::ArtifactLoader::load_artifact(&artifact_path)?;

                println!("Deploying {} to {} with {:?} strategy...", contract, network_obj.name, strategy);

                let rpc_url = network_obj.rpc.first()
                    .ok_or_else(|| anyhow::anyhow!("No RPC URL available for network"))?;

                let deployer = x_deploy::ContractDeployer::new(rpc_url, &private_key, network_obj.clone())
                    .await?;

                let result = deployer.deploy(&artifact, None, strategy).await?;

                let deployer_address = x_signature::get_address_from_private_key(&private_key)?;
                
                x_deploy::MetadataManager::save_deployment(
                    contract,
                    &format!("{:#x}", result.contract_address),
                    network,
                    &format!("{:#x}", result.tx_hash),
                    &format!("{:#x}", deployer_address),
                )?;

                println!("\n✓ Deployment successful!");
                println!("Contract Address: {:#x}", result.contract_address);
                println!("Transaction Hash: {:#x}", result.tx_hash);
                println!("Gas Used: {} ({} gwei)", 
                    result.gas_used,
                    result.gas_estimate.max_fee_per_gas.unwrap_or(result.gas_estimate.gas_price) / 1_000_000_000u64
                );

                let tx_explorer_url = format!(
                    "{}/tx/{:#x}",
                    network_obj.block_explorer.url,
                    result.tx_hash
                );
                let contract_explorer_url = format!(
                    "{}/address/{:#x}",
                    network_obj.block_explorer.url,
                    result.contract_address
                );
                println!("View Transaction: {}", tx_explorer_url);
                println!("View Contract: {}", contract_explorer_url);
                
                Ok(())
            }

            Commands::CompileSc { contract } => {
                if let Some(contract_name) = contract {
                    SmartContractCompiler::compile_contract(contract_name)?;
                } else {
                    SmartContractCompiler::compile_all()?;
                }
                Ok(())
            }
        }
    }
}
