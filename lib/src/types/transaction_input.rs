use serde::{Deserialize, Serialize};

use crate::{crypto::Signature, custom_sha_types::Hash};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionInput {
    pub prev_transaction_output_hash: Hash,
    pub signature: Signature,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::PrivateKey;

    #[test]
    fn test_transaction_input_creation() {
        let private_key = PrivateKey::new();
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
        let private_key = PrivateKey::new();
        let public_key = private_key.public_key();
        let prev_hash = Hash::zero();
        let signature = Signature::sign_output(&prev_hash, &private_key);

        assert!(signature.verify(&prev_hash, &public_key));
    }
}