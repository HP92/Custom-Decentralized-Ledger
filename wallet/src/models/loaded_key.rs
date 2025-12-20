use btclib::crypto::{PrivateKey, PublicKey};

#[derive(Clone)]
pub struct LoadedKey {
    public: PublicKey,
    private: PrivateKey,
}

impl LoadedKey {
    pub fn new(public: PublicKey, private: PrivateKey) -> Self {
        Self { public, private }
    }

    pub fn public(&self) -> &PublicKey {
        &self.public
    }

    pub fn private(&self) -> &PrivateKey {
        &self.private
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loaded_key_creation() {
        let private = PrivateKey::default();
        let public = private.public_key();
        
        let loaded_key = LoadedKey::new(public.clone(), private.clone());
        
        assert_eq!(loaded_key.public(), &public);
    }

    #[test]
    fn test_loaded_key_clone() {
        let private = PrivateKey::default();
        let public = private.public_key();
        let loaded_key = LoadedKey::new(public, private);
        
        let cloned = loaded_key.clone();
        
        assert_eq!(loaded_key.public(), cloned.public());
    }

    #[test]
    fn test_loaded_key_public_private_match() {
        let private = PrivateKey::default();
        let public = private.public_key();
        let loaded_key = LoadedKey::new(public.clone(), private);
        
        // Verify the public key matches the private key
        let derived_public = loaded_key.private().public_key();
        assert_eq!(loaded_key.public(), &derived_public);
    }

    #[test]
    fn test_loaded_key_multiple_instances() {
        let private1 = PrivateKey::default();
        let public1 = private1.public_key();
        let key1 = LoadedKey::new(public1, private1);
        
        let private2 = PrivateKey::default();
        let public2 = private2.public_key();
        let key2 = LoadedKey::new(public2, private2);
        
        // Different keys should have different public keys
        assert_ne!(key1.public(), key2.public());
    }
}
