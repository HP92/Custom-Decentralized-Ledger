# Custom Decentralized Ledger

A Rust-based blockchain implementation designed to demonstrate and explain blockchain fundamentals for learning purposes.

## Project Overview

This project is a modular blockchain system built with Rust, showcasing core blockchain concepts including distributed ledger technology, mining, peer-to-peer networking, and wallet functionality.

## Architecture

The project is organized as a Cargo workspace with the following components:

### Core Library (`lib/`)
- **Purpose**: Shared blockchain core logic and data structures
- **Contains**: Common blockchain functionality used across all components
- **Status**: Foundation library for the blockchain system

### Node (`node/`)
- **Purpose**: Blockchain network node implementation
- **Responsibilities**: 
  - Maintaining the blockchain ledger
  - Peer-to-peer communication
  - Transaction validation and propagation
  - Block synchronization

### Miner (`miner/`)
- **Purpose**: Mining component for block creation
- **Responsibilities**:
  - Proof-of-work computation
  - Block assembly from pending transactions
  - Submitting mined blocks to the network

### Wallet (`wallet/`)
- **Purpose**: User wallet application
- **Responsibilities**:
  - Key pair generation and management
  - Transaction creation and signing
  - Balance tracking
  - Interaction with blockchain nodes

## Project Structure

```
Custom-Decentralized-Ledger/
├── cargo.toml          # Workspace configuration
├── README.md           # Project documentation
├── lib/                # Core blockchain library
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── node/               # Blockchain node
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── miner/              # Mining component
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── wallet/             # Wallet application
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## Getting Started

### Prerequisites
- Rust (latest stable version)
- Cargo

### Building the Project

Build all components:
```bash
cargo build
```

Build specific components:
```bash
cargo build -p lib
cargo build -p node
cargo build -p miner
cargo build -p wallet
```

### Running Components

Run the node:
```bash
# Start a seed node (first node in the network)
cargo run --bin main -- --blockchain-file blockchain.cbor --port 9000

# Start a node connecting to existing nodes
cargo run --bin main -- --blockchain-file blockchain.cbor --port 9001 --nodes localhost:9000

# With logging enabled (PowerShell)
$env:RUST_LOG="info"; cargo run --bin main -- --blockchain-file blockchain.cbor --port 9000

# With logging enabled (Bash/Linux)
RUST_LOG=info cargo run --bin main -- --blockchain-file blockchain.cbor --port 9000
```

Run the offline miner:
```bash
cargo run -p miner -- <block_file> <steps>
# Example:
cargo run -p miner -- my_block.cbor 1000
```

Run the offline miner:
```bash
cargo run -p miner -- <block_file> <steps>
# Example:
cargo run -p miner -- my_block.cbor 1000
```

Run the online miner:
```bash
cargo run --bin online_miner -- <address> <public_key_file>
# Example:
cargo run --bin online_miner -- localhost:9000 alice.pub.pem
```

Run the wallet:
```bash
cargo run -p wallet
```

### Library Utilities

The core library provides several utilities for testing and debugging:

```bash
# Generate a transaction
cargo run --bin tx_gen output.cbor

# Print transaction details
cargo run --bin tx_print tx.cbor

# Generate a block
cargo run --bin block_gen block.cbor

# Print block details
cargo run --bin block_print my_block.cbor

# Generate key pair
cargo run --bin key_gen mykey
# Creates: mykey.pub.pem and mykey.priv.cbor
```

For detailed logging output, set the `RUST_LOG` environment variable:

```bash
# PowerShell
$env:RUST_LOG="info"; cargo run -p miner -- my_block.cbor 1000

# Bash/Linux
RUST_LOG=info cargo run -p miner -- my_block.cbor 1000
```

## Development Status

This project is currently in active development. The node component is functional with:
- ✅ P2P network communication
- ✅ Blockchain synchronization
- ✅ Transaction handling
- ✅ Block validation
- ✅ Comprehensive test coverage
- ✅ Multi-node network support
- ✅ Online mining with network coordination

## Learning Goals

This project aims to demonstrate:
- Blockchain data structures (blocks, transactions, merkle trees)
- Cryptographic hashing and digital signatures
- Proof-of-work consensus mechanism
- Peer-to-peer networking
- Transaction validation and propagation
- Wallet key management
- Distributed ledger synchronization

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here]
