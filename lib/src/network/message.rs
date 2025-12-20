use crate::{
    crypto::PublicKey,
    types::{Block, Transaction, TransactionOutput},
};
use serde::{Deserialize, Serialize};
use std::io::{Error as IoError, Read, Write};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    /// Fetch all UTXOs belonging to a public key
    FetchUTXOs(PublicKey),
    /// UTXOs belonging to a public key. Bool determines if marked
    UTXOs(Vec<(TransactionOutput, bool)>),
    /// Send a transaction to the network
    SubmitTransaction(Transaction),
    /// Broadcast a new transaction to other nodes
    NewTransaction(Transaction),
    /// Ask the node to prepare the optimal block template
    /// with the coinbase transaction paying the specified
    /// public key
    FetchTemplate(PublicKey),
    /// The template
    Template(Block),
    /// Ask the node to validate a block template.
    /// This is to prevent the node from mining an invalid
    /// block (e.g. if one has been found in the meantime,
    /// or if transactions have been removed from the mempool)
    ValidateTemplate(Block),
    /// If template is valid
    TemplateValidity(bool),
    /// Submit a mined block to a node
    SubmitTemplate(Block),
    /// Ask a node to report all the other nodes it knows
    /// about
    DiscoverNodes,
    /// This is the response to DiscoverNodes
    NodeList(Vec<String>),
    /// Ask a node whats the highest block it knows about
    /// in comparison to the local blockchain
    AskDifference(u32),
    /// This is the response to AskDifference
    Difference(i32),
    /// Ask a node to send a block with the specified height
    FetchBlock(usize),
    /// Broadcast a new block to other nodes
    NewBlock(Block),
}

impl Message {
    pub fn encode(&self) -> Result<Vec<u8>, ciborium::ser::Error<IoError>> {
        let mut bytes = Vec::new();
        ciborium::into_writer(self, &mut bytes)?;
        Ok(bytes)
    }

    pub fn decode(data: &[u8]) -> Result<Self, ciborium::de::Error<IoError>> {
        ciborium::from_reader(data)
    }

    pub fn send(&self, stream: &mut impl Write) -> Result<(), ciborium::ser::Error<IoError>> {
        let bytes = self.encode()?;
        let len = bytes.len() as u64;
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(&bytes)?;
        Ok(())
    }

    const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10 MB

