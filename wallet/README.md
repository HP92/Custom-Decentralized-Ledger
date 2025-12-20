# Wallet Module

A user-friendly wallet application for interacting with the Custom Decentralized Ledger blockchain network.

## Overview

The wallet module provides a complete solution for managing cryptocurrency wallets, creating and signing transactions, tracking balances, and interacting with blockchain nodes. It handles key management, UTXO (Unspent Transaction Output) tracking, fee calculation, and transaction broadcasting.

## Features

- 🔐 **Key Management**: Generate and manage multiple cryptographic key pairs
- 💰 **Balance Tracking**: Monitor your wallet balance by tracking UTXOs
- 📤 **Transaction Creation**: Build, sign, and broadcast transactions to the network
- 👥 **Contact Management**: Save recipient addresses with friendly names
- ⚙️ **Fee Configuration**: Support for fixed and percentage-based transaction fees
- 🌐 **Node Connectivity**: Connect to blockchain nodes for UTXO queries and transaction submission
- 💾 **Persistent Storage**: Configuration and keys stored in TOML and CBOR formats
- 🧪 **Comprehensive Testing**: 51 unit tests with 92%+ code coverage

## Architecture

### Core Components

1. **Core** (`core.rs`) - Main wallet engine
   - Manages wallet state and operations
   - Handles network communication with nodes
   - Creates and signs transactions
   - Calculates balances and fees

2. **Config** (`config.rs`) - Wallet configuration
   - Stores wallet settings (keys, contacts, node address, fees)
   - Serializable to TOML format

3. **UtxoStore** (`utxo_store.rs`) - UTXO management
   - Tracks unspent transaction outputs
   - Marks UTXOs as spent/unspent
   - Thread-safe concurrent access via `Arc<SkipMap>`

4. **Key Management**
   - `Key` - File paths to public/private key pairs
   - `LoadedKey` - In-memory cryptographic keys
   - Keys are stored in PEM (public) and CBOR (private) formats

5. **Fee Configuration**
   - `FeeType` - Fixed or percentage-based fees
   - `FeeConfig` - Fee calculation settings

6. **Recipients**
   - `Recipient` - Saved contact addresses
   - `LoadedRecipient` - In-memory recipient data

## Installation

The wallet is part of the Custom-Decentralized-Ledger workspace:

```bash
# Build the wallet
cargo build -p wallet

# Run the wallet (CLI interface)
cargo run -p wallet
```

## Usage

### Quick Start

1. **Generate a Key Pair**

First, create a cryptographic key pair for your wallet:

```bash
cargo run --bin key_gen -- mywallet
```

This creates:
- `mywallet.pub.pem` - Your public key (shareable)
- `mywallet.priv.cbor` - Your private key (keep secret!)

2. **Create a Wallet Configuration**

Create a `wallet_config.toml` file:

```toml
[[my_keys]]
public_key_path = "mywallet.pub.pem"
private_key_path = "mywallet.priv.cbor"

[[contacts]]
name = "Alice"
address_path = "alice.pub.pem"

default_node = "localhost:9000"

[fee_config]
fee_type = "Fixed"
value = 10.0
```

3. **Start a Node**

Ensure you have a blockchain node running:

```bash
cargo run --bin main -- --blockchain-file blockchain.cbor --port 9000
```

4. **Run the Wallet**

```bash
cargo run -p wallet
```

### Configuration Options

#### Wallet Configuration File (`wallet_config.toml`)

```toml
# Define your wallet keys (can have multiple)
[[my_keys]]
public_key_path = "wallet1.pub.pem"
private_key_path = "wallet1.priv.cbor"

[[my_keys]]
public_key_path = "wallet2.pub.pem"
private_key_path = "wallet2.priv.cbor"

# Define saved contacts/recipients
[[contacts]]
name = "Alice"
address_path = "alice.pub.pem"

[[contacts]]
name = "Bob"
address_path = "bob.pub.pem"

# Default blockchain node to connect to
default_node = "localhost:9000"

# Fee configuration
[fee_config]
fee_type = "Fixed"  # or "Percent"
value = 10.0        # 10 units for Fixed, or 10% for Percent
```

#### Fee Types

- **Fixed Fee**: A constant amount deducted from every transaction
  ```toml
  [fee_config]
  fee_type = "Fixed"
  value = 10.0  # 10 units per transaction
  ```

- **Percentage Fee**: A percentage of the transaction amount
  ```toml
  [fee_config]
  fee_type = "Percent"
  value = 2.5  # 2.5% of transaction amount
  ```

### Programmatic Usage

You can also use the wallet as a library in your own Rust applications:

```rust
use wallet::models::{Core, Config, FeeConfig, FeeType, Key};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load wallet from configuration file
    let core = Core::load(PathBuf::from("wallet_config.toml"))?;
    
    // Get current balance
    let balance = core.get_balance();
    println!("Current balance: {} satoshis", balance);
    
    // Fetch UTXOs from the network
    core.fetch_utxos().await?;
    
    // Create a transaction
    let recipient_pubkey = /* load recipient public key */;
    let transaction = core.create_transaction(&recipient_pubkey, 1000).await?;
    
    // Send transaction to network
    core.send_transaction(transaction).await?;
    
    Ok(())
}
```

### Example Workflow

#### 1. Check Balance

```rust
let core = Core::load(config_path)?;
core.fetch_utxos().await?;
let balance = core.get_balance();
println!("Balance: {}", balance);
```

#### 2. Create and Send Transaction

