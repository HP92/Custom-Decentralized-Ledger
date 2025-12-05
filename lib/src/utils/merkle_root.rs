use serde::{Deserialize, Serialize};

use crate::{custom_sha_types::Hash, types::Transaction};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MerkleRoot(Hash);

impl MerkleRoot {
    pub fn calculate(transactions: &[Transaction]) -> Self {
        let mut layer: Vec<Hash> = vec![];
        for transaction in transactions {
            layer.push(Hash::hash(transaction));
        }

        while layer.len() > 1 {
            let mut next_layer: Vec<Hash> = vec![];
            for pair in layer.chunks(2) {
                let left = pair[0];
                let right = pair.get(1).unwrap_or(&pair[0]);
                next_layer.push(Hash::hash(&[left, *right]));
            }
            layer = next_layer;
        }

        MerkleRoot(layer[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{crypto::PrivateKey, types::TransactionOutput};
    use uuid::Uuid;

    fn create_test_transaction(value: u64) -> Transaction {
        let private_key = PrivateKey::new();
        Transaction::new(
            vec![],
            vec![TransactionOutput {
                value,
                unique_id: Uuid::new_v4(),
                pubkey: private_key.public_key(),
            }],
        )
    }

    #[test]
    fn test_merkle_root_single_transaction() {
        let tx = create_test_transaction(1000);
        let merkle_root = MerkleRoot::calculate(&[tx]);
        
        // The merkle root of a single transaction should be deterministic
        let tx2 = create_test_transaction(1000);
        let merkle_root2 = MerkleRoot::calculate(&[tx2]);
        
        // Different transactions should have different roots
        assert_ne!(merkle_root, merkle_root2);
    }

    #[test]
    fn test_merkle_root_same_transaction() {
        let tx = create_test_transaction(1000);
        let merkle_root1 = MerkleRoot::calculate(&[tx.clone()]);
        let merkle_root2 = MerkleRoot::calculate(&[tx]);
        
        // Same transaction should produce same merkle root
        assert_eq!(merkle_root1, merkle_root2);
    }

    #[test]
    fn test_merkle_root_two_transactions() {
        let tx1 = create_test_transaction(1000);
        let tx2 = create_test_transaction(2000);
        
        let merkle_root = MerkleRoot::calculate(&[tx1.clone(), tx2.clone()]);
        
        // Should be deterministic
        let merkle_root2 = MerkleRoot::calculate(&[tx1, tx2]);
        assert_eq!(merkle_root, merkle_root2);
    }

    #[test]
    fn test_merkle_root_odd_number_transactions() {
        let tx1 = create_test_transaction(1000);
        let tx2 = create_test_transaction(2000);
        let tx3 = create_test_transaction(3000);
        
        // Odd number of transactions should duplicate the last one
        let merkle_root = MerkleRoot::calculate(&[tx1.clone(), tx2.clone(), tx3.clone()]);
        
        // Should be deterministic
        let merkle_root2 = MerkleRoot::calculate(&[tx1, tx2, tx3]);
        assert_eq!(merkle_root, merkle_root2);
    }

    #[test]
    fn test_merkle_root_order_matters() {
        let tx1 = create_test_transaction(1000);
        let tx2 = create_test_transaction(2000);
        
        let merkle_root1 = MerkleRoot::calculate(&[tx1.clone(), tx2.clone()]);
        let merkle_root2 = MerkleRoot::calculate(&[tx2, tx1]);
        
        // Order should matter
        assert_ne!(merkle_root1, merkle_root2);
    }

    #[test]
    fn test_merkle_root_many_transactions() {
        let transactions: Vec<Transaction> = (0..8)
            .map(|i| create_test_transaction(i * 1000))
            .collect();
        
        let merkle_root = MerkleRoot::calculate(&transactions);
        
        // Should be deterministic
        let merkle_root2 = MerkleRoot::calculate(&transactions);
        assert_eq!(merkle_root, merkle_root2);
    }

    #[test]
    fn test_merkle_root_clone_and_eq() {
        let tx = create_test_transaction(1000);
        let merkle_root = MerkleRoot::calculate(&[tx]);
        
        let cloned = merkle_root.clone();
        assert_eq!(merkle_root, cloned);
    }

    #[test]
    fn test_merkle_root_debug_format() {
        let tx = create_test_transaction(1000);
        let merkle_root = MerkleRoot::calculate(&[tx]);
        
        let debug_str = format!("{:?}", merkle_root);
        assert!(debug_str.contains("MerkleRoot"));
    }
}