    pub fn receive(stream: &mut impl Read) -> Result<Self, ciborium::de::Error<IoError>> {
        let mut len_bytes = [0u8; 8];
        stream.read_exact(&mut len_bytes)?;
        let len = u64::from_be_bytes(len_bytes) as usize;
        if len > Self::MAX_MESSAGE_SIZE {
            return Err(ciborium::de::Error::Io(IoError::new(
                std::io::ErrorKind::InvalidData,
                "Message size exceeds maximum allowed",
            )));
        }
        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer)?;
        Self::decode(&buffer)
    }

    pub async fn send_async(
        &self,
        stream: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), ciborium::ser::Error<IoError>> {
        let bytes = self.encode()?;
        let len = bytes.len() as u64;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&bytes).await?;
        stream.flush().await?;
        Ok(())
    }

    pub async fn receive_async(
        stream: &mut (impl AsyncRead + Unpin),
    ) -> Result<Self, ciborium::de::Error<IoError>> {
        let mut len_bytes = [0u8; 8];
        stream.read_exact(&mut len_bytes).await?;
        let len = u64::from_be_bytes(len_bytes) as usize;
        if len > Self::MAX_MESSAGE_SIZE {
            return Err(ciborium::de::Error::Io(IoError::new(
                std::io::ErrorKind::InvalidData,
                "Message size exceeds maximum allowed",
            )));
        }
        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer).await?;
        Self::decode(&buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{crypto::PrivateKey, types::Block};
    use std::io::Cursor;

    fn create_test_transaction() -> Transaction {
        let private = PrivateKey::default();
        let public = private.public_key();
        let output = TransactionOutput::new(100, uuid::Uuid::new_v4(), public);
        Transaction::new(vec![], vec![output])
    }

    fn create_test_block() -> Block {
        use crate::types::BlockHeader;
        use crate::custom_sha_types::Hash;
        use crate::utils::MerkleRoot;
        use chrono::Utc;
        use crate::MIN_TARGET;
        
        let tx = create_test_transaction();
        let header = BlockHeader::new(
            Utc::now(),
            0,
            Hash::zero(),
            MerkleRoot::calculate(&[tx.clone()]),
            MIN_TARGET,
        );
        Block::new(header, vec![tx])
    }

    #[test]
    fn test_message_encode_decode() {
        let tx = create_test_transaction();
        let msg = Message::NewTransaction(tx.clone());
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::NewTransaction(decoded_tx) = decoded {
            assert_eq!(tx.hash(), decoded_tx.hash());
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_fetch_utxos() {
        let private = PrivateKey::default();
        let public = private.public_key();
        let msg = Message::FetchUTXOs(public.clone());
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::FetchUTXOs(decoded_public) = decoded {
            assert_eq!(public, decoded_public);
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_utxos() {
        let private = PrivateKey::default();
        let public = private.public_key();
        let output = TransactionOutput::new(100, uuid::Uuid::new_v4(), public);
        let utxos = vec![(output.clone(), false)];
        let msg = Message::UTXOs(utxos);
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::UTXOs(decoded_utxos) = decoded {
            assert_eq!(decoded_utxos.len(), 1);
            assert_eq!(decoded_utxos[0].0.hash(), output.hash());
            assert_eq!(decoded_utxos[0].1, false);
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_submit_transaction() {
        let tx = create_test_transaction();
        let msg = Message::SubmitTransaction(tx.clone());
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::SubmitTransaction(decoded_tx) = decoded {
            assert_eq!(tx.hash(), decoded_tx.hash());
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_fetch_template() {
        let private = PrivateKey::default();
        let public = private.public_key();
        let msg = Message::FetchTemplate(public.clone());
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::FetchTemplate(decoded_public) = decoded {
            assert_eq!(public, decoded_public);
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_template() {
        let block = create_test_block();
        let msg = Message::Template(block.clone());
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::Template(decoded_block) = decoded {
            assert_eq!(block.hash(), decoded_block.hash());
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_validate_template() {
        let block = create_test_block();
        let msg = Message::ValidateTemplate(block.clone());
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::ValidateTemplate(decoded_block) = decoded {
            assert_eq!(block.hash(), decoded_block.hash());
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_template_validity() {
        let msg = Message::TemplateValidity(true);
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::TemplateValidity(valid) = decoded {
            assert_eq!(valid, true);
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_submit_template() {
        let block = create_test_block();
        let msg = Message::SubmitTemplate(block.clone());
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::SubmitTemplate(decoded_block) = decoded {
            assert_eq!(block.hash(), decoded_block.hash());
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_discover_nodes() {
        let msg = Message::DiscoverNodes;
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        matches!(decoded, Message::DiscoverNodes);
    }

    #[test]
    fn test_message_node_list() {
        let nodes = vec!["127.0.0.1:8333".to_string(), "192.168.1.1:8333".to_string()];
        let msg = Message::NodeList(nodes.clone());
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::NodeList(decoded_nodes) = decoded {
            assert_eq!(decoded_nodes, nodes);
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_ask_difference() {
        let msg = Message::AskDifference(100);
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::AskDifference(height) = decoded {
            assert_eq!(height, 100);
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_difference() {
        let msg = Message::Difference(42);
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::Difference(diff) = decoded {
            assert_eq!(diff, 42);
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_fetch_block() {
        let msg = Message::FetchBlock(10);
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::FetchBlock(height) = decoded {
            assert_eq!(height, 10);
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_new_block() {
        let block = create_test_block();
        let msg = Message::NewBlock(block.clone());
        
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();
        
        if let Message::NewBlock(decoded_block) = decoded {
            assert_eq!(block.hash(), decoded_block.hash());
        } else {
            panic!("Decoded message type mismatch");
        }
    }

    #[test]
    fn test_message_send_receive() {
        let tx = create_test_transaction();
        let msg = Message::NewTransaction(tx.clone());
        
        let mut buffer = Cursor::new(Vec::new());
        msg.send(&mut buffer).unwrap();
        
        buffer.set_position(0);
        let received = Message::receive(&mut buffer).unwrap();
        
        if let Message::NewTransaction(received_tx) = received {
            assert_eq!(tx.hash(), received_tx.hash());
        } else {
            panic!("Received message type mismatch");
        }
    }

    #[test]
    fn test_message_size_limit() {
        use std::io::Write;
        
        // Create a very large message that exceeds the limit
        let mut buffer = Cursor::new(Vec::new());
        let oversized_len = (Message::MAX_MESSAGE_SIZE + 1) as u64;
        Write::write_all(&mut buffer, &oversized_len.to_be_bytes()).unwrap();
        
        buffer.set_position(0);
        let result = Message::receive(&mut buffer);
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_message_send_receive_async() {
        use tokio::io::duplex;
        
        let tx = create_test_transaction();
        let msg = Message::NewTransaction(tx.clone());
        
        let (mut client, mut server) = duplex(1024);
        
        // Send in background task
        let msg_clone = msg.clone();
        let send_task = tokio::spawn(async move {
            msg_clone.send_async(&mut client).await.unwrap();
        });
        
        // Receive
        let received = Message::receive_async(&mut server).await.unwrap();
        
        send_task.await.unwrap();
        
        if let Message::NewTransaction(received_tx) = received {
            assert_eq!(tx.hash(), received_tx.hash());
        } else {
            panic!("Received message type mismatch");
        }
    }

    #[tokio::test]
    async fn test_message_async_size_limit() {
        use tokio::io::duplex;
        use tokio::io::AsyncWriteExt;
        
        let (mut _client, mut server) = duplex(1024);
        
        // Manually write oversized length
        let oversized_len = (Message::MAX_MESSAGE_SIZE + 1) as u64;
        server.write_all(&oversized_len.to_be_bytes()).await.unwrap();
        
        // Reset to read position
        drop(server);
        let (mut _client, mut server) = duplex(1024);
        server.write_all(&oversized_len.to_be_bytes()).await.unwrap();
        
        drop(_client);
        let result = Message::receive_async(&mut server).await;
        assert!(result.is_err());
    }
}
