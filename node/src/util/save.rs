use btclib::utils::Saveable;
use log::{info, error};
use tokio::time;

use crate::BLOCKCHAIN;

pub async fn save(name: String) {
    let mut interval = time::interval(time::Duration::from_secs(15));
    loop {
        interval.tick().await;
        info!("saving blockchain to drive...");
        let blockchain = BLOCKCHAIN.read().await;
        if let Err(e) = blockchain.save_to_file(name.clone()) {
            error!("Failed to save blockchain: {}", e);
        } else {
            info!("Blockchain saved successfully");
        }
    }
}
