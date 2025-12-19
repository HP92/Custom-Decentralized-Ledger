use std::{
    collections::{HashMap, HashSet},
    io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write},
};

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};

use crate::{
    INITIAL_REWARD, U256,
    custom_sha_types::Hash,
    error::{BtcError, Result},
    types::{Block, Transaction, TransactionOutput},
    utils::{MerkleRoot, Saveable},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {
    // UTXO: Unspent Transaction Outputs mapped by their hash
    utxos: HashMap<Hash, (bool, TransactionOutput)>,
    target: U256,
    blocks: Vec<Block>,
    #[serde(default, skip_serializing)]
    mempool: Vec<(DateTime<Utc>, Transaction)>,
}

impl Blockchain {
    pub fn utxos(&self) -> HashMap<Hash, TransactionOutput> {
        self.utxos
            .iter()
            .map(|(hash, (_spent, output))| (*hash, output.clone()))
            .collect()
    }

    pub fn target(&self) -> U256 {
        self.target
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn block_height(&self) -> u64 {
        self.blocks.len() as u64
    }

    pub fn mempool(&self) -> &[(DateTime<Utc>, Transaction)] {
        &self.mempool
    }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        if self.blocks.is_empty() {
            // if this is the first block, check if the block's previous hash is all zeros
            if *block.header().prev_block_hash() != Hash::zero() {
                error!(
                    "Previous hash: {:x?} is not equal to zero",
                    block.header().prev_block_hash()
                );
                return Err(crate::error::BtcError::InvalidBlock);
            }
        } else {
            // if this is not the first block, check if the block's
            // previous hash is the hash of the last block
            let last_block = self.blocks.last().unwrap();
            if *block.header().prev_block_hash() != last_block.header().hash() {
                error!(
                    "Previous hash: {:x?} is not equal to last block hash: {:x?}",
                    block.header().prev_block_hash(),
                    last_block.header().hash()
                );
                return Err(crate::error::BtcError::InvalidBlock);
            }

            // check if the block's hash is less than the target
            if !block
                .header()
                .hash()
                .matches_target(block.header().target())
            {
                error!(
                    "Does not match target: {:x?} >= {:x?}",
                    block.header().hash(),
                    block.header().target()
                );
                return Err(crate::error::BtcError::InvalidBlock);
            }

            let calculated_merkle_root = MerkleRoot::calculate(block.transactions());
            if *block.header().merkle_root() != calculated_merkle_root {
                error!(
                    "Invalid Merkle root: {:x?} != {:x?}",
                    block.header().merkle_root(),
                    calculated_merkle_root
                );
                return Err(crate::error::BtcError::InvalidMerkleRoot);
            }

            if block.header().timestamp() <= last_block.header().timestamp() {
                error!(
                    "Invalid timestamp: {} <= {}",
                    block.header().timestamp(),
                    last_block.header().timestamp()
                );
                return Err(crate::error::BtcError::InvalidBlockHeader);
            }

            block.verify_transactions(self.block_height(), &self.utxos)?;
        }

        let block_transactions: HashSet<_> =
            block.transactions().iter().map(|tx| tx.hash()).collect();
        self.mempool
            .retain(|tx| !block_transactions.contains(&tx.1.hash()));

        self.blocks.push(block);

        self.try_adjust_target();
        Ok(())
    }

    pub fn try_adjust_target(&mut self) {
        if self.blocks.is_empty() {
            return;
        }
        if self.blocks.len() % crate::DIFFICULTY_UPDATE_INTERVAL as usize != 0 {
            return;
        }
        // measure the time it took to mine the last
        // crate::DIFFICULTY_UPDATE_INTERVAL blocks
        // with chrono
        let start_time = self.blocks
            [self.blocks.len() - crate::DIFFICULTY_UPDATE_INTERVAL as usize]
            .header()
            .timestamp();
        let end_time = self.blocks.last().unwrap().header().timestamp();
        let time_diff = end_time - start_time;
        // convert time_diff to seconds
        let time_diff_seconds = time_diff.num_seconds();
        // calculate the ideal number of seconds
        let target_seconds = crate::IDEAL_BLOCK_TIME * crate::DIFFICULTY_UPDATE_INTERVAL;
        // multiply the current target by actual time divided by ideal time

        let new_target = BigDecimal::parse_bytes(self.target.to_string().as_bytes(), 10)
            .expect("BUG: impossible")
            * (BigDecimal::from(time_diff_seconds) / BigDecimal::from(target_seconds));
        // cut off decimal point and everything after
        // it from string representation of new_target
        let new_target_str = new_target
            .to_string()
            .split('.')
            .next()
            .expect("BUG: Expected a decimal point")
            .to_owned();
        let new_target: U256 = U256::from_str_radix(&new_target_str, 10).expect("BUG: impossible");

        // let new_target = self.target * (time_diff_seconds as f64 / target_seconds as f64) as usize;
        // clamp new_target to be within the range of
        // 4 * self.target and self.target / 4
        let new_target = if new_target < self.target / 4 {
            self.target / 4
        } else if new_target > self.target * 4 {
            self.target * 4
        } else {
            new_target
        };
        // if the new target is more than the minimum target,
        // set it to the minimum target
        self.target = new_target.min(crate::MIN_TARGET);
    }

    pub fn rebuild_utxos(&mut self) {
        for block in &self.blocks {
            for tx in block.transactions() {
                // Remove spent UTXOs
                for input in tx.inputs() {
                    self.utxos.remove(input.prev_transaction_output_hash());
                }
                // Add new UTXOs
                self.utxos
                    .extend(tx.outputs().iter().map(|o| (tx.hash(), (false, o.clone()))));
            }
        }
    }

    pub fn add_transaction_to_mempool(&mut self, transaction: Transaction) -> Result<()> {
        // validate transaction before insertion
        // all inputs must match known UTXOs, and must be unique
        let mut known_inputs = HashSet::new();
        for input in transaction.inputs() {
            let prev_transaction_output = input.prev_transaction_output_hash();

            if !self.utxos.contains_key(prev_transaction_output) {
                error!(
                    "UTXO not found for input {:x?}",
                    input.prev_transaction_output_hash()
                );
                return Err(BtcError::InvalidTransaction);
            }
            if !known_inputs.insert(prev_transaction_output) {
                error!("duplicate input found");
                return Err(BtcError::InvalidTransaction);
            }
        }
        // check if any of the utxos have the bool mark set to true
        // and if so, find the transaction that references them
        // in mempool, remove it, and set all the utxos it references
        // to false
        for input in transaction.inputs() {
            if let Some((true, _)) = self.utxos.get_mut(input.prev_transaction_output_hash()) {
                // find the transaction that references the UTXO
                // we are trying to reference
                let referencing_transaction =
                    self.mempool.iter().enumerate().find(|(_, transaction)| {
                        transaction
                            .1
                            .outputs()
                            .iter()
                            .any(|output| output.hash() == *input.prev_transaction_output_hash())
                    });
                // If we have found one, unmark all of its UTXOs
                if let Some((idx, referencing_transaction)) = referencing_transaction {
                    for input in referencing_transaction.1.inputs() {
                        // set all utxos from this transaction to false
                        self.utxos
                            .entry(*input.prev_transaction_output_hash())
                            .and_modify(|(marked, _)| {
                                *marked = false;
                            });
                    }
                    // remove the transaction from the mempool
                    self.mempool.remove(idx);
                } else {
                    // if, somehow, there is no matching transaction,
                    // set this utxo to false
                    self.utxos
                        .entry(*input.prev_transaction_output_hash())
                        .and_modify(|(marked, _)| {
                            *marked = false;
                        });
                }
            }
        }
        // all inputs must be lower than all outputs
        let all_inputs = transaction
            .inputs()
            .iter()
            .map(|input| {
                self.utxos
                    .get(input.prev_transaction_output_hash())
                    .expect("BUG: impossible")
                    .1 // < - - - Look here
                    .value()
            })
            .sum::<u64>();
        let all_outputs = transaction
            .outputs()
            .iter()
            .map(|output| output.value())
            .sum();
        if all_inputs < all_outputs {
            return Err(BtcError::InvalidTransaction);
        }
        self.mempool.push((Utc::now(), transaction));
        // sort by miner fee descending
        self.mempool.sort_by_key(|transaction| {
            let all_inputs = transaction
                .1
                .inputs()
                .iter()
                .map(|input| {
                    self.utxos
                        .get(input.prev_transaction_output_hash())
                        .expect("BUG: impossible")
                        .1
                        .value()
                })
                .sum::<u64>();

            let all_outputs = transaction
                .1
                .outputs()
                .iter()
                .map(|output| output.value())
                .sum::<u64>();

            let miner_fee = all_inputs - all_outputs;
            std::cmp::Reverse(miner_fee)
        });

        Ok(())
    }

    pub fn cleanup_mempool(&mut self) {
        let now = Utc::now();
        let mut utxo_hashes_to_unmark: Vec<Hash> = vec![];

        self.mempool.retain(|(timestamp, transaction)| {
            let age = (now - *timestamp).num_seconds() as u64;
            if age > crate::MAX_MEMPOOL_TX_AGE {
                // collect all utxo hashes to unmark
                utxo_hashes_to_unmark.extend(
                    transaction
                        .inputs()
                        .iter()
                        .map(|input| *input.prev_transaction_output_hash()),
                );
                false
            } else {
                true
            }
        });
        // unmark all of the UTXOs
        for hash in utxo_hashes_to_unmark {
            self.utxos.entry(hash).and_modify(|(marked, _)| {
                *marked = false;
            });
        }
    }

    pub fn calculate_block_reward(&self) -> u64 {
        let block_height = self.block_height();
        let halvings = block_height / crate::HALVING_INTERVAL;
        (INITIAL_REWARD * 10u64.pow(8)) >> halvings
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self {
            utxos: HashMap::new(),
            target: crate::MIN_TARGET,
            blocks: vec![],
            mempool: vec![],
        }
    }
}

impl Saveable for Blockchain {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to deserialize Blockchain"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to serialize Blockchain"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MIN_TARGET,
        crypto::{PrivateKey, Signature},
        types::TransactionInput,
    };
    use chrono::{Duration, Utc};
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

    fn create_genesis_block() -> Block {
        let transactions = vec![create_coinbase_transaction(5000000000)];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let header =
            crate::types::BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, MIN_TARGET);
        Block::new(header, transactions)
    }

    fn create_mined_genesis_block() -> Block {
        let transactions = vec![create_coinbase_transaction(5000000000)];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let mut header =
            crate::types::BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, MIN_TARGET);
        header.mine(1000000);
        Block::new(header, transactions)
    }

    #[test]
    fn test_blockchain_new() {
        let blockchain = Blockchain::default();
        assert_eq!(blockchain.block_height(), 0);
        assert_eq!(blockchain.blocks().len(), 0);
        assert_eq!(blockchain.mempool().len(), 0);
    }

    #[test]
    fn test_blockchain_add_genesis_block() {
        let mut blockchain = Blockchain::default();
        let block = create_genesis_block();

        let result = blockchain.add_block(block);
        assert!(result.is_ok());
        assert_eq!(blockchain.block_height(), 1);
    }

    #[test]
    fn test_blockchain_reject_invalid_prev_hash() {
        let mut blockchain = Blockchain::default();
        let transactions = vec![create_coinbase_transaction(5000000000)];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let header = crate::types::BlockHeader::new(
            Utc::now(),
            0,
            Hash::hash(&"invalid"),
            merkle_root,
            MIN_TARGET,
        );
        let block = Block::new(header, transactions);

        let result = blockchain.add_block(block);
        assert!(result.is_err());
    }

    #[test]
    fn test_blockchain_reject_invalid_target() {
        let mut blockchain = Blockchain::default();
        blockchain.add_block(create_mined_genesis_block()).unwrap();

        let transactions = vec![create_coinbase_transaction(5000000000)];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let last_hash = blockchain.blocks().last().unwrap().header().hash();

        // Create block with invalid nonce (won't match target)
        let header =
            crate::types::BlockHeader::new(Utc::now(), 0, last_hash, merkle_root, MIN_TARGET);
        let block = Block::new(header, transactions);

        let result = blockchain.add_block(block);
        assert!(result.is_err());
    }

    #[test]
    fn test_blockchain_reject_invalid_merkle_root() {
        let mut blockchain = Blockchain::default();
        blockchain.add_block(create_mined_genesis_block()).unwrap();

        let transactions = vec![create_coinbase_transaction(5000000000)];
        let wrong_merkle = MerkleRoot::calculate(&[create_coinbase_transaction(1000)]);
        let last_hash = blockchain.blocks().last().unwrap().header().hash();

        let mut header =
            crate::types::BlockHeader::new(Utc::now(), 0, last_hash, wrong_merkle, MIN_TARGET);
        header.mine(1000000);
        let block = Block::new(header, transactions);

        let result = blockchain.add_block(block);
        assert!(result.is_err());
    }

    #[test]
    fn test_blockchain_reject_invalid_timestamp() {
        let mut blockchain = Blockchain::default();
        let first_block = create_mined_genesis_block();
        let first_timestamp = first_block.header().timestamp();
        blockchain.add_block(first_block).unwrap();

        let transactions = vec![create_coinbase_transaction(5000000000)];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let last_hash = blockchain.blocks().last().unwrap().header().hash();

        // Create block with earlier timestamp
        let mut header = crate::types::BlockHeader::new(
            first_timestamp - Duration::seconds(1),
            0,
            last_hash,
            merkle_root,
            MIN_TARGET,
        );
        header.mine(1000000);
        let block = Block::new(header, transactions);

        let result = blockchain.add_block(block);
        assert!(result.is_err());
    }

    #[test]
    fn test_blockchain_utxos() {
        let blockchain = Blockchain::default();
        let utxos = blockchain.utxos();
        assert_eq!(utxos.len(), 0);
    }

    #[test]
    fn test_blockchain_target() {
        let blockchain = Blockchain::default();
        assert_eq!(blockchain.target(), MIN_TARGET);
    }

    #[test]
    fn test_blockchain_serialization() {
        let blockchain = Blockchain::default();

        let mut buffer = Vec::new();
        blockchain
            .save(&mut buffer)
            .expect("Failed to serialize blockchain");

        let loaded = Blockchain::load(buffer.as_slice()).expect("Failed to deserialize blockchain");

        assert_eq!(loaded.block_height(), blockchain.block_height());
    }

    #[test]
    fn test_blockchain_rebuild_utxos() {
        let mut blockchain = Blockchain::default();
        blockchain.rebuild_utxos();
        assert_eq!(blockchain.utxos().len(), 0);
    }

    #[test]
    fn test_blockchain_rebuild_utxos_with_blocks() {
        let mut blockchain = Blockchain::default();
        blockchain.add_block(create_genesis_block()).unwrap();

        // Clear utxos
        blockchain.utxos.clear();
        assert_eq!(blockchain.utxos().len(), 0);

        // Rebuild
        blockchain.rebuild_utxos();
        assert!(blockchain.utxos().len() > 0);
    }

    #[test]
    fn test_blockchain_cleanup_mempool() {
        let mut blockchain = Blockchain::default();
        blockchain.cleanup_mempool();
        assert_eq!(blockchain.mempool().len(), 0);
    }

    #[test]
    fn test_blockchain_add_transaction_to_mempool_no_utxos() {
        let mut blockchain = Blockchain::default();

        let private_key = PrivateKey::default();
        let fake_hash = Hash::zero();
        let signature = Signature::sign_output(&fake_hash, &private_key);

        let tx = Transaction::new(
            vec![TransactionInput::new(fake_hash, signature)],
            vec![TransactionOutput::new(
                1000,
                Uuid::new_v4(),
                private_key.public_key(),
            )],
        );

        let result = blockchain.add_transaction_to_mempool(tx);
        assert!(result.is_err());
    }

    #[test]
    fn test_blockchain_add_transaction_duplicate_inputs() {
        let mut blockchain = Blockchain::default();
        blockchain.add_block(create_genesis_block()).unwrap();
        blockchain.rebuild_utxos();

        let private_key = PrivateKey::default();
        let utxo_hash = blockchain.utxos().keys().next().unwrap().clone();
        let signature = Signature::sign_output(&utxo_hash, &private_key);

        let tx = Transaction::new(
            vec![
                TransactionInput::new(utxo_hash, signature.clone()),
                TransactionInput::new(utxo_hash, signature),
            ],
            vec![TransactionOutput::new(
                1000,
                Uuid::new_v4(),
                private_key.public_key(),
            )],
        );

        let result = blockchain.add_transaction_to_mempool(tx);
        assert!(result.is_err());
    }

    #[test]
    fn test_blockchain_add_transaction_invalid_value() {
        let mut blockchain = Blockchain::default();
        blockchain.add_block(create_genesis_block()).unwrap();
        blockchain.rebuild_utxos();

        let private_key = PrivateKey::default();
        let utxos = blockchain.utxos();
        let (utxo_hash, utxo_output) = utxos.iter().next().unwrap();
        let signature = Signature::sign_output(&utxo_hash, &private_key);

        // Try to spend more than input value
        let tx = Transaction::new(
            vec![TransactionInput::new(utxo_hash.clone(), signature)],
            vec![TransactionOutput::new(
                utxo_output.value() + 1000,
                Uuid::new_v4(),
                private_key.public_key(),
            )],
        );

        let result = blockchain.add_transaction_to_mempool(tx);
        assert!(result.is_err());
    }

    #[test]
    fn test_blockchain_add_valid_transaction_to_mempool() {
        let mut blockchain = Blockchain::default();
        blockchain.add_block(create_genesis_block()).unwrap();
        blockchain.rebuild_utxos();

        let private_key = PrivateKey::default();
        let utxos = blockchain.utxos();
        let (utxo_hash, utxo_output) = utxos.iter().next().unwrap();
        let signature = Signature::sign_output(&utxo_hash, &private_key);

        let tx = Transaction::new(
            vec![TransactionInput::new(utxo_hash.clone(), signature)],
            vec![TransactionOutput::new(
                utxo_output.value() - 100,
                Uuid::new_v4(),
                private_key.public_key(),
            )],
        );

        let result = blockchain.add_transaction_to_mempool(tx);
        assert!(result.is_ok());
        assert_eq!(blockchain.mempool().len(), 1);
    }

    #[test]
    fn test_blockchain_try_adjust_target_empty() {
        let mut blockchain = Blockchain::default();
        let initial_target = blockchain.target();

        blockchain.try_adjust_target();

        assert_eq!(blockchain.target(), initial_target);
    }

    #[test]
    fn test_blockchain_try_adjust_target_not_at_interval() {
        let mut blockchain = Blockchain::default();
        blockchain.add_block(create_genesis_block()).unwrap();
        let initial_target = blockchain.target();

        blockchain.try_adjust_target();

        // Should not adjust since we're not at DIFFICULTY_UPDATE_INTERVAL
        assert_eq!(blockchain.target(), initial_target);
    }

    #[test]
    fn test_blockchain_mempool_removes_mined_transactions() {
        let mut blockchain = Blockchain::default();

        // Manually add some transactions to mempool
        let tx1 = create_coinbase_transaction(1000);
        let tx2 = create_coinbase_transaction(2000);

        blockchain.mempool.push((Utc::now(), tx1.clone()));
        blockchain.mempool.push((Utc::now(), tx2.clone()));
        assert_eq!(blockchain.mempool().len(), 2);

        // Add genesis block with tx1 in it
        let transactions = vec![tx1];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let header =
            crate::types::BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, MIN_TARGET);
        let block = Block::new(header, transactions);

        blockchain.add_block(block).unwrap();

        // tx1 should be removed from mempool, but tx2 should remain
        assert_eq!(blockchain.mempool().len(), 1);
    }

    #[test]
    fn test_blockchain_blocks_accessor() {
        let mut blockchain = Blockchain::default();
        assert_eq!(blockchain.blocks().len(), 0);

        blockchain.add_block(create_genesis_block()).unwrap();
        assert_eq!(blockchain.blocks().len(), 1);
    }

    #[test]
    fn test_blockchain_mempool_accessor() {
        let blockchain = Blockchain::default();
        let mempool = blockchain.mempool();
        assert_eq!(mempool.len(), 0);
    }

    #[test]
    fn test_blockchain_clone() {
        let blockchain = Blockchain::default();
        let cloned = blockchain.clone();

        assert_eq!(blockchain.block_height(), cloned.block_height());
        assert_eq!(blockchain.target(), cloned.target());
    }
}
