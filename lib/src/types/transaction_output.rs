use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{crypto::PublicKey, custom_sha_types::Hash};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub unique_id: Uuid,
    pub pubkey: PublicKey,
}

impl TransactionOutput {
    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::PrivateKey;

    #[test]
    fn test_transaction_output_creation() {
        let private_key = PrivateKey::new();
        let output = TransactionOutput {
            value: 1000,
            unique_id: Uuid::new_v4(),
            pubkey: private_key.public_key(),
        };

        assert_eq!(output.value, 1000);
    }

    #[test]
    fn test_transaction_output_hash_deterministic() {
        let private_key = PrivateKey::new();
        let unique_id = Uuid::new_v4();
        let output = TransactionOutput {
            value: 1000,
            unique_id,
            pubkey: private_key.public_key(),
        };

        let hash1 = output.hash();
        let hash2 = output.hash();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_transaction_output_different_hashes() {
        let private_key = PrivateKey::new();
        let output1 = TransactionOutput {
            value: 1000,
            unique_id: Uuid::new_v4(),
            pubkey: private_key.public_key(),
        };
        let output2 = TransactionOutput {
            value: 2000,
            unique_id: Uuid::new_v4(),
            pubkey: private_key.public_key(),
        };

        assert_ne!(output1.hash(), output2.hash());
    }
}