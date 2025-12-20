use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::custom_sha_types::Hash;
use crate::{U256, utils::MerkleRoot};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    /// Timestamp of the block
    timestamp: DateTime<Utc>,
    /// Nonce used to mine the block
    nonce: u64,
    /// Hash of the previous block
    prev_block_hash: Hash,
    /// Merkle root of the block's transactions
    merkle_root: MerkleRoot,
    /// Proof-of-work difficulty target. The block hash must be less than or equal to this value for the block to be valid.
    target: U256,
}

impl BlockHeader {
    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: Hash,
        merkle_root: MerkleRoot,
        target: U256,
    ) -> Self {
        BlockHeader {
            timestamp,
            nonce,
            prev_block_hash,
            merkle_root,
            target,
        }
    }

    /// Attempts to find a valid nonce such that the block header's hash meets the target difficulty.
    ///
    /// Performs up to `steps` iterations, incrementing the nonce and updating the timestamp if the nonce overflows.
    /// Returns `true` if a valid nonce is found within the given steps, otherwise returns `false`.
    ///
    /// If `false` is returned, users may call this method again to continue mining, or adjust the target difficulty
    /// if mining is taking too long or is not feasible.
    pub fn mine(&mut self, steps: usize) -> bool {
        if self.hash().matches_target(self.target) {
            return true;
        }
        for _ in 0..steps {
            if let Some(new_nonce) = self.nonce.checked_add(1) {
                self.nonce = new_nonce;
            } else {
                self.nonce = 0;
                self.timestamp = Utc::now();
            }

            if self.hash().matches_target(self.target) {
                return true;
            }
        }
        false
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }

    pub fn target(&self) -> U256 {
        self.target
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn prev_block_hash(&self) -> &Hash {
        &self.prev_block_hash
    }

    pub fn merkle_root(&self) -> &MerkleRoot {
        &self.merkle_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MIN_TARGET,
        crypto::PrivateKey,
        types::{Transaction, TransactionOutput},
    };
    use uuid::Uuid;

    fn create_test_merkle_root() -> MerkleRoot {
        let private_key = PrivateKey::default();
        let tx = Transaction::new(
            vec![],
            vec![TransactionOutput::new(
                1000,
                Uuid::new_v4(),
                private_key.public_key(),
            )],
        );
        MerkleRoot::calculate(&[tx])
    }

    #[test]
    fn test_block_header_creation() {
        let timestamp = Utc::now();
        let merkle_root = create_test_merkle_root();
        let header = BlockHeader::new(timestamp, 0, Hash::zero(), merkle_root, MIN_TARGET);

        assert_eq!(header.nonce, 0);
        assert_eq!(header.prev_block_hash, Hash::zero());
        assert_eq!(header.target, MIN_TARGET);
    }

    #[test]
    fn test_block_header_hash_deterministic() {
        let timestamp = Utc::now();
        let merkle_root = create_test_merkle_root();
        let header = BlockHeader::new(timestamp, 0, Hash::zero(), merkle_root, MIN_TARGET);

        let hash1 = header.hash();
        let hash2 = header.hash();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_block_header_nonce_increment() {
        use crate::U256;
        let timestamp = Utc::now();
        let merkle_root = create_test_merkle_root();
        // Use a target that requires some mining (not too easy, not impossible)
        let target = U256([
            0xFFFF_FFFF_FFFF_FFFF,
            0xFFFF_FFFF_FFFF_FFFF,
            0xFFFF_FFFF_FFFF_FFFF,
            0x0000_0000_0000_00FF,
        ]);
        let mut header = BlockHeader::new(timestamp, 0, Hash::zero(), merkle_root, target);

        let initial_nonce = header.nonce;
        header.mine(100000);

        assert_ne!(header.nonce, initial_nonce);
    }

    #[test]
    fn test_block_header_mine_with_easy_target() {
        use crate::U256;
        let timestamp = Utc::now();
        let merkle_root = create_test_merkle_root();
        // Use a very easy target (close to max value) for testing
        let easy_target = U256::MAX / 100;
        let mut header = BlockHeader::new(timestamp, 0, Hash::zero(), merkle_root, easy_target);

        let result = header.mine(100000);
        assert!(result);
        assert!(header.hash().matches_target(header.target));
    }

    #[test]
    fn test_block_header_different_nonce_different_hash() {
        let timestamp = Utc::now();
        let merkle_root = create_test_merkle_root();
        let header1 = BlockHeader::new(timestamp, 0, Hash::zero(), merkle_root, MIN_TARGET);
        let header2 = BlockHeader::new(timestamp, 1, Hash::zero(), merkle_root, MIN_TARGET);

        assert_ne!(header1.hash(), header2.hash());
    }
}
