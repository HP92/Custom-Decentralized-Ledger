# Miner - Proof-of-Work Mining Component

The mining component responsible for creating new blocks through proof-of-work computation for the Custom Decentralized Ledger project.

## Overview

The miner implements the proof-of-work consensus mechanism, performing cryptographic hash computations to find valid block hashes that meet the current difficulty target.

## Features

- **Proof-of-Work Mining**: Computes nonces to find valid block hashes below the difficulty target
- **Incremental Mining**: Configurable mining steps for different hardware capabilities
- **Block Validation**: Ensures mined blocks meet all consensus rules
- **Offline and Online Mining**: Supports both standalone and network-connected mining
- **Detailed Logging**: Comprehensive logging of mining progress and results

## Project Structure

```
miner/
├── Cargo.toml              # Package dependencies and metadata
├── README.md               # This file
├── my_block.cbor           # Example block template
├── alice.pub.pem           # Example public key for mining rewards
└── src/
    └── bin/
        ├── offline_miner.rs    # Offline miner (standalone)
        └── online_miner.rs     # Online miner (network-connected)

> **Note:** The offline miner is implemented as a binary target (`offline_miner`). To run it, use `cargo run -p miner --bin offline_miner -- ...` instead of `cargo run -p miner -- ...`. If you want to use `cargo run -p miner -- ...` directly, set `offline_miner` as the default binary in your `Cargo.toml`.
```

## Mining Modes

### 1. Offline Miner (Standalone)

Mines a single block from a block template file without network connectivity. Useful for:
- Testing mining logic
- Development and debugging
- Educational purposes
- Performance benchmarking

**Usage:**
```bash
cargo run -p miner -- <block_file> <steps>
```

**Arguments:**
- `<block_file>`: Path to the block template file (CBOR format)
- `<steps>`: Number of hash iterations per mining step (adjust based on your CPU speed)

**Example:**
```bash
cargo run -p miner -- my_block.cbor 1000
```

**With logging:**
```bash
# PowerShell
$env:RUST_LOG="info"; cargo run -p miner -- my_block.cbor 1000

# Bash/Linux
RUST_LOG=info cargo run -p miner -- my_block.cbor 1000
```

**Output:**
The miner will:
1. Load the block template from the specified file
2. Display the original block and its hash
3. Mine the block in increments (showing "mining..." progress)
4. Display the final mined block and its hash

### 2. Online Miner (Network-Connected)

Connects to a blockchain node and mines blocks collaboratively over the network.

**Usage:**
```bash
cargo run --bin online_miner -- <address> <public_key_file>
```

**Arguments:**
- `<address>`: Network address of the blockchain node (e.g., `localhost:9000`)
- `<public_key_file>`: Path to your public key file for receiving mining rewards

**Example:**
```bash
# PowerShell
$env:RUST_LOG="info"; cargo run --bin online_miner -- localhost:9000 alice.pub.pem

# Bash/Linux
RUST_LOG=info cargo run --bin online_miner -- localhost:9000 alice.pub.pem
```

## Mining Process

### Offline Mining Flow

1. **Load Block Template**: Read block from file system
2. **Clone Original**: Keep copy of original block for comparison
3. **Initialize Mining**: Get mutable reference to block header
4. **Proof-of-Work Loop**:
   - Call `header.mine(steps)` for incremental mining
   - Log progress after each iteration
   - Continue until valid hash is found
5. **Display Results**: Print original and mined blocks with their hashes

### Mining Algorithm

The mining algorithm (implemented in `btclib::BlockHeader`):

```rust
pub fn mine(&mut self, steps: usize) -> bool {
    for _ in 0..steps {
        // Increment nonce
        self.nonce += 1;
        
        // Calculate hash
        let hash = self.hash();
        
        // Check if hash meets target
        if hash.matches_target(&self.target) {
            return true;  // Valid block found!
        }
    }
    false  // Need more iterations
}
```

## Configuration

### Choosing the Right Step Count

The `steps` parameter controls mining iterations per progress update:

- **Slow CPU**: Use smaller values (100-1000)
  - More frequent progress updates
  - Better responsiveness
  - Longer overall mining time

- **Fast CPU**: Use larger values (10000-100000)
  - Fewer progress updates
  - Better performance
  - Faster overall mining time

