use clap::Parser;
use std::env;

use x_cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let cli = Cli::parse();
        cli.execute().await
    } else {
        x_cli::ui::run()
    }
}
