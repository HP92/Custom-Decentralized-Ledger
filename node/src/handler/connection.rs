use btclib::{
    custom_sha_types::Hash,
    network::Message::{
        self, AskDifference, Difference, DiscoverNodes, FetchBlock, FetchTemplate, FetchUTXOs,
        NewBlock, NewTransaction, NodeList, SubmitTemplate, SubmitTransaction, Template,
        TemplateValidity, UTXOs, ValidateTemplate,
    },
    types::{Block, BlockHeader, Transaction, TransactionOutput},
    utils::MerkleRoot,
};
use chrono::Utc;
use log::error;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::{BLOCKCHAIN, NODES};

pub async fn handle_connection(mut socket: TcpStream) {
    loop {
        // read a message from the socket
        let message = match Message::receive_async(&mut socket).await {
            Ok(message) => message,
            Err(e) => {
                error!("invalid message from peer: {e}, closing that connection");
                return;
            }
        };
        match message {
            UTXOs(_) | Template(_) | Difference(_) | TemplateValidity(_) | NodeList(_) => {
                log::info!(
                    "I am neither a miner nor a \
            wallet! Goodbye"
                );
                return;
            }
            FetchBlock(height) => {
                let blockchain = BLOCKCHAIN.read().await;
                let Some(block) = blockchain.blocks().get(height).cloned() else {
                    log::warn!("Block at height {} not found", height);
                    return;
                };
                let message = NewBlock(block);
                if let Err(e) = message.send_async(&mut socket).await {
                    log::error!("Failed to send block: {}", e);
                    return;
                }
            }

            DiscoverNodes => {
                let nodes = crate::NODES
                    .iter()
                    .map(|x| x.key().clone())
                    .collect::<Vec<_>>();
                let message = NodeList(nodes);
                if let Err(e) = message.send_async(&mut socket).await {
                    log::error!("Failed to send node list: {}", e);
                    return;
                }
            }

            AskDifference(height) => {
                let blockchain = BLOCKCHAIN.read().await;
                let count = blockchain.block_height() as i32 - height as i32;
                let message = Difference(count);
                if let Err(e) = message.send_async(&mut socket).await {
                    log::error!("Failed to send difference: {}", e);
                    return;
                }
            }

            FetchUTXOs(key) => {
                log::info!("received request to fetch UTXOs");
                let blockchain = BLOCKCHAIN.read().await;
                let utxos = blockchain
                    .utxos()
                    .iter()
                    .filter(|(_, txout)| txout.pubkey() == &key)
                    .map(|(_, txout)| (txout.clone(), false))
                    .collect::<Vec<_>>();
                let message = UTXOs(utxos);
                if let Err(e) = message.send_async(&mut socket).await {
                    log::error!("Failed to send UTXOs: {}", e);
                    return;
                }
            }
            NewBlock(block) => {
                let mut blockchain = BLOCKCHAIN.write().await;
                log::info!("received new block");
                if blockchain.add_block(block).is_err() {
                    log::info!("block rejected");
                }
            }
            NewTransaction(tx) => {
                let mut blockchain = BLOCKCHAIN.write().await;
                log::info!("received transaction from friend");
                if blockchain.add_transaction_to_mempool(tx).is_err() {
                    log::info!("transaction rejected, closing connection");
                    return;
                }

                // TODO: We are making a simplification here in that we just add it to the mempool. It would
                // be a nice idea to send it back to other nodes that may not have it. However, we would
                // have to add a mechanism for preventing the network from creating notification
                // loops. You can try implementing one, if you want.
            }
            ValidateTemplate(block_template) => {
                let blockchain = BLOCKCHAIN.read().await;
                let status = *block_template.header().prev_block_hash()
                    == blockchain
                        .blocks()
                        .last()
                        .map(|last_block| last_block.hash())
                        .unwrap_or(Hash::zero());
                let message = TemplateValidity(status);
                if let Err(e) = message.send_async(&mut socket).await {
                    log::error!("Failed to send template validity: {}", e);
                    return;
                }
            }
            SubmitTemplate(block) => {
                log::info!("received allegedly mined template");
                let mut blockchain = BLOCKCHAIN.write().await;
                if let Err(e) = blockchain.add_block(block.clone()) {
                    log::info!("block rejected: {e}, closing connection");
                    return;
                }
                blockchain.rebuild_utxos();
                log::info!("block looks good, broadcasting");
                // send block to all friend nodes
                let nodes = crate::NODES
                    .iter()
                    .map(|x| x.key().clone())
                    .collect::<Vec<_>>();
                for node in nodes {
                    if let Some(mut stream) = NODES.get_mut(&node) {
                        let message = Message::NewBlock(block.clone());
                        if message.send_async(&mut *stream).await.is_err() {
                            log::info!("failed to send new block to node");
                        }
                    }
                }
            }
            SubmitTransaction(tx) => {
                log::info!("submit tx");
                let mut blockchain = crate::BLOCKCHAIN.write().await;
                if let Err(e) = blockchain.add_transaction_to_mempool(tx.clone()) {
                    log::info!("transaction rejected, closing connection: {e}");
                    return;
                }
                log::info!("added transaction to mempool");
                // send transaction to all friend nodes
                let nodes = crate::NODES
                    .iter()
                    .map(|x| x.key().clone())
                    .collect::<Vec<_>>();
                for node in nodes {
                    log::info!("sending to friend: {node}");
                    if let Some(mut stream) = crate::NODES.get_mut(&node) {
                        let message = Message::NewTransaction(tx.clone());
                        if message.send_async(&mut *stream).await.is_err() {
                            log::info!("failed to send transaction to {}", node);
                        }
                    }
                }
                log::info!("transaction sent to friends");
            }
            FetchTemplate(pubkey) => {
                let blockchain = crate::BLOCKCHAIN.read().await;
                let mut transactions = vec![];
                // insert transactions from mempool
                transactions.extend(
                    blockchain
                        .mempool()
                        .iter()
                        .take(btclib::BLOCK_TRANSACTION_CAP)
                        .map(|(_, tx)| tx)
                        .cloned()
                        .collect::<Vec<_>>(),
                );
                // insert coinbase tx with pubkey
                transactions.insert(
                    0,
                    Transaction::new(
                        vec![],
                        vec![TransactionOutput::new(0, Uuid::new_v4(), pubkey.clone())],
                    ),
                );
                let merkle_root = MerkleRoot::calculate(&transactions);
                let header = BlockHeader::new(
                    Utc::now(),
                    0,
                    blockchain
                        .blocks()
                        .last()
                        .map(|last_block| last_block.hash())
                        .unwrap_or(Hash::zero()),
                    merkle_root,
                    blockchain.target(),
                );
                let mut block = Block::new(header, transactions);
                let miner_fees = match block.calculated_miner_fees(
                    &blockchain
                        .utxos()
                        .iter()
                        .map(|(k, v)| (*k, (false, v.clone())))
                        .collect(),
                ) {
                    Ok(fees) => fees,
                    Err(e) => {
                        log::error!("Failed to calculate miner fees: {e}");
                        return;
                    }
                };
                let reward = blockchain.calculate_block_reward();
                // update coinbase tx with reward and recalculate merkle root
                let mut updated_transactions = block.transactions().clone();
                updated_transactions[0] = Transaction::new(
                    vec![],
                    vec![TransactionOutput::new(
                        reward + miner_fees,
                        Uuid::new_v4(),
                        pubkey,
                    )],
                );
                let new_merkle_root = MerkleRoot::calculate(&updated_transactions);
                let updated_header = BlockHeader::new(
                    block.header().timestamp(),
                    0,
                    *block.header().prev_block_hash(),
                    new_merkle_root,
                    blockchain.target(),
                );
                block = Block::new(updated_header, updated_transactions);
                let message = Template(block);
                if let Err(e) = message.send_async(&mut socket).await {
                    log::error!("Failed to send template: {}", e);
                    return;
                }
            }
        }
    }
}
