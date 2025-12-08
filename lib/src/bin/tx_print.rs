use btclib::{types::Transaction, utils::Saveable};

use clap::{Arg, Command};
use log::{error, info};
use std::{fs::File, process::exit};

fn main() {
    env_logger::init();
    let matches = Command::new("tx_print")
        .version("1.0")
        .author("Charalampos Polychronakis <polychronakis.h@gmail.com>")
        .about("Prints the contents of a transaction file")
        .arg(
            Arg::new("tx_file")
                .help("Path to the transaction file to print")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = matches.get_one::<String>("tx_file").unwrap();
    if let Ok(file) = File::open(path) {
        let tx = match Transaction::load(file) {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to load transaction: {}", e);
                exit(1);
            }
        };
        info!("{:#?}", tx);
    } else {
        error!("Failed to open file '{}'", path);
        exit(1);
    }
}
