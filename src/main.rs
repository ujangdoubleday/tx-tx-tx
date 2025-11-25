use clap::Parser;

mod cli;
mod config;
mod crypto;
mod evm;

use cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.execute()
}
