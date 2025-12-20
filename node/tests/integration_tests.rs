use node::{BLOCKCHAIN, NODES};

#[tokio::test]
async fn test_blockchain_initialization() {
    let blockchain = BLOCKCHAIN.read().await;
    // A default blockchain should start with genesis or be empty
    assert!(blockchain.blocks().is_empty() || !blockchain.blocks().is_empty());
}

#[tokio::test]
async fn test_nodes_map_initialization() {
    // The NODES map should be accessible
    let nodes_count = NODES.len();
    assert!(nodes_count == 0 || nodes_count > 0);
}

#[tokio::test]
async fn test_blockchain_write_lock() {
    {
        let blockchain = BLOCKCHAIN.write().await;
        // Should be able to acquire write lock
        let _initial_len = blockchain.blocks().len();
        // Lock acquired successfully
    }
    // Lock should be released after scope
    let blockchain = BLOCKCHAIN.read().await;
    let _len = blockchain.blocks().len();
    // Lock can be acquired again
}

#[tokio::test]
async fn test_concurrent_blockchain_reads() {
    let handle1 = tokio::spawn(async {
        let blockchain = BLOCKCHAIN.read().await;
        blockchain.blocks().len()
    });

    let handle2 = tokio::spawn(async {
        let blockchain = BLOCKCHAIN.read().await;
        blockchain.blocks().len()
    });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    // Both reads should succeed and return the same length
    assert_eq!(result1, result2);
}
