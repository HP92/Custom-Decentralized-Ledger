use ecdsa::VerifyingKey;
use k256::Secp256k1;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicKey(VerifyingKey<Secp256k1>);

impl PublicKey {
    pub fn new(key: VerifyingKey<Secp256k1>) -> Self {
        PublicKey(key)
    }

    /// Returns a reference to the inner VerifyingKey.  
    pub fn as_verifying_key(&self) -> &VerifyingKey<Secp256k1> {
        &self.0
    }
}
