# tx-tx-tx

EVM message signing and verification tool implementing EIP-191 `personal_sign` standard.

## Quick Start

### Build

```bash
cargo build --release
```

### Setup

Create `.env` file from template:

```bash
cp .env.example .env
```

Edit `.env` with your private key:

```env
ETH_PRIVATE_KEY=0x0000000000000000000000000000000000000000000000000000000000000001
```

## Usage

### CLI Commands

**Sign a message:**

```bash
cargo run -- sign --message "Hello, World!"
```

**Verify a signature:**

```bash
cargo run -- verify --message "Hello, World!" --signature 0x{signature_hex} --address 0x{address}
```

### Library API

```rust
use tx_tx_tx::{sign_message, verify_message, config};

fn main() -> anyhow::Result<()> {
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let message = "Hello";

    let signature = sign_message(private_key, message)?;
    let expected_address = "0x5ed9d0e08da37ce9aee1ac9a0d3a95b1a4c6e2ed".parse()?;
    let address = verify_message(&signature, message, expected_address)?;

    println!("Signature: {}", signature);
    println!("Address: {}", address);

    Ok(())
}
```

## Running Tests

```bash
cargo test              # All tests
cargo test --lib       # Unit tests
cargo test --test integration  # Integration tests
cargo run --example example_sign  # Example
```

## ⚠️ Warning

**The private key in `.env` is for testing/development only!**

- This key is publicly known and should NEVER be used for real transactions
- All test keys in this project are randomly generated for demonstration
- For production use, generate your own private key and keep it secure
- Use environment variables, vaults, or HSM in production
