use btclib::{types::Block, utils::Saveable};

use clap::{Arg, Command};
use log::{error, info};
use std::{fs::File, process::exit};

pub fn main() {
    env_logger::init();

    let matches = Command::new("block_gen")
        .version("1.0")
        .author("Charalampos Polychronakis <polychronakis.h@gmail.com>")
        .about("Prints the contents of a block file")
        .arg(
            Arg::new("block_file")
                .help("Path to the block file to print")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = matches.get_one::<String>("block_file").unwrap();

    match File::open(path) {
        Ok(file) => {
            let block = Block::load(file).expect("Failed to load block");
            info!("{:#?}", block);
        }
        Err(e) => {
            error!("Failed to open file '{}': {}", path, e);
            exit(1);
        }
    }
}
