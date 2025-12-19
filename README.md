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

## Testing

The project includes comprehensive unit and integration tests.

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific component
cargo test -p node
cargo test -p lib

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run tests with output
cargo test -- --nocapture
```

### Node Tests

The node component includes:
- **Unit tests**: Testing CLI parsing and configuration
- **Integration tests**: Testing blockchain state management and concurrent access

## Complete Network Setup Example

This section demonstrates how to set up a complete blockchain network with multiple nodes and miners.

### Scenario: Multi-Node Network with Online Miners

This example shows how to:
1. Generate cryptographic keys for miners (public/private key pairs)
2. Spawn multiple interconnected nodes
3. Connect online miners to nodes to mine blocks
4. Observe blockchain synchronization across the network

#### Prerequisites

Before starting, ensure you have built the project:

```bash
# Build all components
cargo build --release
```

This will create the necessary binaries in `target/release/` or `target/debug/` directory.

---

#### Step 1: Generate Mining Keys

Miners need public/private key pairs to receive mining rewards. The public key is used to create addresses where block rewards are sent.

**Generate keys for multiple miners:**

```bash
# Navigate to the project root directory
cd Custom-Decentralized-Ledger

# Generate keys for miner 1
cargo run --bin key_gen -- miner1
# Creates: 
#   - miner1.pub.pem (Public key certificate - needed for mining)
#   - miner1.priv.cbor (Private key - keep secure!)

# Generate keys for miner 2
cargo run --bin key_gen -- miner2
# Creates:
#   - miner2.pub.pem
#   - miner2.priv.cbor
```

> **Important Notes:**
> - The `.pub.pem` file is the **public key certificate** that miners will use when connecting to nodes
> - The `.priv.cbor` file is the **private key** used to sign transactions and prove ownership of coins
> - Keep your private keys secure - never share them!
> - The public key will be encoded into the coinbase transaction of mined blocks


---

#### Step 2: Start the Network Nodes

You'll need to start multiple blockchain nodes that will communicate with each other. Each node needs its own terminal window.

**Terminal 1 - Seed Node (Port 9000):**

The first node acts as the "seed node" - the initial entry point to the network.

```bash
# Navigate to the project root
cd Custom-Decentralized-Ledger

# PowerShell
$env:RUST_LOG="info"; cargo run --bin main -- --blockchain-file blockchain.cbor --port 9000

# Bash/Linux
RUST_LOG=info cargo run --bin main -- --blockchain-file blockchain.cbor --port 9000
```

**What to expect:**
- The node will create a new blockchain file (`blockchain.cbor`) if it doesn't exist
- You'll see: `[INFO] Node listening on 0.0.0.0:9000`
- The node is now ready to accept connections

---

**Terminal 2 - Node 2 (Port 9001):**

The second node connects to the first node to join the network.

```bash
# Open a new terminal and navigate to project root
cd Custom-Decentralized-Ledger

# PowerShell
$env:RUST_LOG="info"; cargo run --bin main -- --blockchain-file blockchain2.cbor --port 9001 --nodes localhost:9000

# Bash/Linux  
RUST_LOG=info cargo run --bin main -- --blockchain-file blockchain2.cbor --port 9001 --nodes localhost:9000
```

**What to expect:**
- Node 2 connects to Node 1 (at `localhost:9000`)
- You'll see: `[INFO] Node listening on 0.0.0.0:9001`
- The nodes exchange blockchain information and synchronize
- On Node 1's terminal, you'll see: `[INFO] Total amount of known nodes: 1`

---

**Terminal 3 - Node 3 (Port 9002):**

The third node connects to both existing nodes for redundancy.

```bash
# Open another new terminal
cd Custom-Decentralized-Ledger

# PowerShell
$env:RUST_LOG="info"; cargo run --bin main -- --blockchain-file blockchain3.cbor --port 9002 --nodes localhost:9000,localhost:9001

