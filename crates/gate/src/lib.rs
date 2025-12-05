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
        let contracts_path = "contracts";
        let artifacts_path = "artifacts";
        
        if !Path::new(contracts_path).exists() {
            return Err(anyhow::anyhow!("Contracts folder not found at {}", contracts_path));
        }

        if !Path::new(artifacts_path).exists() {
            return Err(anyhow::anyhow!("Artifacts folder not found at {}", artifacts_path));
        }

        let mut contracts = Vec::new();
        
        for entry in fs::read_dir(contracts_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        if name_str.ends_with(".sol") {
                            let contract_name = name_str
                                .trim_end_matches(".sol")
                                .to_string();
                            
                            let artifact_path = Path::new(artifacts_path)
                                .join(format!("{}.sol", contract_name))
                                .join(format!("{}.json", contract_name));
                            
                            if artifact_path.exists() {
                                contracts.push(contract_name);
                            }
                        }
                    }
                }
            }
        }

        if contracts.is_empty() {
            return Err(anyhow::anyhow!("No compiled contracts found in {}", contracts_path));
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
