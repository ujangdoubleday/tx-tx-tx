use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use super::abi::DynAbiFunction;
use alloy_primitives::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub contract_name: String,
    pub address: String,
    pub network: String,
    pub tx_hash: String,
    pub deployer: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractArtifact {
    pub abi: serde_json::Value,
    #[serde(default)]
    pub bytecode: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct DeployedContract {
    pub record: DeploymentRecord,
    pub artifact: ContractArtifact,
}

pub struct DeploymentManager;

impl DeploymentManager {
    pub fn load_deployments(file_path: &str) -> Result<Vec<DeploymentRecord>> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| anyhow!("Failed to read deployment file: {}", e))?;

        let records: Vec<DeploymentRecord> = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse deployment file: {}", e))?;

        Ok(records)
    }

    pub fn load_artifact(
        artifact_dir: &str,
        contract_name: &str,
    ) -> Result<ContractArtifact> {
        let artifact_path = format!(
            "{}/{}.sol/{}.json",
            artifact_dir, contract_name, contract_name
        );

        let content = fs::read_to_string(&artifact_path)
            .map_err(|e| anyhow!("Failed to read artifact file {}: {}", artifact_path, e))?;

        let artifact: ContractArtifact = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse artifact: {}", e))?;

        Ok(artifact)
    }

    pub fn get_deployed_contract(
        deployments_file: &str,
        artifact_dir: &str,
        contract_name: &str,
        network: &str,
    ) -> Result<DeployedContract> {
        let records = Self::load_deployments(deployments_file)?;

        let record = records
            .into_iter()
            .find(|r| r.contract_name == contract_name && r.network == network)
            .ok_or_else(|| {
                anyhow!(
                    "Deployment not found for {} on network {}",
                    contract_name,
                    network
                )
            })?;

        let artifact = Self::load_artifact(artifact_dir, contract_name)?;

        Ok(DeployedContract { record, artifact })
    }

    pub fn get_deployed_contract_by_address(
        deployments_file: &str,
        artifact_dir: &str,
        contract_name: &str,
        address: &str,
        network: &str,
    ) -> Result<DeployedContract> {
        let records = Self::load_deployments(deployments_file)?;

        let record = records
            .into_iter()
            .find(|r| r.contract_name == contract_name && r.address == address && r.network == network)
            .ok_or_else(|| {
                anyhow!(
                    "Deployment not found for {} at address {} on network {}",
                    contract_name,
                    address,
                    network
                )
            })?;

        let artifact = Self::load_artifact(artifact_dir, contract_name)?;

        Ok(DeployedContract { record, artifact })
    }

    pub fn get_all_deployments_for_network(
        file_path: &str,
        network: &str,
    ) -> Result<Vec<DeploymentRecord>> {
        let records = Self::load_deployments(file_path)?;
        Ok(records
            .into_iter()
            .filter(|r| r.network == network)
            .collect())
    }
}

impl DeployedContract {
    pub fn address(&self) -> Result<Address> {
        self.record
            .address
            .parse()
            .map_err(|_| anyhow!("Invalid contract address"))
    }

    pub fn get_function_abi(&self, function_name: &str) -> Result<DynAbiFunction> {
        let abi_str = serde_json::to_string(&self.artifact.abi)
            .map_err(|e| anyhow!("Failed to serialize ABI: {}", e))?;

        DynAbiFunction::from_json_abi(&abi_str, function_name)
    }

    pub fn get_all_functions(&self) -> Result<Vec<String>> {
        let functions = self
            .artifact
            .abi
            .as_array()
            .ok_or_else(|| anyhow!("Invalid ABI format"))?
            .iter()
            .filter_map(|item| {
                if item.get("type").and_then(|t| t.as_str()) == Some("function") {
                    item.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(functions)
    }

    pub fn get_function_info(&self, function_name: &str) -> Result<(Vec<(String, String)>, Vec<(String, String)>)> {
        let abi_str = serde_json::to_string(&self.artifact.abi)
            .map_err(|e| anyhow!("Failed to serialize ABI: {}", e))?;

        let dyn_func = DynAbiFunction::from_json_abi(&abi_str, function_name)?;
        Ok((dyn_func.get_inputs(), dyn_func.get_outputs()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_deployment_manager() {
        let deployments_file = "deployments/testnet_sepolia.json";
        let artifact_dir = "artifacts";

        if Path::new(deployments_file).exists() && Path::new(artifact_dir).exists() {
            let records = DeploymentManager::load_deployments(deployments_file);
            assert!(records.is_ok());

            if let Ok(records) = records {
                assert!(!records.is_empty());
                assert_eq!(records[0].contract_name, "HelloWorld");
                assert_eq!(records[0].network, "testnet_sepolia");
            }
        }
    }

    #[test]
    fn test_get_deployed_contract() {
        let deployments_file = "deployments/testnet_sepolia.json";
        let artifact_dir = "artifacts";

        if Path::new(deployments_file).exists() && Path::new(artifact_dir).exists() {
            let deployed = DeploymentManager::get_deployed_contract(
                deployments_file,
                artifact_dir,
                "HelloWorld",
                "testnet_sepolia",
            );

            if deployed.is_ok() {
                let contract = deployed.unwrap();
                assert_eq!(contract.record.contract_name, "HelloWorld");
                assert_eq!(contract.record.network, "testnet_sepolia");
                assert!(contract.address().is_ok());
            }
        }
    }
}
