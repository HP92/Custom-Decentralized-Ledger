use anyhow::Result;
use clap::Parser;
use node::{
    BLOCKCHAIN, NODES,
    util::{cleanup, save},
};
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::signal;

use node::{
    handler::handle_connection,
    util::{
        Cli, download_blockchain, find_longest_chain_node, load_blockchain, populate_connections,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    log::info!("Port: {}", cli.port());
    log::info!("Blockchain file: {}", cli.blockchain_file());
    log::info!("Nodes: {:?}", cli.nodes());
    let port = cli.port();
    let blockchain_file = cli.blockchain_file();
    let nodes = cli.nodes();

    // Load or initialize the blockchain
    if Path::new(&blockchain_file).exists() {
        log::info!("Loading blockchain from file: {}", blockchain_file);
        load_blockchain(blockchain_file).await?;
    } else {
        log::warn!("Blockchain file does not exist!");
        if !nodes.is_empty() {
            populate_connections(nodes).await?;
            log::info!("Total amount of known nodes: {}", NODES.len());
            let (longest_name, longest_count): (String, _) = find_longest_chain_node().await?;
            // request the blockchain from the node with the longest blockchain
            if longest_count > 0 {
                download_blockchain(&longest_name, longest_count).await?;
                log::info!("Blockchain downloaded from node {}", longest_name);
                // recalculate UTXOs and target
                {
                    let mut blockchain = BLOCKCHAIN.write().await;
                    blockchain.rebuild_utxos();
                }
                // adjust target if necessary
                {
                    let mut blockchain = BLOCKCHAIN.write().await;
                    blockchain.try_adjust_target();
                }
            } else {
                log::info!("Connected nodes have empty blockchains, starting with empty blockchain");
            }
        } else {
            log::info!("No initial nodes provided, starting as a seed node with empty blockchain");
        }
    }

    // Start the server
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    log::info!("Node listening on {}", addr);
    
    // Spawn periodic tasks ONCE (not per connection)
    tokio::spawn(cleanup());
    tokio::spawn(save(blockchain_file.to_string()));
    
    // Connection limiting to prevent DoS
    const MAX_CONNECTIONS: usize = 100;
    let connection_limit = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    
    log::info!("Node ready to accept connections (max: {})", MAX_CONNECTIONS);
    
    loop {
        // Wait for either a new connection or shutdown signal
        tokio::select! {
            // Handle shutdown signal (Ctrl+C)
            _ = signal::ctrl_c() => {
                log::info!("Received shutdown signal, stopping node...");
                break;
            }
            // Accept new connection
            result = listener.accept() => {
                match result {
                    Ok((socket, addr)) => {
                        log::info!("New connection from: {}", addr);
                        
                        // Acquire connection permit
                        let permit = match connection_limit.clone().try_acquire_owned() {
                            Ok(permit) => permit,
                            Err(_) => {
                                log::warn!("Connection limit reached, rejecting connection from {}", addr);
                                continue;
                            }
                        };
                        
                        tokio::spawn(async move {
                            let _permit = permit; // Hold permit until task completes
                            handle_connection(socket).await;
                            log::info!("Connection from {} closed", addr);
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to accept connection: {}", e);
                    }
                }
            }
        }
    }
    
    log::info!("Node shutdown complete");
    Ok(())
}
