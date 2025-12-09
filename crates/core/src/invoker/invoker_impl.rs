use anyhow::Result;
use alloy_primitives::{Address, Bytes};
use alloy_dyn_abi::DynSolValue;
use super::deployment::{DeploymentManager, DeployedContract};
use super::executor::{ContractExecutor, ExecutionResult, ReadResult};
use crate::networks::Network;

pub struct ContractInvoker {
    deployments_file: String,
    artifact_dir: String,
}

pub struct DeployedContractInvoker {
    contract: DeployedContract,
}

impl ContractInvoker {
    pub fn new(deployments_file: &str, artifact_dir: &str) -> Self {
        ContractInvoker {
            deployments_file: deployments_file.to_string(),
            artifact_dir: artifact_dir.to_string(),
        }
    }

    pub fn get_contract(
        &self,
        contract_name: &str,
        network: &str,
    ) -> Result<DeployedContractInvoker> {
        let contract = DeploymentManager::get_deployed_contract(
            &self.deployments_file,
            &self.artifact_dir,
            contract_name,
            network,
        )?;

        Ok(DeployedContractInvoker { contract })
    }

    pub fn get_contract_by_address(
        &self,
        contract_name: &str,
        address: &str,
        network: &str,
    ) -> Result<DeployedContractInvoker> {
        let contract = DeploymentManager::get_deployed_contract_by_address(
            &self.deployments_file,
            &self.artifact_dir,
            contract_name,
            address,
            network,
        )?;

        Ok(DeployedContractInvoker { contract })
    }
}

impl DeployedContractInvoker {
    pub fn contract_name(&self) -> &str {
        &self.contract.record.contract_name
    }

    pub fn network(&self) -> &str {
        &self.contract.record.network
    }

    pub fn address(&self) -> Result<Address> {
        self.contract.address()
    }

    pub fn encode_function_call(
        &self,
        function_name: &str,
        args: &[DynSolValue],
    ) -> Result<Bytes> {
        let dyn_func = self.contract.get_function_abi(function_name)?;
        dyn_func.encode_call(function_name, args)
    }

    pub fn get_function_info(
        &self,
        function_name: &str,
    ) -> Result<(Vec<(String, String)>, Vec<(String, String)>)> {
        self.contract.get_function_info(function_name)
    }

    pub fn get_all_functions(&self) -> Result<Vec<String>> {
        self.contract.get_all_functions()
    }

    pub fn get_deployed_record(&self) -> &super::deployment::DeploymentRecord {
        &self.contract.record
    }

    pub fn get_artifact_abi(&self) -> &serde_json::Value {
        &self.contract.artifact.abi
    }

    pub fn get_function_abi(&self, function_name: &str) -> Result<super::abi::DynAbiFunction> {
        self.contract.get_function_abi(function_name)
    }

    pub async fn execute_read_function(
        &self,
        rpc_url: &str,
        private_key: &str,
        network: &Network,
        function_name: &str,
        args: &[DynSolValue],
    ) -> Result<ReadResult> {
        let executor = ContractExecutor::new(rpc_url, private_key, network.clone()).await?;
        let dyn_func = self.contract.get_function_abi(function_name)?;
        let contract_address = self.contract.address()?;
        
        executor.call_read_function(contract_address, &dyn_func, args, function_name).await
    }

    pub async fn execute_write_function(
        &self,
        rpc_url: &str,
        private_key: &str,
        network: &Network,
        function_name: &str,
        args: &[DynSolValue],
    ) -> Result<ExecutionResult> {
        let executor = ContractExecutor::new(rpc_url, private_key, network.clone()).await?;
        let dyn_func = self.contract.get_function_abi(function_name)?;
        let contract_address = self.contract.address()?;
        
        executor.call_write_function(contract_address, &dyn_func, args, function_name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_contract_invoker() {
        let deployments_file = "deployments/testnet_sepolia.json";
        let artifact_dir = "artifacts";

        if Path::new(deployments_file).exists() && Path::new(artifact_dir).exists() {
            let invoker = ContractInvoker::new(deployments_file, artifact_dir);
            let result = invoker.get_contract("HelloWorld", "testnet_sepolia");
            assert!(result.is_ok());

            if let Ok(contract_invoker) = result {
                assert_eq!(contract_invoker.contract_name(), "HelloWorld");
                assert_eq!(contract_invoker.network(), "testnet_sepolia");
                assert!(contract_invoker.address().is_ok());
            }
        }
    }

    #[test]
    fn test_get_all_functions() {
        let deployments_file = "deployments/testnet_sepolia.json";
        let artifact_dir = "artifacts";

        if Path::new(deployments_file).exists() && Path::new(artifact_dir).exists() {
            if let Ok(invoker) = ContractInvoker::new(deployments_file, artifact_dir)
                .get_contract("HelloWorld", "testnet_sepolia")
            {
                let functions = invoker.get_all_functions();
                assert!(functions.is_ok());

                if let Ok(funcs) = functions {
                    assert!(!funcs.is_empty());
                    assert!(funcs.contains(&"getMessage".to_string()));
                    assert!(funcs.contains(&"setMessage".to_string()));
                }
            }
        }
    }

    #[test]
    fn test_get_function_info() {
        let deployments_file = "deployments/testnet_sepolia.json";
        let artifact_dir = "artifacts";

        if Path::new(deployments_file).exists() && Path::new(artifact_dir).exists() {
            if let Ok(invoker) = ContractInvoker::new(deployments_file, artifact_dir)
                .get_contract("HelloWorld", "testnet_sepolia")
            {
                let info = invoker.get_function_info("setMessage");
                assert!(info.is_ok());

                if let Ok((inputs, _outputs)) = info {
                    assert!(!inputs.is_empty());
                    assert_eq!(inputs[0].0, "newMessage");
                    assert_eq!(inputs[0].1, "string");
                }
            }
        }
    }
}
