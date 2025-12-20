use super::{FeeConfig, Key, Recipient};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    my_keys: Vec<Key>,
    contacts: Vec<Recipient>,
    default_node: String,
    fee_config: FeeConfig,
}

impl Config {
    pub fn new(
        my_keys: Vec<Key>,
        contacts: Vec<Recipient>,
        default_node: String,
        fee_config: FeeConfig,
    ) -> Self {
        Self {
            my_keys,
            contacts,
            default_node,
            fee_config,
        }
    }

    pub fn my_keys(&self) -> &Vec<Key> {
        &self.my_keys
    }

    pub fn contacts(&self) -> &Vec<Recipient> {
        &self.contacts
    }

    pub fn default_node(&self) -> &String {
        &self.default_node
    }

    pub fn fee_config(&self) -> &FeeConfig {
        &self.fee_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FeeType;
    use std::path::PathBuf;

    #[test]
    fn test_config_creation() {
        let keys = vec![Key::new(
            PathBuf::from("/path/to/public.key"),
            PathBuf::from("/path/to/private.key"),
        )];
        let contacts = vec![Recipient::new(
            "Alice".to_string(),
            PathBuf::from("/path/to/alice.key"),
        )];
        let default_node = "127.0.0.1:8333".to_string();
        let fee_config = FeeConfig::new(FeeType::Fixed, 100.0);

        let config = Config::new(keys, contacts, default_node.clone(), fee_config);

        assert_eq!(config.my_keys().len(), 1);
        assert_eq!(config.contacts().len(), 1);
        assert_eq!(config.default_node(), &default_node);
        assert_eq!(config.fee_config().value(), 100.0);
    }

    #[test]
    fn test_config_empty_keys_and_contacts() {
        let config = Config::new(
            vec![],
            vec![],
            "127.0.0.1:8333".to_string(),
            FeeConfig::new(FeeType::Percent, 2.5),
        );

        assert_eq!(config.my_keys().len(), 0);
        assert_eq!(config.contacts().len(), 0);
    }

    #[test]
    fn test_config_multiple_keys() {
        let keys = vec![
            Key::new(PathBuf::from("/key1/public"), PathBuf::from("/key1/private")),
            Key::new(PathBuf::from("/key2/public"), PathBuf::from("/key2/private")),
            Key::new(PathBuf::from("/key3/public"), PathBuf::from("/key3/private")),
        ];

        let config = Config::new(
            keys,
            vec![],
            "localhost:8333".to_string(),
            FeeConfig::new(FeeType::Fixed, 50.0),
        );

        assert_eq!(config.my_keys().len(), 3);
    }

    #[test]
    fn test_config_multiple_contacts() {
        let contacts = vec![
            Recipient::new("Alice".to_string(), PathBuf::from("/alice.key")),
            Recipient::new("Bob".to_string(), PathBuf::from("/bob.key")),
            Recipient::new("Carol".to_string(), PathBuf::from("/carol.key")),
        ];

        let config = Config::new(
            vec![],
            contacts,
            "127.0.0.1:9999".to_string(),
            FeeConfig::new(FeeType::Percent, 1.0),
        );

        assert_eq!(config.contacts().len(), 3);
        assert_eq!(config.contacts()[0].name(), "Alice");
        assert_eq!(config.contacts()[1].name(), "Bob");
        assert_eq!(config.contacts()[2].name(), "Carol");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::new(
            vec![Key::new(
                PathBuf::from("/pub.key"),
                PathBuf::from("/priv.key"),
            )],
            vec![Recipient::new(
                "Test".to_string(),
                PathBuf::from("/test.key"),
            )],
            "127.0.0.1:8333".to_string(),
            FeeConfig::new(FeeType::Fixed, 75.0),
        );

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.my_keys().len(), deserialized.my_keys().len());
        assert_eq!(config.contacts().len(), deserialized.contacts().len());
        assert_eq!(config.default_node(), deserialized.default_node());
        assert_eq!(config.fee_config().value(), deserialized.fee_config().value());
    }

    #[test]
    fn test_config_clone() {
        let config = Config::new(
            vec![Key::new(PathBuf::from("/pub"), PathBuf::from("/priv"))],
            vec![],
            "node.example.com:8333".to_string(),
            FeeConfig::new(FeeType::Percent, 3.0),
        );

        let cloned = config.clone();

        assert_eq!(config.my_keys().len(), cloned.my_keys().len());
        assert_eq!(config.contacts().len(), cloned.contacts().len());
        assert_eq!(config.default_node(), cloned.default_node());
    }
}
