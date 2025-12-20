use ecdsa::VerifyingKey;
use k256::Secp256k1;
use serde::{Deserialize, Serialize};
use spki::EncodePublicKey;

use std::cmp::Ordering;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

use crate::utils::Saveable;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicKey(VerifyingKey<Secp256k1>);

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_point = self.0.to_encoded_point(true);
        let other_point = other.0.to_encoded_point(true);
        self_point.as_bytes().cmp(other_point.as_bytes())
    }
}

impl PublicKey {
    pub fn new(key: VerifyingKey<Secp256k1>) -> Self {
        PublicKey(key)
    }

    /// Returns a reference to the inner VerifyingKey.  
    pub fn as_verifying_key(&self) -> &VerifyingKey<Secp256k1> {
        &self.0
    }
}

impl Saveable for PublicKey {
    fn load<I: Read>(mut reader: I) -> IoResult<Self> {
        // read PEM-encoded public key into string
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        // decode the public key from PEM
        let public_key = buf
            .parse()
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to parse PublicKey"))?;
        Ok(PublicKey(public_key))
    }
    fn save<O: Write>(&self, mut writer: O) -> IoResult<()> {
        let s = self
            .0
            .to_public_key_pem(Default::default())
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to serialize PublicKey"))?;
        writer.write_all(s.as_bytes())?;
        Ok(())
    }
}
