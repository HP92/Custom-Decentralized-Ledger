use btclib::{crypto::PublicKey, utils::Saveable};
use clap::{Arg, Command};
use log::{debug, error, info};
use std::process::exit;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::signal;
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
        error!(
            "Invalid address format: '{}'. Expected format is 'host:port' (e.g., 127.0.0.1:8080)",
            address
        );
        exit(1);
    }

    let Ok(public_key) = PublicKey::load_from_file(public_key_file) else {
        error!("Error reading public key from file {}", public_key_file);
        exit(1);
    };
    info!("Connecting to {} to mine", address);
    debug!("Loaded public key: {:?}", public_key);

    // let mut stream = match TcpStream::connect(&address).await {
    //     Ok(stream) => stream,
    //     Err(e) => {
    //         error!("Failed to connect to server: {}", e);
    //         exit(1);
    //     }
    // };

    // info!("Requesting work from {}", address);
    // let message = Message::FetchTemplate(public_key);
    // message.send_async(&mut stream).await.unwrap();

    let miner = match Miner::new(address.clone(), public_key).await {
        Ok(miner) => miner,
        Err(e) => {
            error!(
                "Failed to connect to server at {}: {}\nIs the node running and listening on {}?",
                address, e, address
            );
            exit(1);
        }
    };

    // Create a shared AtomicBool for graceful shutdown or control
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    // Spawn a task to handle Ctrl+C
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        info!("Received shutdown signal (Ctrl+C), stopping miner...");
        running_clone.store(false, Ordering::SeqCst);
    });
    
    if let Err(e) = miner.run(running.clone()).await {
        error!("Miner error: {}", e);
        exit(1);
    }
    
    info!("Miner shutdown complete");
}
