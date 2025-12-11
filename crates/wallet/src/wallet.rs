use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub id: String,
    pub privatekey: String,
    pub address: String,
}

impl Wallet {
    pub fn new(id: String, privatekey: String, address: String) -> Self {
        Self {
            id,
            privatekey,
            address,
        }
    }
}
