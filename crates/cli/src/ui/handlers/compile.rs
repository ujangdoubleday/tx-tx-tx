use colored::Colorize;
use x_core::compiler::SmartContractCompiler;

use super::utils::print_separator;

pub fn handle_compile_smart_contracts() -> anyhow::Result<()> {
    println!("{}", "🔨 COMPILE SMART CONTRACTS".cyan().bold());
    println!();

    print!("{}", "Compiling all smart contracts... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    SmartContractCompiler::compile_all()?;

    println!("\n{}", "✅ COMPILATION SUCCESSFUL".green().bold());
    print_separator();
    println!();

    Ok(())
}
