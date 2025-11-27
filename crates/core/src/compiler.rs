use anyhow::Result;
use std::process::Command;

pub struct SmartContractCompiler;

impl SmartContractCompiler {
    pub fn compile_all() -> Result<()> {
        println!("Compiling all smart contracts...");
        
        let output = Command::new("forge")
            .arg("build")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Compilation failed:\n{}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
        println!("✓ All smart contracts compiled successfully!");
        Ok(())
    }

    pub fn compile_contract(contract_name: &str) -> Result<()> {
        println!("Compiling contract: {}...", contract_name);
        
        let output = Command::new("forge")
            .arg("build")
            .arg("--skip")
            .arg(format!("!{}", contract_name))
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Compilation failed for {}:\n{}", contract_name, stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
        println!("✓ Contract {} compiled successfully!", contract_name);
        Ok(())
    }
}
