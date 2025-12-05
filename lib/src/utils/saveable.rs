use std::{
    fs::File,
    io::{Read, Result as IoResult, Write},
    path::Path,
};

pub trait Saveable
where
    Self: Sized,
{
    fn load<I: Read>(reader: I) -> IoResult<Self>;
    fn save<O: Write>(&self, writer: O) -> IoResult<()>;
    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> IoResult<()> {
        let file = File::create(&path)?;
        self.save(file)
    }
    fn load_from_file<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        let file = File::open(&path)?;
        Self::load(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        crypto::PrivateKey,
        types::{Transaction, TransactionOutput},
    };
    use std::fs;
    use uuid::Uuid;

    fn create_test_transaction(value: u64) -> Transaction {
        let private_key = PrivateKey::new();
        Transaction::new(
            vec![],
            vec![TransactionOutput {
                value,
                unique_id: Uuid::new_v4(),
                pubkey: private_key.public_key(),
            }],
        )
    }

    #[test]
    fn test_save_and_load_from_file() {
        let tx = create_test_transaction(1000);
        let temp_path = "test_transaction_saveable.cbor";

        // Save to file
        tx.save_to_file(temp_path).expect("Failed to save to file");

        // Load from file
        let loaded_tx = Transaction::load_from_file(temp_path)
            .expect("Failed to load from file");

        // Verify the data matches
        assert_eq!(tx.outputs.len(), loaded_tx.outputs.len());
        assert_eq!(tx.outputs[0].value, loaded_tx.outputs[0].value);

        // Cleanup
        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_save_and_load_from_memory() {
        let tx = create_test_transaction(2000);
        let mut buffer = Vec::new();

        // Save to memory buffer
        tx.save(&mut buffer).expect("Failed to save to buffer");

        // Load from memory buffer
        let loaded_tx = Transaction::load(buffer.as_slice())
            .expect("Failed to load from buffer");

        // Verify the data matches
        assert_eq!(tx.outputs.len(), loaded_tx.outputs.len());
        assert_eq!(tx.outputs[0].value, loaded_tx.outputs[0].value);
    }

    #[test]
    fn test_save_to_nonexistent_directory() {
        let tx = create_test_transaction(3000);
        
        // Try to save to a directory that doesn't exist
        let result = tx.save_to_file("nonexistent_dir/test.cbor");
        
        // Should fail because directory doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        // Try to load from a file that doesn't exist
        let result = Transaction::load_from_file("this_file_does_not_exist.cbor");
        
        // Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_save_load_cycles() {
        let tx = create_test_transaction(5000);
        let temp_path = "test_multiple_cycles.cbor";

        // First save and load
        tx.save_to_file(temp_path).expect("Failed to save");
        let loaded1 = Transaction::load_from_file(temp_path)
            .expect("Failed to load");

        // Second save and load (overwrite)
        loaded1.save_to_file(temp_path).expect("Failed to save again");
        let loaded2 = Transaction::load_from_file(temp_path)
            .expect("Failed to load again");

        // Verify consistency
        assert_eq!(tx.outputs[0].value, loaded2.outputs[0].value);

        // Cleanup
        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_save_to_empty_path_string() {
        let tx = create_test_transaction(1000);
        
        // Empty string should create a file (though it's not a valid practice)
        let result = tx.save_to_file("");
        
        // This will fail on most systems
        assert!(result.is_err());
    }
}
