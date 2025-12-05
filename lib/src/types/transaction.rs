use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

use serde::{Deserialize, Serialize};

use crate::{
    custom_sha_types::Hash,
    types::{TransactionInput, TransactionOutput},
    utils::Saveable,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
}

impl Transaction {
    pub fn new(inputs: Vec<TransactionInput>, outputs: Vec<TransactionOutput>) -> Self {
        Transaction { inputs, outputs }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl Saveable for Transaction {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader).map_err(|_| {
            IoError::new(
                IoErrorKind::InvalidData,
                "Failed to deserialize Transaction",
            )
        })
    }
    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to serialize Transaction"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::PrivateKey;
    use uuid::Uuid;

    fn create_test_output(value: u64) -> TransactionOutput {
        let private_key = PrivateKey::new();
        TransactionOutput {
            value,
            unique_id: Uuid::new_v4(),
            pubkey: private_key.public_key(),
        }
    }

    #[test]
    fn test_transaction_new() {
        let outputs = vec![create_test_output(1000)];
        let tx = Transaction::new(vec![], outputs);

        assert_eq!(tx.inputs.len(), 0);
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.outputs[0].value, 1000);
    }

    #[test]
    fn test_transaction_hash_deterministic() {
        let outputs = vec![create_test_output(1000)];
        let tx = Transaction::new(vec![], outputs);

        let hash1 = tx.hash();
        let hash2 = tx.hash();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_transaction_different_hashes() {
        let tx1 = Transaction::new(vec![], vec![create_test_output(1000)]);
        let tx2 = Transaction::new(vec![], vec![create_test_output(2000)]);

        assert_ne!(tx1.hash(), tx2.hash());
    }

    #[test]
    fn test_transaction_serialization() {
        let outputs = vec![create_test_output(1000)];
        let tx = Transaction::new(vec![], outputs);

        let mut buffer = Vec::new();
        tx.save(&mut buffer).expect("Failed to serialize transaction");

        let loaded_tx = Transaction::load(buffer.as_slice())
            .expect("Failed to deserialize transaction");

        assert_eq!(tx.outputs.len(), loaded_tx.outputs.len());
        assert_eq!(tx.outputs[0].value, loaded_tx.outputs[0].value);
    }

    #[test]
    fn test_transaction_empty_inputs_outputs() {
        let tx = Transaction::new(vec![], vec![]);
        assert_eq!(tx.inputs.len(), 0);
        assert_eq!(tx.outputs.len(), 0);
    }
}