# Bash/Linux
RUST_LOG=info cargo run --bin main -- --blockchain-file blockchain3.cbor --port 9002 --nodes localhost:9000,localhost:9001
```

**What to expect:**
- Node 3 connects to both Node 1 and Node 2
- All three nodes discover each other through peer propagation
- Each node's terminal will show: `[INFO] Total amount of known nodes: 2`

> **Note on File Locking (Windows):**
> - If using Windows and `cargo run` is locking files, you can use pre-built binaries for Node 2 and Node 3:
>   ```powershell
>   # After building with cargo build, use the binary directly:
>   $env:RUST_LOG="info"; .\target\debug\main.exe --blockchain-file blockchain2.cbor --port 9001 --nodes localhost:9000
>   ```
> - On Linux/macOS, `cargo run` works fine for all nodes

---

#### Step 3: Start the Online Miners

Now that the network is running, connect miners that will compete to mine new blocks.

**Terminal 4 - Miner 1 (Connected to Node 1):**

```bash
# Open a new terminal
cd Custom-Decentralized-Ledger

# PowerShell
$env:RUST_LOG="info"; cargo run --bin online_miner -- localhost:9000 miner1.pub.pem

# Bash/Linux
RUST_LOG=info cargo run --bin online_miner -- localhost:9000 miner1.pub.pem
```

**What to expect:**
- Miner 1 connects to Node 1 at `localhost:9000`
- You'll see: `[INFO] Connecting to localhost:9000 to mine`
- The miner requests a block template from the node
- Mining begins: `[INFO] mining...` messages appear
- The miner uses `miner1.pub.pem` as the recipient address for block rewards

---

**Terminal 5 - Miner 2 (Connected to Node 2):**

```bash
# Open another terminal
cd Custom-Decentralized-Ledger

# PowerShell
$env:RUST_LOG="info"; cargo run --bin online_miner -- localhost:9001 miner2.pub.pem

# Bash/Linux
RUST_LOG=info cargo run --bin online_miner -- localhost:9001 miner2.pub.pem
```

**What to expect:**
- Miner 2 connects to Node 2 at `localhost:9001`
- Both miners now compete to find valid blocks
- The first miner to find a valid block wins the reward

> **Optional:** You can start additional miners connected to Node 3:
> ```bash
> cargo run --bin online_miner -- localhost:9002 miner1.pub.pem
> ```

---

#### Step 4: Observe the Network in Action


Once all components are running, monitor the terminals to observe blockchain behavior:

**Mining Activity:**
- **Miners:** Look for `[INFO] mining...` messages showing hash computation
- **Miners:** When a block is found: `[INFO] Block mined successfully!` 
- **Miners:** The miner submits the block to its connected node

**Block Propagation:**
- **Nodes:** When a node receives a mined block: `[INFO] received allegedly mined template`
- **Nodes:** After validation: `[INFO] block looks good, broadcasting`
- **Nodes:** The block is propagated to all connected peers
- **All Nodes:** Each node adds the block to its blockchain

**Network Synchronization:**
- Watch as the blockchain grows on all nodes simultaneously
- Each node maintains the same blockchain state
- Verify synchronization by checking block heights across nodes

**Peer Discovery:**
- **Nodes:** `[INFO] Total amount of known nodes: X` - shows network growth
- Nodes automatically discover and connect to peers through gossip

---

#### Step 5: Testing Transaction Flow (Optional)

You can submit transactions to the network to see them included in mined blocks:

```bash
# In a new terminal (Terminal 6)
cd Custom-Decentralized-Ledger

# Generate a transaction
cargo run --bin tx_gen -- test_transaction.cbor

# View the transaction details
cargo run --bin tx_print -- test_transaction.cbor
```

Transactions can be submitted to any node in the network and will:
1. Be validated by the receiving node
2. Propagate to all other nodes
3. Be included in the next mined block
4. Appear in all synchronized blockchains

---

#### Step 6: Monitoring and Verification

**Check Blockchain Files:**

Each node maintains its own blockchain file. After some blocks have been mined:

```bash
# View block details from Node 1's blockchain
cargo run --bin block_print -- blockchain.cbor

# Compare with Node 2's blockchain
cargo run --bin block_print -- blockchain2.cbor

