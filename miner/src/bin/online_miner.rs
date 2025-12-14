use btclib::{crypto::PublicKey, utils::Saveable};
use clap::{Arg, Command};
use log::{debug, info};
use std::process::exit;
// Import Miner from its module (adjust the path if needed)
use miner::Miner;

#[tokio::main]
async fn main() {
    env_logger::init();

    let matches = Command::new("Network Miner")
        .version("1.0")
        .author("Charalampos Polychronakis <polychronakis.h@gmail.com>")
        .about("Connects to a node to mine blocks over the network")
        .arg(
            Arg::new("address")
                .help("Network address to connect to (e.g., 127.0.0.1:8080)")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("public_key_file")
                .help("Path to the public key file")
                .required(true)
                .index(2),
        )
        .get_matches();

    let address = matches.get_one::<String>("address").unwrap().to_string();
    let public_key_file = matches.get_one::<String>("public_key_file").unwrap();

    // Validate address format (should be "host:port")
    if address.matches(':').count() != 1 {
        eprintln!("Invalid address format: '{}'. Expected format is 'host:port' (e.g., 127.0.0.1:8080)", address);
        exit(1);
    }

    let Ok(public_key) = PublicKey::load_from_file(public_key_file) else {
        eprintln!("Error reading public key from file {}", public_key_file);
        exit(1);
    };
    info!("Connecting to {address} to mine");
    debug!("Loaded public key: {:?}", public_key);

    // let mut stream = match TcpStream::connect(&address).await {
    //     Ok(stream) => stream,
    //     Err(e) => {
    //         eprintln!("Failed to connect to server: {}", e);
    //         exit(1);
    //     }
    // };

    // println!("requesting work from {address}");
    // let message = Message::FetchTemplate(public_key);
    // message.send_async(&mut stream).await.unwrap();

    let miner = match Miner::new(address.clone(), public_key).await {
        Ok(miner) => miner,
        Err(e) => {
            eprintln!("Failed to connect to server at {address}: {e}\nIs the node running and listening on {address}?");
            exit(1);
        }
    };
    if let Err(e) = miner.run().await {
        eprintln!("Miner error: {e}");
        exit(1);
    }
}
