use btclib::crypto::PublicKey;

#[derive(Clone)]
pub struct LoadedRecipient {
    name: String,
    key: PublicKey,
}

impl LoadedRecipient {
    pub fn new(name: String, key: PublicKey) -> Self {
        Self { name, key }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn key(&self) -> &PublicKey {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use btclib::crypto::PrivateKey;

    #[test]
    fn test_loaded_recipient_creation() {
        let private = PrivateKey::default();
        let public = private.public_key();
        let name = "Alice".to_string();

        let recipient = LoadedRecipient::new(name.clone(), public.clone());

        assert_eq!(recipient.name(), &name);
        assert_eq!(recipient.key(), &public);
    }

    #[test]
    fn test_loaded_recipient_clone() {
        let private = PrivateKey::default();
        let public = private.public_key();
        let recipient = LoadedRecipient::new("Bob".to_string(), public);

        let cloned = recipient.clone();

        assert_eq!(recipient.name(), cloned.name());
        assert_eq!(recipient.key(), cloned.key());
    }

    #[test]
    fn test_loaded_recipient_different_names() {
        let private = PrivateKey::default();
        let public = private.public_key();

        let recipient1 = LoadedRecipient::new("Alice".to_string(), public.clone());
        let recipient2 = LoadedRecipient::new("Bob".to_string(), public);

        assert_ne!(recipient1.name(), recipient2.name());
        assert_eq!(recipient1.key(), recipient2.key());
    }

    #[test]
    fn test_loaded_recipient_empty_name() {
        let private = PrivateKey::default();
        let public = private.public_key();

        let recipient = LoadedRecipient::new("".to_string(), public);

        assert_eq!(recipient.name(), "");
    }

    #[test]
    fn test_loaded_recipient_long_name() {
        let private = PrivateKey::default();
        let public = private.public_key();
        let long_name = "A".repeat(1000);

        let recipient = LoadedRecipient::new(long_name.clone(), public);

        assert_eq!(recipient.name(), &long_name);
    }
}
