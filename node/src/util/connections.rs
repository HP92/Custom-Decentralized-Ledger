use anyhow::Result;
use btclib::network::Message;
use log::{info, warn};
use tokio::net::TcpStream;

use crate::NODES;

pub async fn populate_connections(nodes: &[String]) -> Result<()> {
    info!("trying to connect to other nodes...");
    for node in nodes {
        info!("connecting to {}", node);
        let mut stream = TcpStream::connect(&node).await?;
        let message = Message::DiscoverNodes;
        message.send_async(&mut stream).await?;
        info!("sent DiscoverNodes to {}", node);
        let message = Message::receive_async(&mut stream).await?;
        match message {
            Message::NodeList(child_nodes) => {
                info!("received NodeList from {}", node);
                for child_node in child_nodes {
                    info!("adding node {}", child_node);
                    let new_stream = TcpStream::connect(&child_node).await?;
                    NODES.insert(child_node, new_stream);
                }
            }
            _ => {
                warn!("unexpected message from {}", node);
            }
        }
        NODES.insert(node.clone(), stream);
    }
    Ok(())
}
