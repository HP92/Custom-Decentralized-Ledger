# BtcLib - Core Blockchain Library

The core library providing fundamental blockchain data structures, cryptographic primitives, and utilities for the Custom Decentralized Ledger project.

## Overview

BtcLib implements the foundational components required for a blockchain system, including:
- Block and transaction data structures
- Cryptographic operations (signing, verification, hashing)
- UTXO (Unspent Transaction Output) management
- Proof-of-work mining
- Merkle tree calculations
- Blockchain state management

## Project Structure

```
lib/
├── Cargo.toml              # Package dependencies and metadata
├── tx.cbor                 # Example transaction in CBOR format
├── README.md               # This file
└── src/
    ├── lib.rs             # Main library entry point and constants
    ├── error.rs           # Error types and Result definitions
    ├── bin/               # Binary utilities for testing
    │   ├── block_gen.rs   # Generate sample blocks
    │   ├── block_print.rs # Print block contents
    │   ├── tx_gen.rs      # Generate sample transactions
    │   └── tx_print.rs    # Print transaction contents
    ├── crypto/            # Cryptographic primitives
    │   ├── mod.rs
    │   ├── private_key.rs # ECDSA private key implementation
    │   ├── public_key.rs  # ECDSA public key implementation
    │   └── signature.rs   # Digital signature operations
    ├── custom_sha_types/  # SHA-256 hashing wrapper
    │   ├── mod.rs
    │   └── hash.rs        # Hash type with target matching
    ├── types/             # Core blockchain data structures
    │   ├── mod.rs
    │   ├── block.rs       # Block structure and validation
    │   ├── block_header.rs # Block header with mining
    │   ├── blockchain.rs  # Blockchain state and UTXO management
    │   ├── transaction.rs # Transaction structure
    │   ├── transaction_input.rs  # Transaction inputs
    │   └── transaction_output.rs # Transaction outputs
    └── utils/             # Utility modules
        ├── mod.rs
        ├── merkle_root.rs # Merkle tree root calculation
        └── saveable.rs    # Serialization trait for persistence
```

## Core Components

### Types ([`src/types/`](src/types/))

#### [`Block`](src/types/block.rs)
Complete block containing a header and transactions. Implements:
- Transaction verification
- Coinbase transaction validation
- Miner fee calculation
- CBOR serialization/deserialization

#### [`BlockHeader`](src/types/block_header.rs)
Block metadata and proof-of-work:
- `timestamp`: Block creation time
- `nonce`: Mining nonce
- `prev_block_hash`: Previous block hash
- `merkle_root`: Merkle root of transactions
- `target`: Difficulty target
- `mine()`: Performs proof-of-work mining

#### [`Blockchain`](src/types/blockchain.rs)
Maintains blockchain state:
- UTXO set management
- Dynamic difficulty adjustment
- Mempool for pending transactions
- Block validation and addition
- Target recalculation every `DIFFICULTY_UPDATE_INTERVAL` blocks

#### [`Transaction`](src/types/transaction.rs)
Represents value transfers with inputs and outputs. Supports CBOR serialization.

#### [`TransactionInput`](src/types/transaction_input.rs)
References a previous transaction output with a signature for authorization.

#### [`TransactionOutput`](src/types/transaction_output.rs)
Defines a spendable output:
- `value`: Amount in satoshis
- `unique_id`: UUID for uniqueness
- `pubkey`: Owner's public key

### Cryptography ([`src/crypto/`](src/crypto/))

Built on `k256` (secp256k1 curve) and `ecdsa`:

- [`PrivateKey`](src/crypto/private_key.rs): ECDSA signing key with custom serde serialization
- [`PublicKey`](src/crypto/public_key.rs): ECDSA verification key
- [`Signature`](src/crypto/signature.rs): Digital signatures with `sign_output()` and `verify()` methods

### Hashing ([`src/custom_sha_types/`](src/custom_sha_types/))

- [`Hash`](src/custom_sha_types/hash.rs): SHA-256 hash wrapper with:
  - CBOR-based serialization for hashing
  - `matches_target()`: Proof-of-work validation
  - U256 internal representation

### Utilities ([`src/utils/`](src/utils/))

- [`MerkleRoot`](src/utils/merkle_root.rs): Calculates Merkle root from transaction list
- [`Saveable`](src/utils/saveable.rs): Trait for CBOR file persistence with `load()`, `save()`, `load_from_file()`, and `save_to_file()`

## Constants ([`src/lib.rs`](src/lib.rs))

| Constant | Value | Description |
|----------|-------|-------------|
| `INITIAL_REWARD` | 50 | Initial block reward in BTC |
| `HALVING_INTERVAL` | 210 | Blocks between reward halvings |
| `IDEAL_BLOCK_TIME` | 10 | Target block time in seconds |
| `MIN_TARGET` | `U256([0xFFFF...])` | Minimum difficulty target |
| `DIFFICULTY_UPDATE_INTERVAL` | 50 | Blocks between difficulty adjustments |
| `MAX_MEMPOOL_TX_AGE` | 600 | Maximum transaction age in mempool (10 minutes) |

## Binary Utilities

Located in [`src/bin/`](src/bin/):

- **`tx_gen`**: Generate a sample transaction
  ```bash
  cargo run --bin tx_gen <output_file>
  ```

- **`tx_print`**: Display transaction contents
  ```bash
  cargo run --bin tx_print <tx_file>
  ```

- **`block_gen`**: Generate a sample block
  ```bash
  cargo run --bin block_gen 