use crate::models::{Config, FeeType, LoadedKey, UtxoStore};
use anyhow::Result;
use btclib::{
    crypto::{PrivateKey, PublicKey, Signature},
    network::Message,
    types::{Transaction, TransactionInput, TransactionOutput},
    utils::Saveable,
};
use kanal::AsyncSender;
use std::{fs, path::PathBuf};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct Core {
    config: Config,
    utxos: UtxoStore,
    tx_sender: AsyncSender<Transaction>,
}
impl Core {
    fn new(config: Config, utxos: UtxoStore) -> Self {
        let (tx_sender, _) = kanal::bounded(100);

        Self {
            config,
            utxos,
            tx_sender: tx_sender.clone_async(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn utxos(&self) -> &UtxoStore {
        &self.utxos
    }

    pub fn tx_sender(&self) -> &AsyncSender<Transaction> {
        &self.tx_sender
    }

    pub fn load(config_path: PathBuf) -> Result<Self> {
        let config: Config = toml::from_str(&fs::read_to_string(&config_path)?)?;
        let mut utxos = UtxoStore::default();
        // Load keys from config
        for key in config.my_keys() {
            let public = PublicKey::load_from_file(key.public_path())?;
            let private = PrivateKey::load_from_file(key.private_path())?;
            utxos.add_key(LoadedKey::new(public, private));
        }
        Ok(Core::new(config, utxos))
    }
    pub async fn fetch_utxos(&self) -> Result<()> {
        let mut stream = TcpStream::connect(self.config().default_node()).await?;
        for key in self.utxos().my_keys() {
            let message = Message::FetchUTXOs(key.public().clone());
            message.send_async(&mut stream).await?;
            if let Message::UTXOs(utxos) = Message::receive_async(&mut stream).await? {
                // Replace the entire UTXO set for this key
                self.utxos.utxos().insert(
                    key.public().clone(),
                    utxos
                        .into_iter()
                        .map(|(output, marked)| (marked, output))
                        .collect(),
                );
            } else {
                return Err(anyhow::anyhow!("Unexpected response from node"));
            }
        }
        Ok(())
    }

    pub async fn send_transaction(&self, transaction: Transaction) -> Result<()> {
        let mut stream = TcpStream::connect(self.config().default_node()).await?;
        let message = Message::SubmitTransaction(transaction);
        message.send_async(&mut stream).await?;
        Ok(())
    }

    pub fn get_balance(&self) -> u64 {
        let mut total = 0;
        for entry in self.utxos().utxos().iter() {
            for utxo in entry.value().iter() {
                total += utxo.1.value();
            }
        }
        total
    }

    pub async fn create_transaction(
        &self,
        recipient: &PublicKey,
        amount: u64,
    ) -> Result<Transaction> {
        let fee = self.calculate_fee(amount);
        let total_amount = amount + fee;
        let mut inputs = Vec::new();
        let mut input_sum = 0;
        for entry in self.utxos.utxos().iter() {
            let pubkey = entry.key();
            let utxos = entry.value();
            for (marked, utxo) in utxos.iter() {
                if *marked {
                    continue; // Skip marked UTXOs
                }
                if input_sum >= total_amount {
                    break;
                }
                let signature = Signature::sign_output(
                    &utxo.hash(),
                    self.utxos()
                        .my_keys()
                        .iter()
                        .find(|k| k.public() == pubkey)
                        .unwrap()
                        .private(),
                );

                let input = TransactionInput::new(utxo.hash(), signature);

                inputs.push(input);
                input_sum += utxo.value();
            }
            if input_sum >= total_amount {
                break;
            }
        }
        if input_sum < total_amount {
            return Err(anyhow::anyhow!("Insufficient funds"));
        }
        let mut outputs = vec![TransactionOutput::new(
            amount,
            uuid::Uuid::new_v4(),
            recipient.clone(),
        )];
        if input_sum > total_amount {
            outputs.push(TransactionOutput::new(
                input_sum - total_amount,
                uuid::Uuid::new_v4(),
                self.utxos().my_keys()[0].public().clone(),
            ));
        }
        Ok(Transaction::new(inputs, outputs))
    }

    fn calculate_fee(&self, amount: u64) -> u64 {
        match self.config.fee_config().fee_type() {
            FeeType::Fixed => self.config.fee_config().value() as u64,
            FeeType::Percent => (amount as f64 * self.config.fee_config().value() / 100.0) as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FeeConfig, FeeType, Key};
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir) -> Config {
        // Create test keys
        let private = PrivateKey::default();
        let public = private.public_key();
        
        let pub_path = temp_dir.path().join("test.pub");
        let priv_path = temp_dir.path().join("test.priv");
        
        public.save_to_file(&pub_path).unwrap();
        private.save_to_file(&priv_path).unwrap();
        
        Config::new(
            vec![Key::new(pub_path, priv_path)],
            vec![],
            "127.0.0.1:8333".to_string(),
            FeeConfig::new(FeeType::Fixed, 10.0),
        )
    }

    #[test]
    fn test_core_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let utxos = UtxoStore::default();
        
        let core = Core::new(config.clone(), utxos);
        
        assert_eq!(core.config().default_node(), config.default_node());
        assert_eq!(core.utxos().my_keys().len(), 0);
    }

    #[test]
    fn test_core_load_from_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        
        let config_path = temp_dir.path().join("config.toml");
        let config_str = toml::to_string(&config).unwrap();
        std::fs::write(&config_path, config_str).unwrap();
        
        let core = Core::load(config_path).unwrap();
        
        assert_eq!(core.config().my_keys().len(), 1);
        assert_eq!(core.utxos().my_keys().len(), 1);
    }

    #[test]
    fn test_core_get_balance_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let utxos = UtxoStore::default();
        
        let core = Core::new(config, utxos);
        
        assert_eq!(core.get_balance(), 0);
    }

