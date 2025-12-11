#[path = "handlers/utils.rs"]
mod utils;

#[path = "handlers/signature.rs"]
mod signature;

#[path = "handlers/transfer.rs"]
mod transfer;

#[path = "handlers/compile.rs"]
mod compile;

#[path = "handlers/gate.rs"]
mod gate;

#[path = "handlers/invoker.rs"]
mod invoker;

#[path = "handlers/wallet.rs"]
mod wallet;

pub use utils::clear_screen;
pub use signature::{handle_sign, handle_verify};
pub use transfer::handle_transfer_sepolia;
pub use compile::handle_compile_smart_contracts;
pub use gate::{handle_gate_mainnet, handle_gate_sepolia, handle_gate_deploy};
pub use invoker::handle_smart_contract_invoker;
pub use wallet::handle_generate_wallet;
