use std::{
    collections::HashMap,
    io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write},
};

use serde::{Deserialize, Serialize};

use crate::{
    custom_sha_types::Hash,
    error::{BtcError, Result},
    types::{BlockHeader, Transaction, TransactionOutput},
    utils::Saveable,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    header: BlockHeader,
    transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Block {
            header,
            transactions,
        }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }

    pub fn verify_transactions(
        &self,
        predicted_block_height: u64,
        utxos: &HashMap<Hash, (bool, TransactionOutput)>,
    ) -> Result<()> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();

        // Rejecting empty blocks
        if self.transactions.is_empty() {
            return Err(BtcError::InvalidTransaction);
        }
        // Verify coinbase transaction
        self.verify_coinbase_transaction(predicted_block_height, utxos)?;

        for transaction in &self.transactions {
            let mut input_value = 0;
            let mut output_value = 0;
            for input in transaction.inputs() {
                let prev_output = utxos
                    .get(input.prev_transaction_output_hash())
                    .map(|(_, output)| output);

                let prev_output = prev_output.ok_or(BtcError::InvalidTransaction)?;

                if inputs.contains_key(input.prev_transaction_output_hash()) {
                    return Err(BtcError::DoubleSpending);
                }

                if !input
                    .signature()
                    .verify(input.prev_transaction_output_hash(), prev_output.pubkey())
                {
                    return Err(BtcError::InvalidSignature);
                }

                input_value += prev_output.value();
                inputs.insert(*input.prev_transaction_output_hash(), prev_output.clone());
            }

            for output in transaction.outputs() {
                output_value += output.value();
            }

            if input_value < output_value {
                return Err(BtcError::InvalidTransaction);
            }
        }

        Ok(())
    }

    pub fn verify_coinbase_transaction(
        &self,
        predicted_block_height: u64,
        utxos: &HashMap<Hash, (bool, TransactionOutput)>,
    ) -> Result<()> {
        let coinbase_transaction = &self.transactions[0];

        if !coinbase_transaction.inputs().is_empty() {
            return Err(BtcError::InvalidTransaction);
        }

        if coinbase_transaction.outputs().is_empty() {
            return Err(BtcError::InvalidTransaction);
        }

        let miner_fees = self.calculated_miner_fees(utxos)?;
        let block_reward = crate::INITIAL_REWARD * 10u64.pow(8)
            / 2u64.pow((predicted_block_height / crate::HALVING_INTERVAL) as u32);

        let total_coinbase_outputs: u64 = coinbase_transaction
            .outputs()
            .iter()
            .map(|output| output.value())
            .sum();

        if total_coinbase_outputs != block_reward + miner_fees {
            return Err(BtcError::InvalidTransaction);
        }

        Ok(())
    }

    pub fn calculated_miner_fees(
        &self,
        utxos: &HashMap<Hash, (bool, TransactionOutput)>,
    ) -> Result<u64> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        let mut outputs: HashMap<Hash, TransactionOutput> = HashMap::new();

        for transaction in &self.transactions[1..] {
            for input in transaction.inputs() {
                let previous_transaction_output_hash = input.prev_transaction_output_hash();
                if inputs.contains_key(previous_transaction_output_hash) {
                    return Err(BtcError::DoubleSpending);
                }

                let prev_output = utxos
                    .get(previous_transaction_output_hash)
                    .map(|(_, output)| output);

                let prev_output = prev_output.ok_or(BtcError::InvalidTransaction)?;
                
                inputs.insert(*previous_transaction_output_hash, prev_output.clone());
            }

            for output in transaction.outputs() {
                if outputs.insert(output.hash(), output.clone()).is_some() {
                    return Err(BtcError::DoubleSpending);
                }
            }
        }

        let input_value: u64 = inputs.values().map(|output| output.value()).sum();
        let output_value: u64 = outputs.values().map(|output| output.value()).sum();

        match input_value.checked_sub(output_value) {
            Some(fee) => Ok(fee),
            None => Err(BtcError::InvalidTransaction),
        }
    }

    pub fn header(&self) -> &BlockHeader {
        &self.header
    }

    pub fn mine(&mut self, steps: usize) -> bool {
        self.header.mine(steps)
    }

    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }
}

impl Saveable for Block {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to deserialize Block"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to serialize Block"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MIN_TARGET, crypto::PrivateKey, utils::MerkleRoot};
    use chrono::Utc;
    use uuid::Uuid;

    fn create_coinbase_transaction(value: u64) -> Transaction {
        let private_key = PrivateKey::default();
        Transaction::new(
            vec![],
            vec![TransactionOutput::new(
                value,
                Uuid::new_v4(),
                private_key.public_key(),
            )],
        )
    }

    #[test]
    fn test_block_creation() {
        let transactions = vec![create_coinbase_transaction(5000000000)];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let header = BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, MIN_TARGET);
        let block = Block::new(header, transactions);

        assert_eq!(block.transactions.len(), 1);
    }

    #[test]
    fn test_block_hash_deterministic() {
        let transactions = vec![create_coinbase_transaction(5000000000)];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let header = BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, MIN_TARGET);
        let block = Block::new(header, transactions);

        let hash1 = block.hash();
        let hash2 = block.hash();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_block_verify_empty_transactions() {
        // Create a dummy transaction for merkle root calculation
        let dummy_tx = create_coinbase_transaction(5000000000);
        let merkle_root = MerkleRoot::calculate(&[dummy_tx]);

        let header = BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, MIN_TARGET);
        // Create block with empty transactions (invalid)
        let block = Block::new(header, vec![]);
        let utxos = HashMap::new();

        let result = block.verify_transactions(0, &utxos);
        assert!(result.is_err());
    }

    #[test]
    fn test_block_verify_coinbase_no_inputs() {
        let transactions = vec![create_coinbase_transaction(5000000000)];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let header = BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, MIN_TARGET);
        let block = Block::new(header, transactions);
        let utxos = HashMap::new();

        let result = block.verify_coinbase_transaction(0, &utxos);
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_serialization() {
        let transactions = vec![create_coinbase_transaction(5000000000)];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let header = BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, MIN_TARGET);
        let block = Block::new(header, transactions);

        let mut buffer = Vec::new();
        block.save(&mut buffer).expect("Failed to serialize block");

        let loaded_block = Block::load(buffer.as_slice()).expect("Failed to deserialize block");

        assert_eq!(block.transactions.len(), loaded_block.transactions.len());
    }

    #[test]
    fn test_calculated_miner_fees_no_transactions() {
        let transactions = vec![create_coinbase_transaction(5000000000)];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let header = BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, MIN_TARGET);
        let block = Block::new(header, transactions);
        let utxos = HashMap::new();

        let fees = block.calculated_miner_fees(&utxos);
        assert!(fees.is_ok());
        assert_eq!(fees.unwrap(), 0);
    }
}
