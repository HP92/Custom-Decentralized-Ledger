use log::info;
use tokio::time;

use crate::{BLOCKCHAIN, NODES};

pub async fn cleanup() {
    let mut interval = time::interval(time::Duration::from_secs(30));
    loop {
        interval.tick().await;

        // Clean mempool
        info!("cleaning the mempool from old transactions");
        {
            let mut blockchain = BLOCKCHAIN.write().await;
            blockchain.cleanup_mempool();
        }

        // Clean stale connections
        info!("checking for stale connections");
        let mut stale_nodes = Vec::new();

        for entry in NODES.iter() {
            let node_addr = entry.key().clone();
            let stream = entry.value();

            // Try to peek at the stream to see if it's still alive
            // If we can't peek, the connection is likely dead
            if stream.peer_addr().is_err() {
                stale_nodes.push(node_addr);
            }
        }

        // Remove stale connections
        for node in stale_nodes {
            info!("Removing stale connection: {}", node);
            NODES.remove(&node);
        }

        info!("Active connections: {}", NODES.len());
    }
}