    #[test]
    fn test_core_get_balance_with_utxos() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut utxos = UtxoStore::default();
        
        let private = PrivateKey::default();
        let public = private.public_key();
        let key = LoadedKey::new(public.clone(), private);
        utxos.add_key(key);
        
        // Add some UTXOs
        let utxo1 = TransactionOutput::new(100, uuid::Uuid::new_v4(), public.clone());
        let utxo2 = TransactionOutput::new(200, uuid::Uuid::new_v4(), public.clone());
        let utxo3 = TransactionOutput::new(300, uuid::Uuid::new_v4(), public.clone());
        
        utxos.utxos().insert(public, vec![
            (false, utxo1),
            (false, utxo2),
            (false, utxo3),
        ]);
        
        let core = Core::new(config, utxos);
        
        assert_eq!(core.get_balance(), 600);
    }

    #[test]
    fn test_core_calculate_fee_fixed() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = create_test_config(&temp_dir);
        config = Config::new(
            config.my_keys().clone(),
            config.contacts().clone(),
            config.default_node().clone(),
            FeeConfig::new(FeeType::Fixed, 50.0),
        );
        
        let utxos = UtxoStore::default();
        let core = Core::new(config, utxos);
        
        assert_eq!(core.calculate_fee(100), 50);
        assert_eq!(core.calculate_fee(1000), 50);
    }

    #[test]
    fn test_core_calculate_fee_percent() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = create_test_config(&temp_dir);
        config = Config::new(
            config.my_keys().clone(),
            config.contacts().clone(),
            config.default_node().clone(),
            FeeConfig::new(FeeType::Percent, 2.5),
        );
        
        let utxos = UtxoStore::default();
        let core = Core::new(config, utxos);
        
        assert_eq!(core.calculate_fee(100), 2);  // 2.5% of 100 = 2.5 -> 2
        assert_eq!(core.calculate_fee(1000), 25); // 2.5% of 1000 = 25
    }

    #[tokio::test]
    async fn test_core_create_transaction_insufficient_funds() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut utxos = UtxoStore::default();
        
        let private = PrivateKey::default();
        let public = private.public_key();
        let key = LoadedKey::new(public.clone(), private);
        utxos.add_key(key);
        
        // Add a small UTXO
        let utxo = TransactionOutput::new(50, uuid::Uuid::new_v4(), public.clone());
        utxos.utxos().insert(public.clone(), vec![(false, utxo)]);
        
        let core = Core::new(config, utxos);
        
        let recipient_private = PrivateKey::default();
        let recipient = recipient_private.public_key();
        
        // Try to send more than available (50 available, but need 100 + fee)
        let result = core.create_transaction(&recipient, 100).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Insufficient funds"));
    }

    #[tokio::test]
    async fn test_core_create_transaction_success() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut utxos = UtxoStore::default();
        
        let private = PrivateKey::default();
        let public = private.public_key();
        let key = LoadedKey::new(public.clone(), private);
        utxos.add_key(key);
        
        // Add sufficient UTXOs - only the first one sufficient for our transaction
        let utxo1 = TransactionOutput::new(100, uuid::Uuid::new_v4(), public.clone());
        let utxo2 = TransactionOutput::new(200, uuid::Uuid::new_v4(), public.clone());
        utxos.utxos().insert(public.clone(), vec![
            (false, utxo1),
            (false, utxo2),
        ]);
        
        let core = Core::new(config, utxos);
        
        let recipient_private = PrivateKey::default();
        let recipient = recipient_private.public_key();
        
        // Send 50 (fee is 10, so total 60)
        // Since we have utxo1 (100), it's sufficient. Change should be 100 - 60 = 40
        let transaction = core.create_transaction(&recipient, 50).await.unwrap();
        
        assert_eq!(transaction.outputs().len(), 2); // Payment + change
        assert_eq!(transaction.outputs()[0].value(), 50); // Payment to recipient
        // Change should be 100 - 60 = 40 (only first UTXO is used)
        assert_eq!(transaction.outputs()[1].value(), 40);
    }

    #[tokio::test]
    async fn test_core_create_transaction_exact_amount() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut utxos = UtxoStore::default();
        
        let private = PrivateKey::default();
        let public = private.public_key();
        let key = LoadedKey::new(public.clone(), private);
        utxos.add_key(key);
        
        // Add UTXO that matches exactly amount + fee
        let utxo = TransactionOutput::new(110, uuid::Uuid::new_v4(), public.clone());
        utxos.utxos().insert(public.clone(), vec![(false, utxo)]);
        
        let core = Core::new(config, utxos);
        
        let recipient_private = PrivateKey::default();
        let recipient = recipient_private.public_key();
        
        // Send 100 (fee is 10, so total 110 - exact match)
        let transaction = core.create_transaction(&recipient, 100).await.unwrap();
        
        assert_eq!(transaction.outputs().len(), 1); // No change needed
        assert_eq!(transaction.outputs()[0].value(), 100);
    }

    #[tokio::test]
    async fn test_core_create_transaction_skips_marked_utxos() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut utxos = UtxoStore::default();
        
        let private = PrivateKey::default();
        let public = private.public_key();
        let key = LoadedKey::new(public.clone(), private);
        utxos.add_key(key);
        
        // Add UTXOs, some marked as used
        let utxo1 = TransactionOutput::new(100, uuid::Uuid::new_v4(), public.clone());
        let utxo2 = TransactionOutput::new(200, uuid::Uuid::new_v4(), public.clone());
        let utxo3 = TransactionOutput::new(300, uuid::Uuid::new_v4(), public.clone());
        utxos.utxos().insert(public.clone(), vec![
            (true, utxo1),  // Marked as used
            (false, utxo2), // Available
            (true, utxo3),  // Marked as used
        ]);
        
        let core = Core::new(config, utxos);
        
        let recipient_private = PrivateKey::default();
        let recipient = recipient_private.public_key();
        
        // Try to send 150 - should only use utxo2 (200)
        let transaction = core.create_transaction(&recipient, 150).await.unwrap();
        
        assert_eq!(transaction.inputs().len(), 1); // Only one UTXO used
        assert_eq!(transaction.outputs().len(), 2); // Payment + change
        assert_eq!(transaction.outputs()[0].value(), 150);
        assert_eq!(transaction.outputs()[1].value(), 40); // 200 - 150 - 10 fee
    }

    #[test]
    fn test_core_clone() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let utxos = UtxoStore::default();
        
        let core = Core::new(config, utxos);
        let cloned = core.clone();
        
        assert_eq!(core.config().default_node(), cloned.config().default_node());
        assert_eq!(core.utxos().my_keys().len(), cloned.utxos().my_keys().len());
    }

    #[test]
    fn test_core_tx_sender_accessor() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let utxos = UtxoStore::default();
        
        let core = Core::new(config, utxos);
        
        // Just verify we can access the sender
        let _sender = core.tx_sender();
    }
}
