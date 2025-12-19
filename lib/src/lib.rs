use serde::{Deserialize, Serialize};
use uint::construct_uint;
construct_uint! {
 // Construct an unsigned 256-bit integer
 // consisting of 4 x 64-bit words
 #[derive(Serialize, Deserialize)]
 pub struct U256(4);
}

// initial reward in bitcoin - multiply by 10^8 to get satoshis
pub const INITIAL_REWARD: u64 = 50;
// halving interval in blocks (Bitcoin uses 210,000)
pub const HALVING_INTERVAL: u64 = 210_000;
// ideal block time in seconds (Bitcoin: 10 minutes = 600 seconds)
pub const IDEAL_BLOCK_TIME: u64 = 600;
// minimum target
pub const MIN_TARGET: U256 = U256([
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0x0000_FFFF_FFFF_FFFF,
]);
// difficulty update interval in blocks (Bitcoin uses 2016)
pub const DIFFICULTY_UPDATE_INTERVAL: u64 = 2016;
// maximum mempool transaction age in seconds
pub const MAX_MEMPOOL_TX_AGE: u64 = 600; // 10 minutes
// maximum amount of transactions allowed in the block
pub const BLOCK_TRANSACTION_CAP: usize = 20;

pub mod crypto;
pub mod custom_sha_types;
pub mod error;
pub mod network;
pub mod types;
pub mod utils;
