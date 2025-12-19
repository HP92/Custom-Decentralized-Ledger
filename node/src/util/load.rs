use anyhow::Result;
use btclib::{types::Blockchain, utils::Saveable};
use log::info;

use crate::BLOCKCHAIN;

pub async fn load_blockchain(blockchain_file: &str) -> Result<()> {
    info!("blockchain file exists, loading...");
    let new_blockchain = Blockchain::load_from_file(blockchain_file)?;
    info!("blockchain loaded");
    let mut blockchain = BLOCKCHAIN.write().await;
    *blockchain = new_blockchain;
    info!("rebuilding utxos...");
    blockchain.rebuild_utxos();
    info!("utxos rebuilt");
    info!("checking if target needs to be adjusted...");
    info!("current target: {}", blockchain.target());
    blockchain.try_adjust_target();
    info!("new target: {}", blockchain.target());
    info!("initialization complete");
    Ok(())
}
