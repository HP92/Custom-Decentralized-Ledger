use std::process::exit;

use btclib::{
    crypto::PrivateKey,
    custom_sha_types::Hash,
    types::{Block, BlockHeader, Transaction, TransactionOutput},
    utils::{MerkleRoot, Saveable},
};

use chrono::Utc;
use clap::{Arg, Command};
use log::error;
use uuid::Uuid;

fn main() {
    env_logger::init();

    let matches = Command::new("block_gen")
        .version("1.0")
        .author("Charalampos Polychronakis <polychronakis.h@gmail.com>")
        .about("Generates a new block and saves it to a file")
        .arg(
            Arg::new("block_file")
                .help("Path to the output block file")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = matches.get_one::<String>("block_file").unwrap();
    let private_key = PrivateKey::default();
    let transactions = vec![Transaction::new(
        vec![],
        vec![TransactionOutput::new(
            btclib::INITIAL_REWARD * 10u64.pow(8),
            Uuid::new_v4(),
            private_key.public_key(),
        )],
    )];
    let merkle_root = MerkleRoot::calculate(&transactions);
    let block = Block::new(
        BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, btclib::MIN_TARGET),
        transactions,
    );
    match block.save_to_file(path) {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to save block: {}", e);
            exit(1);
        }
    };
}