**Tip**: Start with 1000 and adjust based on how frequently you see "mining..." messages.

### Difficulty Target

Mining difficulty is determined by the `target` field in the block header:
- Lower target = harder difficulty = more time to mine
- Higher target = easier difficulty = less time to mine

The target is automatically adjusted by the blockchain based on:
- `IDEAL_BLOCK_TIME`: 10 seconds
- `DIFFICULTY_UPDATE_INTERVAL`: Every 50 blocks

## Block Rewards

Mining rewards follow a Bitcoin-style halving schedule:

| Blocks | Reward per Block |
|--------|------------------|
| 0-209 | 50 BTC |
| 210-419 | 25 BTC |
| 420-629 | 12.5 BTC |
| 630-839 | 6.25 BTC |
| ... | ... |

**Formula**: `INITIAL_REWARD / 2^(block_height / HALVING_INTERVAL)`

Where:
- `INITIAL_REWARD` = 50 BTC
- `HALVING_INTERVAL` = 210 blocks

## Example Workflow

### 1. Generate Mining Keys

```bash
cd lib
cargo run --bin key_gen alice
# Creates: alice.pub.pem and alice.priv.cbor
```

### 2. Create Block Template

```bash
cargo run --bin block_gen ../miner/my_block.cbor
```

### 3. Mine the Block

```bash
cd ../miner
cargo run -p miner -- my_block.cbor 1000
```

**Example output:**
```
[INFO] original: Block {
    header: BlockHeader {
        timestamp: 1702483200,
        nonce: 0,
        prev_block_hash: Hash(0x0000...),
        merkle_root: Hash(0xabcd...),
        target: U256([0xFFFF...])
    },
    transactions: [...]
}
[INFO] hash: Hash(0xffff1234...)
[INFO] mining...
[INFO] mining...
[INFO] mining...
[INFO] final: Block {
    header: BlockHeader {
        timestamp: 1702483200,
        nonce: 123456,
        prev_block_hash: Hash(0x0000...),
        merkle_root: Hash(0xabcd...),
        target: U256([0xFFFF...])
    },
    transactions: [...]
}
[INFO] hash: Hash(0x00001a2b...)
```

## Dependencies

From `Cargo.toml`:

```toml
[dependencies]
btclib = { path = "../lib" }
clap = "4.5"
env_logger = "0.11"
log = "0.4"
tokio = { version = "1", features = ["full"] }  # For online miner
```

## Troubleshooting

### Issue: Mining takes too long

**Solutions:**
- Increase the `steps` parameter (e.g., from 1000 to 10000)
- Use a faster CPU
- Check if the difficulty target is reasonable

### Issue: No log output in PowerShell

**Solution:**
Set the `RUST_LOG` environment variable before running:
```powershell
$env:RUST_LOG="info"
cargo run -p miner -- my_block.cbor 1000
```

### Issue: "Failed to load block" error

**Solutions:**
- Verify the block file path is correct
- Ensure the block file is in CBOR format
- Generate a new block template using `block_gen`

### Issue: Online miner can't connect

**Solutions:**
- Verify the node is running at the specified address
- Check firewall settings
- Ensure the address format is correct (e.g., `localhost:9000`)

## Development Status

Currently implemented:
- ✅ Offline proof-of-work mining
- ✅ Incremental mining with configurable steps
- ✅ Block validation
- ✅ Command-line argument parsing with clap
- ✅ Comprehensive logging
- ✅ Online miner structure (in progress)

Planned features:
- 🔄 Full network integration for online mining
- 🔄 Mining pool support
- 🔄 GPU mining support
- 🔄 Performance optimizations
- 🔄 Mining statistics dashboard

## Testing

Run miner tests:

```bash
cargo test -p miner
```

Run with verbose output:

```bash
cargo test -p miner -- --nocapture
```

## Performance Tips

1. **Optimize step count**: Find the sweet spot for your hardware
2. **Use release builds**: `cargo run --release -p miner -- my_block.cbor 10000`
3. **Monitor CPU usage**: Ensure miner is utilizing available resources
4. **Reduce logging**: Use `RUST_LOG=warn` for less output overhead

## License

[Add your license information here]

## Contributing

Contributions are welcome! Areas for improvement:
- Mining algorithm optimizations
- Network protocol implementation
- Mining pool integration
- Documentation improvements

[Add contribution guidelines here]
