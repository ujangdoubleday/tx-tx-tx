use x_core::compiler::SmartContractCompiler;

pub async fn handle_compile_sc(contract: Option<String>) -> anyhow::Result<()> {
    if let Some(contract_name) = contract {
        SmartContractCompiler::compile_contract(&contract_name)?;
    } else {
        SmartContractCompiler::compile_all()?;
    }
    Ok(())
}
