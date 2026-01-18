# Node - Blockchain Network Node

A peer-to-peer blockchain node implementation for the Custom Decentralized Ledger project.

## Overview

The node component is responsible for:
- Maintaining the blockchain ledger
- Peer-to-peer network communication
- Transaction validation and propagation
- Block synchronization with other nodes
- Persistent blockchain storage

## Architecture

### Core Components

- **BLOCKCHAIN**: Globally shared blockchain state using `RwLock` for concurrent access
- **NODES**: Thread-safe map of connected peer nodes using `DashMap`
- **Handler**: Connection handling and message processing
- **Utilities**: Helper functions for blockchain management

### Module Structure

```
node/
├── src/
│   ├── lib.rs              # Global state and module definitions
│   ├── bin/
│   │   └── main.rs         # Main server entry point
│   ├── handler/
│   │   ├── mod.rs
│   │   └── connection.rs   # Connection handling
│   └── util/
│       ├── mod.rs
│       ├── chain_node.rs   # Node discovery and chain comparison
│       ├── cleanup.rs      # Connection cleanup
│       ├── cli.rs          # Command-line interface
│       ├── connections.rs  # Peer connection management
│       ├── download.rs     # Blockchain download
│       ├── load.rs         # Blockchain loading from disk
│       ├── save.rs         # Periodic blockchain saving
│       └── tests.rs        # Unit tests
└── tests/
    └── integration_tests.rs # Integration tests
```

## Usage

### Command-Line Arguments

```bash
cargo run --bin main -- [OPTIONS]

Options:
  -p, --port <PORT>                    Port to listen on [default: 9000]
  -b, --blockchain-file <FILE>         Path to the blockchain file (required)
  -n, --nodes <NODES>                  Comma-separated list of peer nodes
  -h, --help                           Print help
  -V, --version                        Print version
```

### Examples

#### Starting a Seed Node

Start the first node in the network:

```bash
cargo run --bin main -- --blockchain-file blockchain.cbor --port 9000
```

Or with logging:

```powershell
# PowerShell
$env:RUST_LOG="info"; cargo run --bin main -- --blockchain-file blockchain.cbor --port 9000
```

```bash
# Bash/Linux
RUST_LOG=info cargo run --bin main -- --blockchain-file blockchain.cbor --port 9000
```

#### Joining an Existing Network

Connect to existing nodes:

```bash
cargo run --bin main --blockchain-file blockchain.cbor --port 9001 --nodes localhost:9000
```

Connect to multiple nodes:

```bash
cargo run --bin main --blockchain-file blockchain.cbor --port 9002 --nodes localhost:9000, localhost:9001
```

## Behavior

### Startup Process

1. **Parse CLI arguments**: Port, blockchain file, and peer nodes
2. **Load or initialize blockchain**:
   - If blockchain file exists: Load from disk
   - Otherwise:
     - If nodes provided: Download from longest chain
     - If no nodes: Start as seed node with empty blockchain
3. **Start TCP listener**: Listen for incoming connections
4. **Accept connections**: Handle each connection in a separate task
5. **Background tasks**:
   - Periodic cleanup of stale connections
   - Periodic blockchain persistence to disk (every 15 seconds)

### Network Discovery

When joining a network, the node:
1. Connects to specified peer nodes
2. Sends `DiscoverNodes` message
3. Receives list of other nodes in the network
4. Establishes connections to discovered nodes
5. Finds the node with the longest blockchain
6. Downloads the complete blockchain from that node

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture
```

### Test Coverage

#### Unit Tests (`src/util/tests.rs`)
- ✅ CLI parsing with default values
- ✅ CLI parsing with custom port
- ✅ Blockchain file path handling
- ✅ Empty node list
- ✅ Single peer node
- ✅ Multiple peer nodes (comma-separated)

#### Integration Tests (`tests/integration_tests.rs`)
- ✅ Blockchain initialization
- ✅ Nodes map initialization
- ✅ Write lock acquisition and release
- ✅ Concurrent read access

## Dependencies

Key dependencies:
- `tokio`: Async runtime for networking
- `anyhow`: Error handling
- `clap`: Command-line argument parsing
- `dashmap`: Concurrent hashmap for node connections
- `btclib`: Core blockchain library
- `env_logger` & `log`: Logging functionality
- `static_init`: Global static initialization

## Development

### Building

```bash
# Debug build
cargo build --bin main

# Release build
cargo build --bin main --release
```

### Logging Levels

Set `RUST_LOG` to control verbosity:
- `error`: Only errors
- `warn`: Warnings and errors
- `info`: Informational messages (recommended)
- `debug`: Detailed debugging information
- `trace`: Very verbose tracing

Example:
```bash
RUST_LOG=debug cargo run --bin main -- --blockchain-file blockchain.cbor --port 9000
```

## Future Enhancements

- [ ] Implement proper consensus algorithm
- [ ] Add transaction mempool
- [ ] Implement block mining coordination
- [ ] Add metrics and monitoring
- [ ] Implement connection encryption
- [ ] Add rate limiting and DDoS protection
- [ ] Implement NAT traversal for broader network connectivity

## Troubleshooting

### Port Already in Use

If you see "Address already in use" error, either:
1. Choose a different port: `--port 9001`
2. Stop the existing process using that port

### Blockchain File Not Found

On first run, the blockchain file doesn't exist. The node will:
- Create a new empty blockchain if no peers are specified
- Download the blockchain from peers if nodes are provided

### Connection Refused to Peers

Ensure peer nodes are:
1. Running and listening
2. Reachable on the network
3. Specified with correct address format: `host:port`

## License

Part of the Custom Decentralized Ledger project.
