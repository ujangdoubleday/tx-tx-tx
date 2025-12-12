# tx-tx-tx

EVM toolkit for signing, verification, transfers, and smart contract deployment.

## Quick Start with Docker (Recommended)

**Recommended for easy setup and deployment**

```bash
# Make the run script executable
chmod +x run

# Build the Docker image
./run build

# Setup your private key
./run secret

# Run interactive UI mode
./run run

# Or run CLI commands
ARGS="--help" ./run run
ARGS="sign --message 'Hello, World!'" ./run run
```

## Local Development

### Prerequisites

- [Rust](https://rust-lang.org/tools/install/)
- [Foundry](https://getfoundry.sh/)

### Setup

```bash
cp .env.example .env
```

Edit `.env` with your private key.

**Never use `.env.example` private key on mainnet**

### Build

```bash
make
```

## Docker Commands

### Basic Usage
```bash
# Build Docker image
./run build

# Setup private key (creates .eth_secret file)
./run secret

# Interactive UI mode
./run run

# CLI commands
ARGS="--help" ./run run
ARGS="sign --message 'Hello, World!'" ./run run
ARGS="transfer-eth --network testnet_sepolia --amount 0.01 --address 0x..." ./run run
ARGS="compile-sc" ./run run
ARGS="deploy --network testnet_sepolia --contract HelloWorld" ./run run
```

### Docker Swarm (Production)
```bash
# Deploy to Docker Swarm with secrets
./run swarm

# View logs
./run logs

# Remove stack
./run down
```

### Management
```bash
# Clean Docker resources
./run clean

# Remove existing Docker secret
./run secret-remove

# Show help
./run help
```

## Local Commands

### Interactive menu:
```bash
./tx
```

### Sign message:
```bash
./tx sign --message "Hello, World!"
```

### Verify signature:
```bash
./tx verify --message "Hello, World!" --signature 0x... --address 0x...
```

### Transfer ETH:
```bash
./tx transfer-eth --network testnet_sepolia --amount 0.01 --address 0x...
```

### Compile contracts:
```bash
./tx compile-sc                    # All contracts
./tx compile-sc --contract HelloWorld  # Specific contract
```

### Deploy contract:
```bash
./tx deploy --network testnet_sepolia --contract HelloWorld --gas-strategy standard
```

**Gas strategies:** `low`, `standard`, `fast`, `instant`

**Networks:** `ethereum_mainnet`, `testnet_sepolia` (add more in `data/networks.json`)

## License

See [LICENSE.md](LICENSE.md)
