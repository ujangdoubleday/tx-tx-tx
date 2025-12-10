use colored::Colorize;
use x_core::compiler::SmartContractCompiler;

use super::utils::print_separator;
use crate::ui::loading::{create_spinner, finish_spinner};

pub fn handle_compile_smart_contracts() -> anyhow::Result<()> {
    println!("{}", "🔨 COMPILE SMART CONTRACTS".cyan().bold());
    println!();

    let spinner = create_spinner("Compiling all smart contracts...");

    SmartContractCompiler::compile_all()?;

    finish_spinner(spinner, "Compiling all smart contracts... ");

    println!("\n{}", "✅ COMPILATION SUCCESSFUL".green().bold());
    print_separator();
    println!();

    Ok(())
}
