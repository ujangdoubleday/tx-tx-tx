use anyhow::Result;
use std::fs;
use std::path::Path;
use x_core::networks::Network;

pub struct Gate;

pub enum GateFeature {
    Deploy,
}

impl GateFeature {
    pub fn to_string(&self) -> String {
        match self {
            GateFeature::Deploy => "Deploy Smart Contract".to_string(),
        }
    }
}

impl Gate {
    pub fn new() -> Self {
        Gate
    }

    pub fn get_features() -> Vec<GateFeature> {
        vec![
            GateFeature::Deploy,
        ]
    }

    pub fn get_available_contracts() -> Result<Vec<String>> {
        let artifacts_path = "artifacts";
        
        if !Path::new(artifacts_path).exists() {
            return Err(anyhow::anyhow!("Artifacts folder not found at {}", artifacts_path));
        }

        let mut contracts = Vec::new();
        
        for entry in fs::read_dir(artifacts_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    if let Some(name_str) = dir_name.to_str() {
                        if name_str.ends_with(".sol") {
                            let contract_name = name_str
                                .trim_end_matches(".sol")
                                .to_string();
                            
                            let artifact_path = path.join(format!("{}.json", contract_name));
                            if artifact_path.exists() {
                                contracts.push(contract_name);
                            }
                        }
                    }
                }
            }
        }

        if contracts.is_empty() {
            return Err(anyhow::anyhow!("No compiled contracts found in {}", artifacts_path));
        }

        contracts.sort();
        Ok(contracts)
    }

    pub async fn execute_feature(network: &Network, feature: &GateFeature) -> Result<()> {
        match feature {
            GateFeature::Deploy => {
                println!("Deploy feature selected for {}", network.name);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_creation() {
        let _gate = Gate::new();
        assert_eq!(Gate::get_features().len(), 1);
    }
}
