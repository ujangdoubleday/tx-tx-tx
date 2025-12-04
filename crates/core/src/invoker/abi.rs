use alloy_dyn_abi::{DynSolValue, JsonAbiExt, FunctionExt};
use alloy_json_abi::Function;
use alloy_primitives::Bytes;
use anyhow::{anyhow, Result};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct DynAbiFunction {
    function: Function,
}

impl DynAbiFunction {
    pub fn from_json_abi(abi: &str, function_name: &str) -> Result<Self> {
        let abi_json: Vec<Value> = serde_json::from_str(abi)
            .map_err(|e| anyhow!("Failed to parse ABI JSON: {}", e))?;

        for item in abi_json {
            if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                if name == function_name {
                    let func_json = serde_json::to_string(&item)
                        .map_err(|e| anyhow!("Failed to serialize function: {}", e))?;
                    
                    let func: Function = serde_json::from_str(&func_json)
                        .map_err(|e| anyhow!("Failed to parse function: {}", e))?;
                    
                    return Ok(DynAbiFunction { function: func });
                }
            }
        }

        Err(anyhow!(
            "Function '{}' not found in ABI",
            function_name
        ))
    }

    pub fn from_signature(signature: &str) -> Result<Self> {
        let func = Function::parse(signature)
            .map_err(|e| anyhow!("Failed to parse function signature: {:?}", e))?;
        
        Ok(DynAbiFunction { function: func })
    }

    pub fn encode_input(&self, args: &[DynSolValue]) -> Result<Bytes> {
        self.function
            .abi_encode_input(args)
            .map(Bytes::from)
            .map_err(|e| anyhow!("Failed to encode input: {:?}", e))
    }

    pub fn encode_call(&self, _name: &str, args: &[DynSolValue]) -> Result<Bytes> {
        self.function
            .abi_encode_input(args)
            .map(Bytes::from)
            .map_err(|e| anyhow!("Failed to encode call: {:?}", e))
    }



    pub fn decode_output(&self, data: &[u8]) -> Result<Vec<DynSolValue>> {
        self.function
            .abi_decode_output(data, false)
            .map_err(|e| anyhow!("Failed to decode output: {:?}", e))
    }

    pub fn get_inputs(&self) -> Vec<(String, String)> {
        self.function
            .inputs
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    p.ty.clone(),
                )
            })
            .collect()
    }

    pub fn get_outputs(&self) -> Vec<(String, String)> {
        self.function
            .outputs
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    p.ty.clone(),
                )
            })
            .collect()
    }
}

pub struct DynAbiInvoker;

impl DynAbiInvoker {
    pub fn invoke(
        abi: &str,
        function_name: &str,
        args: &[DynSolValue],
    ) -> Result<Bytes> {
        let dyn_func = DynAbiFunction::from_json_abi(abi, function_name)?;
        dyn_func.encode_call(function_name, args)
    }

    pub fn invoke_with_signature(
        signature: &str,
        args: &[DynSolValue],
    ) -> Result<Bytes> {
        let dyn_func = DynAbiFunction::from_signature(signature)?;
        
        dyn_func
            .function
            .abi_encode_input(args)
            .map(|calldata| {
                let selector = dyn_func.function.selector();
                let mut result = selector.to_vec();
                result.extend_from_slice(&calldata);
                result.into()
            })
            .map_err(|e| anyhow!("Failed to encode call: {:?}", e))
    }

    pub fn decode(
        abi: &str,
        function_name: &str,
        output: &[u8],
    ) -> Result<Vec<DynSolValue>> {
        let dyn_func = DynAbiFunction::from_json_abi(abi, function_name)?;
        dyn_func.decode_output(output)
    }

    pub fn get_function_info(
        abi: &str,
        function_name: &str,
    ) -> Result<(Vec<(String, String)>, Vec<(String, String)>)> {
        let dyn_func = DynAbiFunction::from_json_abi(abi, function_name)?;
        Ok((dyn_func.get_inputs(), dyn_func.get_outputs()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_with_signature() {
        use alloy_primitives::U256;
        
        let sig = "function transfer(address to, uint256 amount) returns (bool)";
        let dyn_func = DynAbiFunction::from_signature(sig).unwrap();
        
        let args = vec![
            DynSolValue::Address(
                "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
                    .parse()
                    .unwrap(),
            ),
            DynSolValue::Uint(U256::from(123u64), 256),
        ];

        let encoded = dyn_func.encode_call("transfer", &args);
        assert!(encoded.is_ok());
        
        let calldata = encoded.unwrap();
        assert!(calldata.len() >= 4);
    }

    #[test]
    fn test_function_info() {
        let sig = "function swap(uint256 amount0Out, uint256 amount1Out, address to, bytes data) external";
        let dyn_func = DynAbiFunction::from_signature(sig).unwrap();
        
        let inputs = dyn_func.get_inputs();
        assert_eq!(inputs.len(), 4);
        assert_eq!(inputs[0].0, "amount0Out");
        assert_eq!(inputs[1].0, "amount1Out");
    }
}
