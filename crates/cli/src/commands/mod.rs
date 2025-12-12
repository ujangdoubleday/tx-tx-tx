pub mod sign;
pub mod verify;
pub mod transfer_eth;
pub mod deploy;
pub mod compile_sc;
pub mod gen_wallet;
pub mod invoke_stress;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tx-tx-tx")]
#[command(about = "EVM toolkit for signing, verification, transfers, and smart contract deployment.", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Sign {
        #[arg(short, long)]
        message: String,

        #[arg(short, long)]
        private_key: Option<String>,
    },

    Verify {
        #[arg(short, long)]
        message: String,

        #[arg(short, long)]
        signature: String,

        #[arg(short, long)]
        address: String,
    },

    TransferEth {
        #[arg(short, long)]
        network: String,

        #[arg(short, long)]
        amount: f64,

        #[arg(short = 't', long)]
        address: String,

        #[arg(short = 'N', long)]
        notes: Option<String>,
    },

    Deploy {
        #[arg(short, long)]
        network: String,

        #[arg(short, long)]
        contract: String,

        #[arg(short, long, default_value = "standard")]
        gas_strategy: String,
    },

    #[command(name = "compile-sc")]
    CompileSc {
        #[arg(short, long)]
        contract: Option<String>,
    },

    #[command(name = "gen-wallet")]
    GenWallet {
        #[arg(short, long, default_value = "1")]
        count: usize,

        #[arg(short, long, default_value = "wallets.json")]
        filename: String,
    },

    #[command(name = "invoke-stress")]
    InvokeStress {
        #[arg(short, long)]
        contract: String,

        #[arg(short, long)]
        network: String,

        #[arg(short, long)]
        function: String,

        #[arg(short, long, default_value = "")]
        args: String,

        #[arg(short, long, default_value = "10")]
        transactions: usize,

        #[arg(short, long, default_value = "1000")]
        interval: u64,
    },
}

impl Cli {
    pub async fn execute(&self) -> anyhow::Result<()> {
        match &self.command {
            Commands::Sign {
                message,
                private_key,
            } => {
                sign::handle_sign(message.clone(), private_key.clone()).await
            }

            Commands::Verify { message, signature, address } => {
                verify::handle_verify(message.clone(), signature.clone(), address.clone()).await
            }

            Commands::TransferEth {
                network,
                amount,
                address,
                notes,
            } => {
                transfer_eth::handle_transfer_eth(
                    network.clone(),
                    *amount,
                    address.clone(),
                    notes.clone(),
                ).await
            }

            Commands::Deploy {
                network,
                contract,
                gas_strategy,
            } => {
                deploy::handle_deploy(
                    network.clone(),
                    contract.clone(),
                    gas_strategy.clone(),
                ).await
            }

            Commands::CompileSc { contract } => {
                compile_sc::handle_compile_sc(contract.clone()).await
            }

            Commands::GenWallet { count, filename } => {
                gen_wallet::handle_gen_wallet(*count, filename.clone()).await
            }

            Commands::InvokeStress { contract, network, function, args, transactions, interval } => {
                invoke_stress::handle_invoke_stress(
                    contract.clone(),
                    network.clone(),
                    function.clone(),
                    args.clone(),
                    *transactions,
                    *interval,
                ).await
            }
        }
    }
}
