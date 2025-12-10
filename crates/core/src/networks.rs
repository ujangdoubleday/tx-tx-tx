use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub id: String,
    pub name: String,
    #[serde(rename = "chainId")]
    pub chain_id: u64,
    pub rpc: Vec<String>,
    #[serde(rename = "wsRpc")]
    pub ws_rpc: Vec<String>,
    pub currency: Currency,
    #[serde(rename = "blockExplorer")]
    pub block_explorer: BlockExplorer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockExplorer {
    pub url: String,
}

pub fn load_networks() -> Result<Vec<Network>> {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.parent().unwrap().parent().unwrap();
    let networks_path = project_root.join("data/networks.json");
    
    let content = fs::read_to_string(&networks_path)
        .map_err(|e| anyhow::anyhow!("Failed to read networks.json at {}: {}", networks_path.display(), e))?;

    let networks: Vec<Network> = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse networks.json: {}", e))?;

    Ok(networks)
}

pub fn get_network_by_id<'a>(networks: &'a [Network], id: &str) -> Option<&'a Network> {
    networks.iter().find(|n| n.id == id)
}
