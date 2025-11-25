use tx_tx_tx::{core::config, sign_message, verify_message};

#[test]
fn test_sign_and_verify_round_trip() {
    use ethers::types::Address;
    use std::str::FromStr;
    
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let expected_address = Address::from_str("0x7e5f4552091a69125d5dfcb7b8c2659029395bdf").unwrap();
    let message = "Test message for round-trip";

    let signature = sign_message(private_key, message).expect("Failed to sign message");
    let address = verify_message(&signature, message, expected_address).expect("Failed to verify signature");

    assert_eq!(address, expected_address);
    println!("Round-trip test passed: {}", address);
}

#[test]
fn test_signature_format() {
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let message = "test";

    let signature = sign_message(private_key, message).expect("Failed to sign");

    assert!(signature.starts_with("0x"));
    assert_eq!(signature.len(), 132);

    let trimmed = signature.trim_start_matches("0x");
    assert!(hex::decode(trimmed).is_ok());
}

#[test]
fn test_multiple_messages_different_signatures() {
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";

    let sig1 = sign_message(private_key, "message1").expect("Failed to sign");
    let sig2 = sign_message(private_key, "message2").expect("Failed to sign");

    assert_ne!(sig1, sig2);
}

#[test]
fn test_verify_with_wrong_message_fails() {
    use ethers::types::Address;
    use std::str::FromStr;
    
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let expected_address = Address::from_str("0x7e5f4552091a69125d5dfcb7b8c2659029395bdf").unwrap();
    let message = "original";
    let wrong_message = "wrong";

    let signature = sign_message(private_key, message).expect("Failed to sign");
    let addr_original =
        verify_message(&signature, message, expected_address).expect("Failed to verify with original");
    let addr_wrong =
        verify_message(&signature, wrong_message, expected_address);

    assert_eq!(addr_original, expected_address);
    assert!(addr_wrong.is_err());
}

#[test]
fn test_load_private_key_from_env() {
    let result = config::load_private_key();
    assert!(result.is_ok(), "Failed to load private key from .env");

    let key = result.unwrap();
    assert!(!key.is_empty());
}

#[test]
fn test_hex_with_and_without_prefix() {
    let key_with_prefix = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let key_without_prefix = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let message = "test";

    let sig1 = sign_message(key_with_prefix, message).expect("Failed with prefix");
    let sig2 = sign_message(key_without_prefix, message).expect("Failed without prefix");

    assert_eq!(sig1, sig2);
}

#[test]
fn test_verify_signature_case_insensitive() {
    use ethers::types::Address;
    use std::str::FromStr;
    
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let expected_address = Address::from_str("0x7e5f4552091a69125d5dfcb7b8c2659029395bdf").unwrap();
    let message = "case test";

    let signature = sign_message(private_key, message).expect("Failed to sign");

    let sig_lower = signature.to_lowercase();
    let sig_upper = signature.to_uppercase();

    let addr_lower = verify_message(&sig_lower, message, expected_address).expect("Failed with lowercase");
    let addr_upper = verify_message(&sig_upper, message, expected_address).expect("Failed with uppercase");

    assert_eq!(addr_lower, addr_upper);
}

#[test]
fn test_recovery_with_different_messages() {
    use tx_tx_tx::core::crypto;
    use secp256k1::{Message, Secp256k1, SecretKey};
    use secp256k1::ecdsa::RecoverableSignature;
    
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&[1u8; 32]).unwrap();
    
    // Message 1
    let msg1_payload = crypto::prepare_message_for_signing("Hello World!");
    let digest1 = crypto::keccak256(&msg1_payload);
    let message1 = Message::from_slice(&digest1).unwrap();
    
    let sig = secp.sign_ecdsa_recoverable(&message1, &secret_key);
    let (recovery_id, sig_bytes) = sig.serialize_compact();
    
    // Recover with message 1
    let sig_struct = RecoverableSignature::from_compact(&sig_bytes, recovery_id).unwrap();
    let pk1 = secp.recover_ecdsa(&message1, &sig_struct);
    
    // Message 2
    let msg2_payload = crypto::prepare_message_for_signing("NEW MESSAGE");
    let digest2 = crypto::keccak256(&msg2_payload);
    let message2 = Message::from_slice(&digest2).unwrap();
    
    // Recover with message 2
    let pk2 = secp.recover_ecdsa(&message2, &sig_struct);
    
    println!("Recovery 1 ok: {}", pk1.is_ok());
    println!("Recovery 2 ok: {}", pk2.is_ok());
    
    if pk1.is_ok() && pk2.is_ok() {
        let key1 = pk1.unwrap().serialize_uncompressed();
        let key2 = pk2.unwrap().serialize_uncompressed();
        println!("Keys are same: {}", key1 == key2);
        println!("Key1: {}", hex::encode(&key1));
        println!("Key2: {}", hex::encode(&key2));
    }
}

#[test]
fn test_verify_with_wrong_message_debug() {
    use tx_tx_tx::features;
    use ethers::types::Address;
    use std::str::FromStr;
    
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let message1 = "hello";
    let message2 = "world";
    let expected_address = Address::from_str("0x7e5f4552091a69125d5dfcb7b8c2659029395bdf").unwrap();
    
    let signature = features::sign_message(private_key, message1).unwrap();
    println!("Signature: {}", signature);
    
    let result1 = features::verify_message(&signature, message1, expected_address);
    println!("Verify with message1: {:?}", result1);
    
    let result2 = features::verify_message(&signature, message2, expected_address);
    println!("Verify with message2: {:?}", result2);
}

#[test]
fn test_verify_ecdsa_directly() {
    use tx_tx_tx::core::crypto;
    use secp256k1::{Message, Secp256k1, SecretKey};
    use secp256k1::ecdsa::Signature;
    
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&[1u8; 32]).unwrap();
    
    // Sign message 1
    let msg1_payload = crypto::prepare_message_for_signing("hello");
    let digest1 = crypto::keccak256(&msg1_payload);
    let message1 = Message::from_slice(&digest1).unwrap();
    
    let sig = secp.sign_ecdsa(&message1, &secret_key);
    let sig_bytes = sig.serialize_compact();
    
    println!("Signed message1: hello");
    println!("Signature: {}", hex::encode(&sig_bytes));
    
    // Verify with message 1
    let sig_struct = Signature::from_compact(&sig_bytes).unwrap();
    let pk = secret_key.public_key(&secp);
    let verify1 = secp.verify_ecdsa(&message1, &sig_struct, &pk);
    println!("Verify1 (correct message): {:?}", verify1);
    
    // Now create message2 with different hash
    let msg2_payload = crypto::prepare_message_for_signing("world");
    let digest2 = crypto::keccak256(&msg2_payload);
    let message2 = Message::from_slice(&digest2).unwrap();
    
    println!("Message1 hash: {}", hex::encode(&digest1));
    println!("Message2 hash: {}", hex::encode(&digest2));
    println!("Hashes equal: {}", digest1 == digest2);
    
    // Verify with message 2
    let verify2 = secp.verify_ecdsa(&message2, &sig_struct, &pk);
    println!("Verify2 (wrong message): {:?}", verify2);
}
