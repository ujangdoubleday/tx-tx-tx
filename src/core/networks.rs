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
    let content = fs::read_to_string("data/networks.json")
        .map_err(|e| anyhow::anyhow!("Failed to read networks.json: {}", e))?;

    let networks: Vec<Network> = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse networks.json: {}", e))?;

    Ok(networks)
}

pub fn get_network_by_id<'a>(networks: &'a [Network], id: &str) -> Option<&'a Network> {
    networks.iter().find(|n| n.id == id)
}