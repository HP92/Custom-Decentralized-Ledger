use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::custom_sha_types::Hash;
use crate::{U256, utils::MerkleRoot};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    /// Timestamp of the block
    pub timestamp: DateTime<Utc>,
    /// Nonce used to mine the block
    pub nonce: u64,
    /// Hash of the previous block
    pub prev_block_hash: Hash,
    /// Merkle root of the block's transactions
    pub merkle_root: MerkleRoot,
    /// target
    pub target: U256,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MIN_TARGET, crypto::PrivateKey, types::{Transaction, TransactionOutput}};
    use uuid::Uuid;

    fn create_test_merkle_root() -> MerkleRoot {
        let private_key = PrivateKey::new();
        let tx = Transaction::new(
            vec![],
            vec![TransactionOutput {
                value: 1000,
                unique_id: Uuid::new_v4(),
                pubkey: private_key.public_key(),
            }],
        );
        MerkleRoot::calculate(&[tx])
    }

    #[test]
    fn test_block_header_creation() {
        let timestamp = Utc::now();
        let merkle_root = create_test_merkle_root();
        let header = BlockHeader::new(
            timestamp,
            0,
            Hash::zero(),
            merkle_root,
            MIN_TARGET,
        );

        assert_eq!(header.nonce, 0);
        assert_eq!(header.prev_block_hash, Hash::zero());
        assert_eq!(header.target, MIN_TARGET);
    }

    #[test]
    fn test_block_header_hash_deterministic() {
        let timestamp = Utc::now();
        let merkle_root = create_test_merkle_root();
        let header = BlockHeader::new(
            timestamp,
            0,
            Hash::zero(),
            merkle_root,
            MIN_TARGET,
        );

        let hash1 = header.hash();
        let hash2 = header.hash();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_block_header_nonce_increment() {
        let timestamp = Utc::now();
        let merkle_root = create_test_merkle_root();
        let mut header = BlockHeader::new(
            timestamp,
            0,
            Hash::zero(),
            merkle_root,
            MIN_TARGET,
        );

        let initial_nonce = header.nonce;
        header.mine(1);
        
        assert_ne!(header.nonce, initial_nonce);
    }

    #[test]
    fn test_block_header_mine_with_easy_target() {
        use crate::U256;
        let timestamp = Utc::now();
        let merkle_root = create_test_merkle_root();
        // Use a very easy target (close to max value) for testing
        let easy_target = U256::MAX / 100;
        let mut header = BlockHeader::new(
            timestamp,
            0,
            Hash::zero(),
            merkle_root,
            easy_target,
        );

        let result = header.mine(100000);
        assert!(result);
        assert!(header.hash().matches_target(header.target));
    }

    #[test]
    fn test_block_header_different_nonce_different_hash() {
        let timestamp = Utc::now();
        let merkle_root = create_test_merkle_root();
        let header1 = BlockHeader::new(
            timestamp,
            0,
            Hash::zero(),
            merkle_root,
            MIN_TARGET,
        );
        let header2 = BlockHeader::new(
            timestamp,
            1,
            Hash::zero(),
            merkle_root,
            MIN_TARGET,
        );

        assert_ne!(header1.hash(), header2.hash());
    }
}