use clap::Parser;
use std::env;

use x_cli::Cli;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let rt = tokio::runtime::Runtime::new()?;
        let cli = Cli::parse();
        rt.block_on(cli.execute())
    } else {
        x_cli::ui::run()
    }
}
