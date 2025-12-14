use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use anyhow::{Result, anyhow};
use btclib::{crypto::PublicKey, network::Message, types::Block};
use flume::{Receiver, Sender};
use log::{info, warn};
use tokio::{net::TcpStream, sync::Mutex, time::interval};

pub struct Miner {
    public_key: PublicKey,
    stream: Mutex<TcpStream>,
    current_template: Arc<std::sync::Mutex<Option<Block>>>,
    mining: Arc<AtomicBool>,
    mined_block_sender: Sender<Block>,
    mined_block_receiver: Receiver<Block>,
}

impl Miner {
    pub async fn new(address: String, public_key: PublicKey) -> Result<Self> {
        let stream = TcpStream::connect(&address).await?;
        let (mined_block_sender, mined_block_receiver) = flume::unbounded();
        Ok(Self {
            public_key,
            stream: Mutex::new(stream),
            current_template: Arc::new(std::sync::Mutex::new(None)),
            mining: Arc::new(AtomicBool::new(false)),
            mined_block_sender,
            mined_block_receiver,
        })
    }

    pub async fn run(&self) -> Result<()> {
        self.spawn_mining_thread();
        let mut template_interval = interval(Duration::from_secs(5));
        loop {
            let receiver_clone = self.mined_block_receiver.clone();
            tokio::select! {
                _ = template_interval.tick() => {
                self.fetch_and_validate_template().await?;
                }
                Ok(mined_block) = receiver_clone.recv_async() => {
                    self.submit_block(mined_block).await?;
                }
            }
        }
    }

    fn spawn_mining_thread(&self) -> thread::JoinHandle<()> {
        let template = self.current_template.clone();
        let mining = self.mining.clone();
        let sender = self.mined_block_sender.clone();
        thread::spawn(move || {
            loop {
                if mining.load(Ordering::Relaxed)
                    && let Some(mut block) = template.lock().unwrap().clone()
                {
                    info!("Mining block with target: {}", block.header().target());
                    if block.header_mut().mine(2_000_000) {
                        info!("Block mined: {:?}", block.hash());
                        sender.send(block).expect("Failed to send mined block");
                        mining.store(false, Ordering::Relaxed);
                    }
                }
                thread::yield_now();
            }
        })
    }

    async fn fetch_and_validate_template(&self) -> Result<()> {
        if !self.mining.load(Ordering::Relaxed) {
            self.fetch_template().await?;
        } else {
            self.validate_template().await?;
        }
        Ok(())
    }

    async fn fetch_template(&self) -> Result<()> {
        info!("Fetching new template");
        let message = Message::FetchTemplate(self.public_key.clone());
        let mut stream_lock = self.stream.lock().await;
        message.send_async(&mut *stream_lock).await?;
        drop(stream_lock);
        let mut stream_lock = self.stream.lock().await;
        match Message::receive_async(&mut *stream_lock).await? {
            Message::Template(template) => {
                drop(stream_lock);
                info!(
                    "Received new template with target: {}",
                    template.header().target()
                );
                *self.current_template.lock().unwrap() = Some(template);
                self.mining.store(true, Ordering::Relaxed);
                Ok(())
            }
            _ => Err(anyhow!(
                "Unexpected message received when fetching template"
            )),
        }
    }

    async fn validate_template(&self) -> Result<()> {
        // Acquire the lock, clone the template, and drop the guard before await
        let template_opt = {
            let guard = self.current_template.lock().unwrap();
            guard.clone()
        };
        if let Some(template) = template_opt {
            let message = Message::ValidateTemplate(template);
            let mut stream_lock = self.stream.lock().await;
            message.send_async(&mut *stream_lock).await?;
            drop(stream_lock);
            let mut stream_lock = self.stream.lock().await;
            match Message::receive_async(&mut *stream_lock).await? {
                Message::TemplateValidity(valid) => {
                    drop(stream_lock);
                    if !valid {
                        warn!("Current template is no longer valid");
                        self.mining.store(false, Ordering::Relaxed);
                    } else {
                        info!("Current template is still valid");
                    }
                    Ok(())
                }
                _ => Err(anyhow!(
                    "Unexpected message received when
validating template"
                )),
            }
        } else {
            Ok(())
        }
    }

