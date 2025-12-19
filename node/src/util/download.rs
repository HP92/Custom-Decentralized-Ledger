use anyhow::Result;
use btclib::network::Message;

use crate::BLOCKCHAIN;

pub async fn download_blockchain(node: &str, count: u32) -> Result<()> {
    let mut stream = crate::NODES.get_mut(node).unwrap();
    for i in 0..count as usize {
        let message = Message::FetchBlock(i);
        message.send_async(&mut *stream).await?;
        let message = Message::receive_async(&mut *stream).await?;
        match message {
            Message::NewBlock(block) => {
                let mut blockchain = BLOCKCHAIN.write().await;
                blockchain.add_block(block)?;
            }
            _ => {
                log::info!("unexpected message from {}", node);
            }
        }
    }
    Ok(())
}

// TODO: This is another spot where an improvement could be made. Instead of making
// many small requests, we could add another message type that would return an
// entire chain of blocks. That’s it for the helper functions in utils.rs for this bit.