# Compare with Node 3's blockchain  
cargo run --bin block_print -- blockchain3.cbor
```

All blockchains should be identical, showing successful synchronization.

**Key Metrics to Observe:**
- **Block Height:** Number of blocks in the chain (should be the same across all nodes)
- **Block Hashes:** Each block's hash (should match across all nodes)
- **Mined By:** Public key addresses of miners who found each block
- **Network Latency:** Time between block discovery and propagation

---

#### Step 7: Cleanup and Shutdown

To properly stop the network:

1. **Stop Miners:**
   - Press `Ctrl+C` in Terminal 4 (Miner 1)
   - Press `Ctrl+C` in Terminal 5 (Miner 2)

2. **Stop Nodes:**
   - Press `Ctrl+C` in Terminal 1 (Node 1)
   - Press `Ctrl+C` in Terminal 2 (Node 2)
   - Press `Ctrl+C` in Terminal 3 (Node 3)

3. **Verify Persistence:**
   - Blockchain files are automatically saved: `blockchain.cbor`, `blockchain2.cbor`, `blockchain3.cbor`
   - On next startup, nodes will load their saved blockchains
   - The network state is preserved across restarts

**Restarting the Network:**

When you restart the nodes later, they will:
- Load their existing blockchain from the `.cbor` files
- Reconnect to peers
- Synchronize any differences
- Continue mining from the current block height

---

### Troubleshooting

**Issue: Nodes not connecting**
- Verify ports are not already in use
- Check firewall settings
- Ensure node addresses in `--nodes` parameter are correct

**Issue: Miners not finding blocks**
- This is normal! Mining difficulty can make finding blocks take time
- Increase the number of miners to improve chances
- Monitor `[INFO] mining...` messages to confirm miners are working

**Issue: File locking errors (Windows)**
- Use pre-built binaries instead of `cargo run` for nodes 2 and 3
- Or close all cargo processes and rebuild

**Issue: Blockchain synchronization issues**
- Ensure all nodes start with empty blockchain files or compatible existing files
- Check network connectivity between nodes
- Verify all nodes are using the same genesis block

---

### Network Topology Visualization

```
                    ┌─────────────┐
                    │   Miner 1   │
                    │ (miner1.pub)│
                    └──────┬──────┘
                           │
                           │ connects to
                           ▼
    ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
    │   Node 1     │◄──┤   Node 2     │◄──┤   Node 3     │
    │  Port 9000   │──►│  Port 9001   │──►│  Port 9002   │
    │blockchain.cbor│   │blockchain2.cbor│  │blockchain3.cbor│
    └──────────────┘   └──────┬───────┘   └──────────────┘
                              │
                              │ connects to
                              ▼
                       ┌─────────────┐
                       │   Miner 2   │
                       │ (miner2.pub)│
                       └─────────────┘

Legend:
- ◄──► : P2P connections between nodes
- │ : Miner to node connection
- Each node maintains its own blockchain file
- All blockchains synchronize to the same state
```

---

#### Expected Behavior

Once the network is running, you should observe:

1. **Node Discovery**: Nodes discover each other and establish connections
1. **Node Discovery**: Nodes discover each other and establish persistent P2P connections
2. **Mining Competition**: Miners compete to solve proof-of-work puzzles and find valid blocks
3. **Block Creation**: When a miner finds a valid nonce, they create a block with:
   - A coinbase transaction sending rewards to their public key address
   - Any pending transactions from the network
   - A valid proof-of-work hash meeting difficulty requirements
4. **Block Submission**: The successful miner submits the block to their connected node
5. **Block Validation**: The receiving node validates:
   - Proof-of-work is correct
   - All transactions are valid
   - Block structure is correct
   - Previous block hash matches
6. **Block Propagation**: Validated blocks propagate to all nodes in the network via P2P gossip
7. **Blockchain Synchronization**: All nodes add the block to their blockchain, maintaining consensus
8. **Reward Distribution**: The winning miner's public key receives the block reward

#### Monitoring the Network

Watch the logs to see:
- `[INFO] Node listening on 0.0.0.0:XXXX` - Node is ready and accepting connections
- `[INFO] Total amount of known nodes: X` - Peer discovery and network growth
- `[INFO] Connecting to localhost:XXXX to mine` - Miner establishing connection
- `[INFO] mining...` - Proof-of-work computation in progress
- `[INFO] Block mined successfully!` - A miner found a valid block
- `[INFO] received allegedly mined template` - Node received a new block submission
- `[INFO] block looks good, broadcasting` - Block passed validation and is being propagated
- Hash rate information and difficulty adjustments

#### Testing Transaction Flow

You can submit transactions through the wallet to see them included in mined blocks:


```bash
# In another terminal
cargo run -p wallet
# Follow wallet prompts to create and submit transactions
```

The wallet will allow you to:
- Create new transactions
- Sign transactions with your private key
- Submit transactions to the network
- View transaction history

---

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
