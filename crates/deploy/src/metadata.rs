use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetadata {
    pub contract_name: String,
    pub address: String,
    pub network: String,
    pub tx_hash: String,
    pub deployer: String,
    pub timestamp: u64,
}

pub struct MetadataManager;

impl MetadataManager {
    pub fn save_deployment(
        contract_name: &str,
        address: &str,
        network_id: &str,
        tx_hash: &str,
        deployer: &str,
    ) -> Result<()> {
        let deployments_dir = PathBuf::from("deployments");
        
        if !deployments_dir.exists() {
            fs::create_dir_all(&deployments_dir)?;
        }

        let filename = format!("{}.json", network_id);
        let filepath = deployments_dir.join(&filename);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let metadata = DeploymentMetadata {
            contract_name: contract_name.to_string(),
            address: address.to_string(),
            network: network_id.to_string(),
            tx_hash: tx_hash.to_string(),
            deployer: deployer.to_string(),
            timestamp,
        };

        let mut deployments: Vec<DeploymentMetadata> = if filepath.exists() {
            let content = fs::read_to_string(&filepath)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        deployments.push(metadata);

        let json = serde_json::to_string_pretty(&deployments)?;
        fs::write(&filepath, json)?;

        Ok(())
    }

    pub fn get_deployments(network_id: &str) -> Result<Vec<DeploymentMetadata>> {
        let deployments_dir = PathBuf::from("deployments");
        let filename = format!("{}.json", network_id);
        let filepath = deployments_dir.join(&filename);

        if !filepath.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&filepath)?;
        let deployments = serde_json::from_str(&content)?;
        Ok(deployments)
    }
}
