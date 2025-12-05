use std::{
    collections::{HashMap, HashSet},
    io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write},
};

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    U256,
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
    pub fn new() -> Self {
        Blockchain {
            utxos: HashMap::new(),
            target: crate::MIN_TARGET,
            blocks: vec![],
            mempool: vec![],
        }
    }

    pub fn utxos(&self) -> HashMap<Hash, TransactionOutput> {
        self.utxos
            .iter()
            .map(|(hash, (_spent, output))| (hash.clone(), output.clone()))
            .collect()
    }

    pub fn target(&self) -> U256 {
        self.target
    }

    pub fn blocks(&self) -> &Vec<Block> {
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
            if block.header.prev_block_hash != Hash::zero() {
                println!(
                    "Previous hash: {:x?} is not equal to zero\n",
                    block.header.prev_block_hash
                );
                return Err(crate::error::BtcError::InvalidBlock);
            }
        } else {
            // if this is not the first block, check if the block's
            // previous hash is the hash of the last block
            let last_block = self.blocks.last().unwrap();
            if block.header.prev_block_hash != last_block.header.hash() {
                println!(
                    "Previous hash: {:x?} is not equal to last block hash: {:x?}\n",
                    block.header.prev_block_hash,
                    last_block.header.hash()
                );
                return Err(crate::error::BtcError::InvalidBlock);
            }

            // check if the block's hash is less than the target
            if !block.header.hash().matches_target(block.header.target) {
                print!(
                    "Does not match target: {:x?} >= {:x?}\n",
                    block.header.hash(),
                    block.header.target
                );
                return Err(crate::error::BtcError::InvalidBlock);
            }

            let calculated_merkle_root = MerkleRoot::calculate(&block.transactions);
            if block.header.merkle_root != calculated_merkle_root {
                print!(
                    "Invalid Merkle root: {:x?} != {:x?}\n",
                    block.header.merkle_root, calculated_merkle_root
                );
                return Err(crate::error::BtcError::InvalidMerkleRoot);
            }

            if block.header.timestamp <= last_block.header.timestamp {
                print!(
                    "Invalid timestamp: {} <= {}\n",
                    block.header.timestamp, last_block.header.timestamp
                );
                return Err(crate::error::BtcError::InvalidBlockHeader);
            }

            block.verify_transactions(self.block_height(), &self.utxos);
        }

        let block_transactions: HashSet<_> =
            block.transactions.iter().map(|tx| tx.hash()).collect();
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
            .header
            .timestamp;
        let end_time = self.blocks.last().unwrap().header.timestamp;
        let time_diff = end_time - start_time;
        // convert time_diff to seconds
        let time_diff_seconds = time_diff.num_seconds();
        // calculate the ideal number of seconds
        let target_seconds = crate::IDEAL_BLOCK_TIME * crate::DIFFICULTY_UPDATE_INTERVAL;
        // multiply the current target by actual time divided by ideal time

        let new_target = BigDecimal::parse_bytes(&self.target.to_string().as_bytes(), 10)
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
            for tx in &block.transactions {
                // Remove spent UTXOs
                for input in &tx.inputs {
                    self.utxos.remove(&input.prev_transaction_output_hash);
                }
                // Add new UTXOs
                for output in tx.outputs.iter() {
                    self.utxos.insert(tx.hash(), (false, output.clone()));
                }
            }
        }
    }

    pub fn add_transaction_to_mempool(&mut self, transaction: Transaction) -> Result<()> {
        // validate transaction before insertion
        // all inputs must match known UTXOs, and must be unique
        let mut known_inputs = HashSet::new();
        for input in &transaction.inputs {
            if !self.utxos.contains_key(&input.prev_transaction_output_hash) {
                println!(
                    "UTXO not found for input {:x?}",
                    input.prev_transaction_output_hash
                );
                return Err(BtcError::InvalidTransaction);
            }
            if known_inputs.contains(&input.prev_transaction_output_hash) {
                println!("duplicate input found");
                return Err(BtcError::InvalidTransaction);
            }
            known_inputs.insert(input.prev_transaction_output_hash);
        }
        // check if any of the utxos have the bool mark set to true
        // and if so, find the transaction that references them
        // in mempool, remove it, and set all the utxos it references
        // to false
        for input in &transaction.inputs {
            if let Some((true, _)) = self.utxos.get(&input.prev_transaction_output_hash) {
                // find the transaction that references the UTXO
                // we are trying to reference
                let referencing_transaction =
                    self.mempool.iter().enumerate().find(|(_, transaction)| {
                        transaction
                            .1
                            .outputs
                            .iter()
                            .any(|output| output.hash() == input.prev_transaction_output_hash)
                    });
                // If we have found one, unmark all of its UTXOs
                if let Some((idx, referencing_transaction)) = referencing_transaction {
                    for input in &referencing_transaction.1.inputs {
                        // set all utxos from this transaction to false
                        self.utxos
                            .entry(input.prev_transaction_output_hash)
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
                        .entry(input.prev_transaction_output_hash)
                        .and_modify(|(marked, _)| {
                            *marked = false;
                        });
                }
            }
        }
        // all inputs must be lower than all outputs
        let all_inputs = transaction
            .inputs
            .iter()
            .map(|input| {
                self.utxos
                    .get(&input.prev_transaction_output_hash)
                    .expect("BUG: impossible")
                    .1 // < - - - Look here
                    .value
            })
            .sum::<u64>();
        let all_outputs = transaction.outputs.iter().map(|output| output.value).sum();
        if all_inputs < all_outputs {
            return Err(BtcError::InvalidTransaction);
        }
        self.mempool.push((Utc::now(), transaction));
        // sort by miner fee descending
        self.mempool.sort_by_key(|transaction| {
            let all_inputs = transaction
                .1
                .inputs
                .iter()
                .map(|input| {
                    self.utxos
                        .get(&input.prev_transaction_output_hash)
                        .expect("BUG: impossible")
                        .1 // < - - - Look here
                        .value
                })
                .sum::<u64>();

            let all_outputs = transaction
                .1
                .outputs
                .iter()
                .map(|output| output.value)
                .sum::<u64>();

            let miner_fee = all_inputs - all_outputs;
            miner_fee
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
                        .inputs
                        .iter()
                        .map(|input| input.prev_transaction_output_hash.clone()),
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
