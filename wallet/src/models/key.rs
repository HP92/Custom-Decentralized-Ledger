use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Key {
    public: PathBuf,
    private: PathBuf,
}

impl Key {
    pub fn new(public: PathBuf, private: PathBuf) -> Self {
        Self { public, private }
    }

    pub fn public_path(&self) -> &PathBuf {
        &self.public
    }

    pub fn private_path(&self) -> &PathBuf {
        &self.private
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_creation() {
        let public_path = PathBuf::from("/path/to/public.key");
        let private_path = PathBuf::from("/path/to/private.key");
        
        let key = Key::new(public_path.clone(), private_path.clone());
        
        assert_eq!(key.public_path(), &public_path);
        assert_eq!(key.private_path(), &private_path);
    }

    #[test]
    fn test_key_serialization() {
        let public_path = PathBuf::from("/path/to/public.key");
        let private_path = PathBuf::from("/path/to/private.key");
        let key = Key::new(public_path, private_path);
        
        let serialized = serde_json::to_string(&key).unwrap();
        let deserialized: Key = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(key.public_path(), deserialized.public_path());
        assert_eq!(key.private_path(), deserialized.private_path());
    }

    #[test]
    fn test_key_clone() {
        let key = Key::new(
            PathBuf::from("/path/to/public.key"),
            PathBuf::from("/path/to/private.key"),
        );
        
        let cloned = key.clone();
        
        assert_eq!(key.public_path(), cloned.public_path());
        assert_eq!(key.private_path(), cloned.private_path());
    }
}