```rust
use btclib::crypto::PublicKey;

// Load recipient's public key
let recipient = PublicKey::load_from_file("alice.pub.pem")?;

// Create transaction for 1000 units
let tx = core.create_transaction(&recipient, 1000).await?;

// Transaction includes:
// - Input: Your UTXOs (automatically selected)
// - Output 1: Payment to recipient (1000 units)
// - Output 2: Change back to you (if applicable)
// - Fee: Automatically calculated and deducted

// Send to network
core.send_transaction(tx).await?;
```

#### 3. Multi-Key Wallet

The wallet supports multiple keys, allowing you to manage funds across different addresses:

```rust
let config = Config::new(
    vec![
        Key::new(
            PathBuf::from("key1.pub.pem"),
            PathBuf::from("key1.priv.cbor")
        ),
        Key::new(
            PathBuf::from("key2.pub.pem"),
            PathBuf::from("key2.priv.cbor")
        ),
    ],
    vec![],
    "localhost:9000".to_string(),
    FeeConfig::new(FeeType::Fixed, 10.0),
);
```

The wallet will automatically:
- Track UTXOs for all keys
- Use UTXOs from any key when creating transactions
- Calculate total balance across all keys

## Transaction Flow

1. **Fetch UTXOs**: Query the node for unspent outputs belonging to your keys
2. **Select Inputs**: Automatically select sufficient UTXOs to cover amount + fees
3. **Calculate Fee**: Apply configured fee (fixed or percentage)
4. **Create Outputs**: 
   - Payment output to recipient
   - Change output back to your wallet (if any)
5. **Sign Transaction**: Sign inputs with your private key(s)
6. **Broadcast**: Send transaction to node for inclusion in the blockchain

## UTXO Management

The wallet tracks UTXOs (Unspent Transaction Outputs) to determine your balance and create transactions:

- **Fetching**: Query nodes for UTXOs associated with your public keys
- **Marking**: Mark UTXOs as "spent" when used in pending transactions
- **Balance**: Sum of all unspent (unmarked) UTXO values
- **Selection**: Automatically select UTXOs to fund transactions

```rust
// UTXO structure
pub struct UtxoStore {
    utxos: Arc<SkipMap<PublicKey, Vec<(bool, TransactionOutput)>>>,
    //                                 ^^^^  ^^^^^^^^^^^^^^^^^^
    //                                 |     |
    //                                 |     +-- Transaction output
    //                                 +-------- Marked as spent?
    my_keys: Vec<LoadedKey>,
}
```

## Testing

The wallet module includes comprehensive test coverage:

```bash
# Run all wallet tests
cargo test -p wallet

# Run specific test
cargo test -p wallet test_core_create_transaction_success

# Run with output
cargo test -p wallet -- --nocapture
```

### Test Coverage

- **Config**: 7 tests (100% coverage)
- **Core**: 12 tests (92.42% coverage)
- **FeeConfig**: 6 tests (100% coverage)
- **FeeType**: 3 tests (100% coverage)
- **Key**: 3 tests (100% coverage)
- **LoadedKey**: 4 tests (100% coverage)
- **LoadedRecipient**: 5 tests (100% coverage)
- **Recipient**: 6 tests (100% coverage)
- **UtxoStore**: 5 tests (100% coverage)

Total: **51 tests**, all passing ✅

## Security Considerations

⚠️ **Important Security Notes:**

1. **Private Key Protection**
   - Never share your `.priv.cbor` files
   - Store private keys securely with appropriate file permissions
   - Consider encrypting private keys at rest

2. **Network Security**
   - Use TLS/SSL for node connections in production
   - Validate node responses to prevent MITM attacks
   - Consider running your own node for enhanced security

3. **Transaction Verification**
   - Always verify transaction details before signing
   - Check recipient addresses carefully
   - Confirm amounts and fees

4. **Backup Strategy**
   - Regularly backup wallet configuration and private keys
   - Store backups in secure, offline locations
   - Test backup restoration procedures

## Dependencies

Key dependencies:
- `btclib` - Core blockchain types and cryptography
- `tokio` - Async runtime for network operations
- `serde` / `toml` - Configuration serialization
- `crossbeam-skiplist` - Concurrent UTXO storage
- `anyhow` - Error handling
- `flume` - Transaction broadcasting channels

## Future Enhancements

Potential improvements:
- [ ] Interactive CLI interface with command menu
- [ ] Transaction history and explorer
- [ ] HD wallet support (hierarchical deterministic keys)
- [ ] Multi-signature wallet support
- [ ] Hardware wallet integration
- [ ] Encrypted private key storage
- [ ] Coin selection optimization algorithms
- [ ] Transaction fee estimation
- [ ] Address QR code generation
- [ ] Watch-only wallet mode

## Troubleshooting

### Cannot connect to node
- Verify node is running: `netstat -an | findstr :9000`
- Check `default_node` address in config
- Ensure firewall allows connections

### Transaction fails with "Insufficient funds"
- Run `core.fetch_utxos().await?` to update UTXO set
- Check balance with `core.get_balance()`
- Ensure you have enough to cover amount + fees

### Private key errors
- Verify file paths in wallet configuration
- Check file permissions are readable
- Ensure keys were generated with `key_gen`

### UTXO synchronization issues
- Node may not have indexed your addresses yet
- Wait for a few blocks to be mined
- Try connecting to a different node

## Contributing

Contributions welcome! Areas for improvement:
- Enhanced CLI user interface
- Additional fee estimation strategies
- Performance optimizations for large UTXO sets
- Additional test coverage for edge cases

## Related Documentation

- [Main README](../README.md) - Full project documentation
- [Node README](../node/README.md) - Blockchain node documentation
- [Miner README](../miner/README.md) - Mining component documentation
- [Library Documentation](../lib/README.md) - Core library reference

## License

[Inherit from main project license]
