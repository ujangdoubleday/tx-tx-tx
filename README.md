# tx-tx-tx

EVM toolkit for signing, verification, transfers, and smart contract deployment.

## Quick Start with Docker (Recommended)

```bash
# Make the run script executable
chmod +x docker

# Build the Docker image
./docker build

# Setup your private key
./docker secret

# Run interactive UI mode
./docker run
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

```bash
# Build Docker image
./docker build

# Setup private key (creates .eth_secret file)
./docker secret

# Run interactive UI mode
./docker run

# Clean Docker resources
./docker clean

# Show help
./docker help
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
