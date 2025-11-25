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
cargo run -p node
```

Run the miner:
```bash
cargo run -p miner
```

Run the wallet:
```bash
cargo run -p wallet
```

## Development Status

This project is currently in early development. The basic workspace structure has been established with placeholder implementations for each component.

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