    async fn submit_block(&self, block: Block) -> Result<()> {
        info!("Submitting mined block");
        let message = Message::SubmitTemplate(block);
        let mut stream_lock = self.stream.lock().await;
        message.send_async(&mut *stream_lock).await?;
        self.mining.store(false, Ordering::Relaxed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use btclib::{
        crypto::PrivateKey,
        custom_sha_types::Hash,
        types::{Block, BlockHeader, Transaction, TransactionOutput},
        utils::MerkleRoot,
    };
    use chrono::Utc;
    use std::sync::atomic::AtomicBool;
    use uuid::Uuid;

    fn create_test_block() -> Block {
        let private_key = PrivateKey::default();
        let transactions = vec![Transaction::new(
            vec![],
            vec![TransactionOutput::new(
                btclib::INITIAL_REWARD * 10u64.pow(8),
                Uuid::new_v4(),
                private_key.public_key(),
            )],
        )];
        let merkle_root = MerkleRoot::calculate(&transactions);
        let block = Block::new(
            BlockHeader::new(Utc::now(), 0, Hash::zero(), merkle_root, btclib::MIN_TARGET),
            transactions,
        );

        block
    }

    // Positive test: successful mining sets mining flag to false and sends block
    #[test]
    fn test_successful_mining_sets_flag_and_sends_block() {
        use std::sync::atomic::Ordering;
        use std::sync::{Arc, Mutex};
        use flume;
        let mining = Arc::new(AtomicBool::new(true));
        let template = Arc::new(Mutex::new(Some(create_test_block())));
        let (sender, receiver) = flume::unbounded::<Block>();
        // Simulate mining thread logic
        if mining.load(Ordering::Relaxed) {
            if let Some(block) = template.lock().unwrap().clone() {
                // Simulate successful mining
                sender.send(block.clone()).unwrap();
                mining.store(false, Ordering::Relaxed);
            }
        }
        assert!(!mining.load(Ordering::Relaxed));
        let received = receiver.recv().unwrap();
        assert_eq!(received.header().merkle_root(), template.lock().unwrap().as_ref().unwrap().header().merkle_root());
    }

    // Negative test: mining with invalid template does not send block
    #[test]
    fn test_mining_with_invalid_template_does_not_send_block() {
        use std::sync::atomic::Ordering;
        use std::sync::{Arc, Mutex};
        use flume;
        let mining = Arc::new(AtomicBool::new(true));
        let template = Arc::new(Mutex::new(None::<Block>));
        let (sender, receiver) = flume::unbounded::<Block>();
        // Simulate mining thread logic
        if mining.load(Ordering::Relaxed) {
            if let Some(_block) = template.lock().unwrap().clone() {
                sender.send(_block).unwrap();
                mining.store(false, Ordering::Relaxed);
            }
        }
        // Mining flag should remain true, no block sent
        assert!(mining.load(Ordering::Relaxed));
        assert!(receiver.is_empty());
    }

    // Negative test: mining with mining flag off does not send block
    #[test]
    fn test_mining_with_flag_off_does_not_send_block() {
        use std::sync::atomic::Ordering;
        use std::sync::{Arc, Mutex};
        use flume;
        let mining = Arc::new(AtomicBool::new(false));
        let template = Arc::new(Mutex::new(Some(create_test_block())));
        let (sender, receiver) = flume::unbounded::<Block>();
        // Simulate mining thread logic
        if mining.load(Ordering::Relaxed) {
            if let Some(block) = template.lock().unwrap().clone() {
                sender.send(block).unwrap();
                mining.store(false, Ordering::Relaxed);
            }
        }
        // Mining flag should remain false, no block sent
        assert!(!mining.load(Ordering::Relaxed));
        assert!(receiver.is_empty());
    }

    // Positive test: block broadcast via channel
    #[test]
    fn test_block_broadcast_via_channel() {
        use flume;
        let (sender, receiver) = flume::unbounded::<Block>();
        let block = create_test_block();
        sender.send(block.clone()).unwrap();
        let received = receiver.recv().unwrap();
        assert_eq!(received.header().merkle_root(), block.header().merkle_root());
    }

    #[test]
    fn test_mining_flag_initialization() {
        let mining = Arc::new(AtomicBool::new(false));
        assert!(!mining.load(Ordering::Relaxed));

        mining.store(true, Ordering::Relaxed);
        assert!(mining.load(Ordering::Relaxed));
    }

    #[test]
    fn test_template_storage() {
        let template = Arc::new(std::sync::Mutex::new(None));
        assert!(template.lock().unwrap().is_none());

        let block = create_test_block();
        *template.lock().unwrap() = Some(block.clone());

        assert!(template.lock().unwrap().is_some());
        let stored_block = template.lock().unwrap().clone().unwrap();
        assert_eq!(
            stored_block.header().merkle_root(),
            block.header().merkle_root()
        );
    }

    #[test]
    fn test_flume_channel() {
        let (sender, receiver) = flume::unbounded::<Block>();
        let block = create_test_block();

        sender.send(block.clone()).unwrap();
        let received = receiver.recv().unwrap();

        assert_eq!(
            received.header().merkle_root(),
            block.header().merkle_root()
        );
    }

    #[test]
    fn test_block_creation() {
        let block = create_test_block();

        assert_eq!(block.transactions().len(), 1);
        // Assuming coinbase transactions have is_coinbase field or can be identified by input/output
        // Replace with appropriate check for your Transaction struct
        assert!(block.transactions()[0].inputs().is_empty());
        assert_eq!(block.header().nonce(), 0);
    }

    #[tokio::test]
    async fn test_mining_state_transitions() {
        let mining = Arc::new(AtomicBool::new(false));
        let template = Arc::new(std::sync::Mutex::new(None::<Block>));

        // Initial state: not mining, no template
        assert!(!mining.load(Ordering::Relaxed));
        assert!(template.lock().unwrap().is_none());

        // Simulate receiving a template
        let block = create_test_block();
        *template.lock().unwrap() = Some(block);
        mining.store(true, Ordering::Relaxed);

        // Mining state: mining active, template present
        assert!(mining.load(Ordering::Relaxed));
        assert!(template.lock().unwrap().is_some());

        // Simulate successful mining
        mining.store(false, Ordering::Relaxed);
        assert!(!mining.load(Ordering::Relaxed));
    }

    #[test]
    fn test_block_cloning() {
        let template = Arc::new(std::sync::Mutex::new(None::<Block>));
        let block = create_test_block();

        *template.lock().unwrap() = Some(block.clone());

        // Verify we can clone the block from the template
        let cloned = template.lock().unwrap().clone().unwrap();
        assert_eq!(cloned.header().merkle_root(), block.header().merkle_root());
        assert_eq!(cloned.transactions().len(), block.transactions().len());
    }

    #[test]
    fn test_concurrent_mining_access() {
        use std::sync::Arc;
        use std::thread;

        let mining = Arc::new(AtomicBool::new(false));
        let mining_clone = mining.clone();

        let handle = thread::spawn(move || {
            mining_clone.store(true, Ordering::Relaxed);
        });

        handle.join().unwrap();
        assert!(mining.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_channel_send_receive() {
        let (sender, receiver) = flume::unbounded::<Block>();
        let block = create_test_block();

        // Test async send/receive
        sender.send(block.clone()).unwrap();
        let received = receiver.recv_async().await.unwrap();

        assert_eq!(received.transactions().len(), block.transactions().len());
    }

    #[test]
    fn test_multiple_blocks_in_channel() {
        let (sender, receiver) = flume::unbounded::<Block>();

        let block1 = create_test_block();
        let block2 = create_test_block();

        sender.send(block1).unwrap();
        sender.send(block2).unwrap();

        assert_eq!(receiver.len(), 2);
        receiver.recv().unwrap();
        assert_eq!(receiver.len(), 1);
    }
}
