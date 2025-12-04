use alloy_dyn_abi::{DynSolValue, Word};
use alloy_primitives::{Address, U256, I256, Sign};
use anyhow::{anyhow, Result};

pub struct Codec;

impl Codec {
    pub fn parse_value(input: &str, type_str: &str) -> Result<DynSolValue> {
        let trimmed = input.trim();

        if type_str.contains("string") {
            Ok(DynSolValue::String(trimmed.to_string()))
        } else if type_str.contains("bytes32") {
            let hex_str = if trimmed.starts_with("0x") {
                &trimmed[2..]
            } else {
                trimmed
            };

            let bytes = hex::decode(hex_str)
                .map_err(|_| anyhow!("Invalid hex format for bytes32"))?;

            if bytes.len() != 32 {
                anyhow::bail!("bytes32 must be exactly 32 bytes");
            }

            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Ok(DynSolValue::FixedBytes(Word::from(arr), 32))
        } else if type_str.contains("bytes") {
            let hex_str = if trimmed.starts_with("0x") {
                &trimmed[2..]
            } else {
                trimmed
            };
            let bytes = hex::decode(hex_str)
                .map_err(|_| anyhow!("Invalid hex format for bytes"))?;
            Ok(DynSolValue::Bytes(bytes.into()))
        } else if type_str.contains("uint256") || type_str.contains("uint") {
            let val: u128 = trimmed
                .parse()
                .map_err(|_| anyhow!("Invalid number format for {}", type_str))?;
            Ok(DynSolValue::Uint(U256::from(val), 256))
        } else if type_str.contains("int256") || type_str.contains("int") {
            let val: i128 = trimmed
                .parse()
                .map_err(|_| anyhow!("Invalid number format for {}", type_str))?;
            let sign = if val < 0 { Sign::Negative } else { Sign::Positive };
            let abs_val = U256::from(val.abs() as u128);
            let i256_val = I256::checked_from_sign_and_abs(sign, abs_val)
                .ok_or_else(|| anyhow!("Value out of range for int256"))?;
            Ok(DynSolValue::Int(i256_val, 256))
        } else if type_str.contains("bool") {
            let val = trimmed.to_lowercase();
            let b = val == "true" || val == "1" || val == "yes";
            Ok(DynSolValue::Bool(b))
        } else if type_str.contains("address") {
            let addr: Address = trimmed
                .parse()
                .map_err(|_| anyhow!("Invalid address format"))?;
            Ok(DynSolValue::Address(addr))
        } else {
            Err(anyhow!("Unsupported type: {}", type_str))
        }
    }

    pub fn format_value(value: &DynSolValue, _type_str: &str) -> String {
        match value {
            DynSolValue::String(s) => s.clone(),
            DynSolValue::Bool(b) => b.to_string(),
            DynSolValue::Uint(u, _) => u.to_string(),
            DynSolValue::Int(i, _) => i.to_string(),
            DynSolValue::Address(a) => format!("{:#x}", a),
            DynSolValue::Bytes(b) => format!("0x{}", hex::encode(b)),
            DynSolValue::FixedBytes(b, _) => format!("0x{}", hex::encode(b.as_slice())),
            DynSolValue::Array(arr) => {
                let formatted_items: Vec<String> = arr.iter()
                    .map(|v| Codec::format_value(v, ""))
                    .collect();
                format!("[{}]", formatted_items.join(", "))
            }
            DynSolValue::Tuple(tuple) => {
                let formatted_items: Vec<String> = tuple.iter()
                    .map(|v| Codec::format_value(v, ""))
                    .collect();
                format!("({})", formatted_items.join(", "))
            }
            DynSolValue::FixedArray(arr) => {
                let formatted_items: Vec<String> = arr.iter()
                    .map(|v| Codec::format_value(v, ""))
                    .collect();
                format!("[{}]", formatted_items.join(", "))
            }
            DynSolValue::Function(_) => "[Function Pointer]".to_string(),
        }
    }

    pub fn format_values(values: &[DynSolValue], types: &[(String, String)]) -> Vec<(String, String)> {
        values
            .iter()
            .zip(types.iter())
            .map(|(value, (name, ty))| {
                let formatted = Codec::format_value(value, ty);
                (name.clone(), formatted)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let result = Codec::parse_value("hello", "string");
        assert!(result.is_ok());
        if let Ok(DynSolValue::String(s)) = result {
            assert_eq!(s, "hello");
        }
    }

    #[test]
    fn test_parse_uint() {
        let result = Codec::parse_value("123", "uint256");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_bool() {
        let result = Codec::parse_value("true", "bool");
        assert!(result.is_ok());
        if let Ok(DynSolValue::Bool(b)) = result {
            assert!(b);
        }
    }

    #[test]
    fn test_parse_address() {
        let result = Codec::parse_value("0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826", "address");
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_string() {
        let val = DynSolValue::String("hello".to_string());
        let formatted = Codec::format_value(&val, "string");
        assert_eq!(formatted, "hello");
    }

    #[test]
    fn test_format_uint() {
        let val = DynSolValue::Uint(U256::from(123u64), 256);
        let formatted = Codec::format_value(&val, "uint256");
        assert_eq!(formatted, "123");
    }

    #[test]
    fn test_format_address() {
        let addr: Address = "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826".parse().unwrap();
        let val = DynSolValue::Address(addr);
        let formatted = Codec::format_value(&val, "address");
        assert!(formatted.starts_with("0x"));
    }
}
