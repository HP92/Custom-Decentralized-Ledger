use std::sync::Arc;

use btclib::{crypto::PublicKey, types::TransactionOutput};
use crossbeam_skiplist::SkipMap;

use crate::models::LoadedKey;

#[derive(Clone)]
pub struct UtxoStore {
    my_keys: Vec<LoadedKey>,
    utxos: Arc<SkipMap<PublicKey, Vec<(bool, TransactionOutput)>>>,
}

impl UtxoStore {
    pub fn my_keys(&self) -> &Vec<LoadedKey> {
        &self.my_keys
    }

    pub fn utxos(&self) -> Arc<SkipMap<PublicKey, Vec<(bool, TransactionOutput)>>> {
        Arc::clone(&self.utxos)
    }

    pub fn add_key(&mut self, key: LoadedKey) {
        self.my_keys.push(key);
    }
}

impl Default for UtxoStore {
    fn default() -> Self {
        Self {
            my_keys: Vec::new(),
            utxos: Arc::new(SkipMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use btclib::crypto::PrivateKey;

    #[test]
    fn test_utxo_store_default() {
        let store = UtxoStore::default();
        assert_eq!(store.my_keys().len(), 0);
        assert_eq!(store.utxos().len(), 0);
    }

    #[test]
    fn test_utxo_store_add_key() {
        let mut store = UtxoStore::default();
        
        let private = PrivateKey::default();
        let public = private.public_key();
        let key = LoadedKey::new(public, private);
        
        store.add_key(key);
        assert_eq!(store.my_keys().len(), 1);
    }

    #[test]
    fn test_utxo_store_add_multiple_keys() {
        let mut store = UtxoStore::default();
        
        for _ in 0..5 {
            let private = PrivateKey::default();
            let public = private.public_key();
            let key = LoadedKey::new(public, private);
            store.add_key(key);
        }
        
        assert_eq!(store.my_keys().len(), 5);
    }

    #[test]
    fn test_utxo_store_clone() {
        let mut store = UtxoStore::default();
        
        let private = PrivateKey::default();
        let public = private.public_key();
        let key = LoadedKey::new(public, private);
        store.add_key(key);
        
        let cloned = store.clone();
        assert_eq!(cloned.my_keys().len(), store.my_keys().len());
    }

    #[test]
    fn test_utxo_store_add_utxo() {
        let mut store = UtxoStore::default();
        
        let private = PrivateKey::default();
        let public = private.public_key();
        let key = LoadedKey::new(public.clone(), private);
        store.add_key(key);
        
        let utxo = TransactionOutput::new(100, uuid::Uuid::new_v4(), public.clone());
        store.utxos().insert(public, vec![(false, utxo)]);
        
        assert_eq!(store.utxos().len(), 1);
    }

    #[test]
    fn test_utxo_store_multiple_utxos_per_key() {
        let store = UtxoStore::default();
        
        let private = PrivateKey::default();
        let public = private.public_key();
        
        let utxo1 = TransactionOutput::new(100, uuid::Uuid::new_v4(), public.clone());
        let utxo2 = TransactionOutput::new(200, uuid::Uuid::new_v4(), public.clone());
        let utxo3 = TransactionOutput::new(300, uuid::Uuid::new_v4(), public.clone());
        
        store.utxos().insert(public, vec![
            (false, utxo1),
            (false, utxo2),
            (true, utxo3), // marked as used
        ]);
        
        assert_eq!(store.utxos().len(), 1);
        let utxos_ref = store.utxos();
        let entry = utxos_ref.iter().next().unwrap();
        assert_eq!(entry.value().len(), 3);
    }
}
