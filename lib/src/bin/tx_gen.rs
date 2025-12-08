use btclib::{
    crypto::PrivateKey,
    types::{Transaction, TransactionOutput},
    utils::Saveable,
};

use clap::{Arg, Command};
use log::error;
use std::process::exit;
use uuid::Uuid;

fn main() {
    env_logger::init();

    let matches = Command::new("tx_gen")
        .version("1.0")
        .author("Charalampos Polychronakis <polychronakis.h@gmail.com>")
        .about("Generates a new transaction and saves it to a file")
        .arg(
            Arg::new("tx_file")
                .help("Path to the output transaction file")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = matches.get_one::<String>("tx_file").unwrap();

    let private_key = PrivateKey::default();
    let transaction = Transaction::new(
        vec![],
        vec![TransactionOutput::new(
            btclib::INITIAL_REWARD * 10u64.pow(8),
            Uuid::new_v4(),
            private_key.public_key(),
        )],
    );

    match transaction.save_to_file(path) {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to save transaction: {}", e);
            exit(1);
        }
    };
}
