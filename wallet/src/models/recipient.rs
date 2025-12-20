use super::LoadedRecipient;
use anyhow::Result;
use btclib::{crypto::PublicKey, utils::Saveable};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Recipient {
    name: String,
    key: PathBuf,
}

impl Recipient {
    pub fn new(name: String, key: PathBuf) -> Self {
        Self { name, key }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn key_path(&self) -> &PathBuf {
        &self.key
    }

    pub fn load(&self) -> Result<LoadedRecipient> {
        let key = PublicKey::load_from_file(&self.key)?;
        Ok(LoadedRecipient::new(self.name.clone(), key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipient_creation() {
        let name = "Alice".to_string();
        let key_path = PathBuf::from("/path/to/alice.key");

        let recipient = Recipient::new(name.clone(), key_path.clone());

        assert_eq!(recipient.name(), &name);
        assert_eq!(recipient.key_path(), &key_path);
    }

    #[test]
    fn test_recipient_serialization() {
        let recipient = Recipient::new(
            "Bob".to_string(),
            PathBuf::from("/path/to/bob.key"),
        );

        let serialized = serde_json::to_string(&recipient).unwrap();
        let deserialized: Recipient = serde_json::from_str(&serialized).unwrap();

        assert_eq!(recipient.name(), deserialized.name());
        assert_eq!(recipient.key_path(), deserialized.key_path());
    }

    #[test]
    fn test_recipient_clone() {
        let recipient = Recipient::new(
            "Carol".to_string(),
            PathBuf::from("/path/to/carol.key"),
        );

        let cloned = recipient.clone();

        assert_eq!(recipient.name(), cloned.name());
        assert_eq!(recipient.key_path(), cloned.key_path());
    }

    #[test]
    fn test_recipient_multiple_instances() {
        let recipient1 = Recipient::new(
            "Alice".to_string(),
            PathBuf::from("/alice.key"),
        );
        let recipient2 = Recipient::new(
            "Bob".to_string(),
            PathBuf::from("/bob.key"),
        );

        assert_ne!(recipient1.name(), recipient2.name());
        assert_ne!(recipient1.key_path(), recipient2.key_path());
    }

    #[test]
    fn test_recipient_empty_name() {
        let recipient = Recipient::new(
            "".to_string(),
            PathBuf::from("/path/to/key"),
        );

        assert_eq!(recipient.name(), "");
    }

    #[test]
    fn test_recipient_special_characters_in_name() {
        let name = "Alice O'Brien (alice@example.com)".to_string();
        let recipient = Recipient::new(name.clone(), PathBuf::from("/key"));

        assert_eq!(recipient.name(), &name);
    }
}
