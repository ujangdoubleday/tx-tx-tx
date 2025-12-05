# tx-tx-tx

EVM toolkit for signing, verification, transfers, and smart contract deployment.

## Prerequisites

- [Rust](https://rust-lang.org/tools/install/)
- [Foundry](https://getfoundry.sh/)

## Setup

```bash
cp .env.example .env
```

Edit `.env` with your private key.

**Never use `.env.example` private key on mainnet**

## Build

```bash
make
```

## Commands

Interactive menu:
```bash
./tx
```

Sign message:
```bash
./tx sign --message "Hello, World!"
```

Verify signature:
```bash
./tx verify --message "Hello, World!" --signature 0x... --address 0x...
```

Transfer ETH:
```bash
./tx transfer-eth --network testnet_sepolia --amount 0.01 --address 0x...
```

Compile all contracts:
```bash
./tx compile-sc
```

Compile specific contract:
```bash
./tx compile-sc --contract HelloWorld
```

Deploy contract:
```bash
./tx deploy --network testnet_sepolia --contract HelloWorld --gas-strategy standard
```

**Gas strategies:** `low`, `standard`, `fast`, `instant`

**Networks:** `ethereum_mainnet`, `testnet_sepolia` (add more in `data/networks.json`)

## License

See [LICENSE.md](LICENSE.md)
