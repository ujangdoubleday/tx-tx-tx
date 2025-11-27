use anyhow::Result;
use ethers::prelude::*;
use ethers::abi::Abi;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractArtifact {
    pub abi: serde_json::Value,
    pub bytecode: serde_json::Value,
    #[serde(rename = "deployedBytecode")]
    pub deployed_bytecode: Option<serde_json::Value>,
}

pub struct ArtifactLoader;

impl ArtifactLoader {
    pub fn load_artifact<P: AsRef<Path>>(path: P) -> Result<ContractArtifact> {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read artifact: {}", e))?;

        let artifact: ContractArtifact = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse artifact: {}", e))?;

        Ok(artifact)
    }

    pub fn get_bytecode(artifact: &ContractArtifact) -> Result<Bytes> {
        let bytecode_obj = &artifact.bytecode;

        let bytecode_str = if let Some(object) = bytecode_obj.get("object") {
            object.as_str().ok_or_else(|| {
                anyhow::anyhow!("Bytecode object is not a string")
            })?
        } else if bytecode_obj.is_string() {
            bytecode_obj.as_str().ok_or_else(|| {
                anyhow::anyhow!("Bytecode is not a string")
            })?
        } else {
            return Err(anyhow::anyhow!("Invalid bytecode format"));
        };

        let bytes = hex::decode(bytecode_str.trim_start_matches("0x"))
            .map_err(|e| anyhow::anyhow!("Failed to decode bytecode: {}", e))?;

        Ok(Bytes::from(bytes))
    }

    pub fn get_abi(artifact: &ContractArtifact) -> Result<Abi> {
        let abi_json = &artifact.abi;
        let abi = serde_json::from_value(abi_json.clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse ABI: {}", e))?;

        Ok(abi)
    }
}
