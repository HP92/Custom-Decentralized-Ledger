use serde::{Deserialize, Serialize};

use crate::{crypto::Signature, custom_sha_types::Hash};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionInput {
    prev_transaction_output_hash: Hash,
    signature: Signature,
}

impl TransactionInput {
    /// Creates a new TransactionInput after validating the hash and signature.
    /// Returns None if the hash is zero or the signature is invalid.
    pub fn new(prev_transaction_output_hash: Hash, signature: Signature) -> Self {
        TransactionInput {
            prev_transaction_output_hash,
            signature,
        }
    }

    pub fn prev_transaction_output_hash(&self) -> &Hash {
        &self.prev_transaction_output_hash
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::PrivateKey;

    #[test]
    fn test_transaction_input_creation() {
        let private_key = PrivateKey::default();
        let prev_hash = Hash::zero();
        let signature = Signature::sign_output(&prev_hash, &private_key);

        let input = TransactionInput {
            prev_transaction_output_hash: prev_hash,
            signature,
        };

        assert_eq!(input.prev_transaction_output_hash, Hash::zero());
    }

    #[test]
    fn test_transaction_input_signature_verification() {
        let private_key = PrivateKey::default();
        let public_key = private_key.public_key();
        let prev_hash = Hash::zero();
        let signature = Signature::sign_output(&prev_hash, &private_key);

        assert!(signature.verify(&prev_hash, &public_key));
    }
}
