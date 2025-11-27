# tx-tx-tx

EVM message signing, verification, and ETH transfer tool implementing EIP-191 `personal_sign` standard.

## Prerequisites

- [Rustc](https://rust-lang.org/tools/install/)
- [Foundry](https://getfoundry.sh/) - Required for smart contract development and building

## Build

```bash
make
```

For release build:
```bash
make release
```

## Setup

Create `.env` file from template:

```bash
cp .env.example .env
```

Edit `.env` with your private key:

```env
ETH_PRIVATE_KEY=0x0000000000000000000000000000000000000000000000000000000000000001
```

## Network Configuration

Networks are configured in `data/networks.json`. You can add custom networks by editing this file.

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

## Usage

### Interactive Mode

Run without arguments:

```bash
./tx-tx-tx
```

### CLI Commands

**Sign a message:**

```bash
./tx-tx-tx sign --message "Hello, World!"
```

**Verify a signature:**

```bash
./tx-tx-tx verify --message "Hello, World!" --signature 0x{signature_hex} --address 0x{address}
```

**Transfer ETH:**

```bash
./tx-tx-tx transfer-eth --network testnet_sepolia --amount 0.01 --address 0x{recipient_address}
```

With optional notes:

```bash
./tx-tx-tx transfer-eth --network testnet_sepolia --amount 0.01 --address 0x{recipient_address} --notes "payment for services"
```

**Available Networks:**

See the [Network Configuration](#network-configuration) section for available networks. Use the `id` field as the `--network` parameter.

## Security Warning

**NEVER use the example private key for real transactions!**

- The key in `.env.example` is publicly known and should only be used for testing
- For ETH transfers, use a secure private key with testnet funds first
- Always backup your private key securely
- Use hardware wallets or secure key management in production
- Test on Sepolia testnet before mainnet transactions
