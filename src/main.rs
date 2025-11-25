use clap::Parser;
use std::env;

mod cli;
mod core;
mod features;

use cli::Cli;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let cli = Cli::parse();
        cli.execute()
    } else {
        cli::ui::run()
    }
}
