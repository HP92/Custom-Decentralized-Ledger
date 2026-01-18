use anyhow::{Context, Result};
use btclib::network::Message;
use log::info;

use crate::NODES;

pub async fn find_longest_chain_node() -> Result<(String, u32)> {
    info!("finding nodes with the highest blockchain length...");
    let mut longest_name = String::new();
    let mut longest_count = 0;
    let all_nodes = NODES.iter().map(|x| x.key().clone()).collect::<Vec<_>>();
    for node in all_nodes {
        info!("asking {} for blockchain length", node);
        let mut stream = NODES.get_mut(&node).context("no node")?;
        let message = Message::AskDifference(0);
        message.send_async(&mut *stream).await.unwrap();
        info!("sent AskDifference to {}", node);
        let message = Message::receive_async(&mut *stream).await?;
        match message {
            Message::Difference(count) => {
                info!("received Difference from {}", node);
                if count > longest_count {
                    info!(
                        "new longest blockchain: \
 {} blocks from {node}",
                        count
                    );
                    longest_count = count;
                    longest_name = node;
                }
            }
            e => {
                info!("unexpected message from {}: {:?}", node, e);
            }
        }
    }
    Ok((longest_name, longest_count as u32))
}

// TODO: a proper implementation of a consensus algorithm
// for now, just find the node with the longest chain
// and download the blockchain from it
// returns the name and length of the longest chain node
