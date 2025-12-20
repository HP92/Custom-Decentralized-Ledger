use std::{sync::Arc, time::Duration};

use btclib::types::Transaction;
use tokio::time;

use crate::models::Core;

pub async fn update_utxos(core: Arc<Core>) {
    let mut interval = time::interval(Duration::from_secs(20));
    loop {
        interval.tick().await;
        if let Err(e) = core.fetch_utxos().await {
            log::error!("Failed to update UTXOs: {}", e);
        }
    }
}

pub async fn handle_transactions(rx: kanal::AsyncReceiver<Transaction>, core: Arc<Core>) {
    while let Ok(transaction) = rx.recv().await {
        if let Err(e) = core.send_transaction(transaction).await {
            log::error!("Failed to send transaction: {}", e);
        }
    }
}
