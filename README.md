# tx-tx-tx

EVM message signing, verification, and ETH transfer tool implementing EIP-191 `personal_sign` standard.

## Prerequisites

- [Foundry](https://getfoundry.sh/) - Required for smart contract development and building

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

### Network Configuration

The app supports multiple EVM networks. Networks are configured in `data/networks.json`. You can add custom networks by editing this file.

**Network JSON template:**

```json
{
  "id": "your_network_id",
  "name": "Your Network Name",
  "chainId": 12345,
  "rpc": [
    "https://your-rpc-endpoint.com"
  ],
  "wsRpc": [
    "wss://your-ws-endpoint.com"
  ],
  "currency": {
    "name": "Ether",
    "symbol": "ETH",
    "decimals": 18
  },
  "blockExplorer": {
    "url": "https://your-explorer.com"
  }
}
```

**Example networks included:**
- Ethereum Mainnet (chainId: 1)
- Ethereum Sepolia Testnet (chainId: 11155111)

For development, use Sepolia testnet. Add your custom network to the array in `networks.json` to use it in the transfer menu.

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

### Interactive Mode

Run without arguments:

```bash
cargo run
```

## Running Tests

```bash
cargo test              # All tests
cargo test --lib       # Unit tests
cargo test --test integration  # Integration tests
cargo run --example example_sign  # Example
```

## Security Warning

**NEVER use the example private key for real transactions!**

- The key in `.env.example` is publicly known and should only be used for testing
- For ETH transfers, use a secure private key with testnet funds first
- Always backup your private key securely
- Use hardware wallets or secure key management in production
- Test on Sepolia testnet before mainnet transactions
