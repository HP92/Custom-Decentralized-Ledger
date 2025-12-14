use btclib::{types::Block, utils::Saveable};
use clap::{Arg, Command};
use log::info;

fn main() {
    env_logger::init();

    let matches = Command::new("CPU Miner")
        .version("1.0")
        .author("Charalampos Polychronakis <polychronakis.h@gmail.com>")
        .about("Reads a block template file, mines the block in specified increments, and prints the original and mined blocks with their hashes")
        .arg(
            Arg::new("block_file")
                .help("Path to the output block file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("steps")
                .help("Number of mining steps")
                .required(true)
                .index(2)
                .value_parser(clap::value_parser!(usize)),
        )
        .get_matches();

    // Get block path and steps count from clap matches
    let path = matches.get_one::<String>("block_file").unwrap().to_string();
    let steps = *matches.get_one::<usize>("steps").unwrap();

    let og_block = Block::load_from_file(path).expect("Failed to load block");
    let mut block = og_block.clone();

    while !block.header_mut().mine(steps) {
        info!("mining...");
    }

    // print original block and its hash
    info!("original: {:#?}", og_block);
    info!("hash: {:?}", og_block.header().hash());
    // print mined block and its hash
    info!("final: {:#?}", block);
    info!("hash: {:?}", block.header().hash());
}
