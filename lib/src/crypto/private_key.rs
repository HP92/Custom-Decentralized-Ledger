use ecdsa::SigningKey;
use k256::Secp256k1;
use serde::{Deserialize, Serialize};

use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

use rand_core::OsRng; // Use rand_core's OsRng for compatibility

use crate::{crypto::PublicKey, utils::Saveable};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PrivateKey(#[serde(with = "signkey_serde")] SigningKey<Secp256k1>);

impl PrivateKey {
    pub fn public_key(&self) -> PublicKey {
        PublicKey::new(*self.0.verifying_key())
    }

    /// Returns a reference to the inner SigningKey.
    ///
    /// # Safety
    /// This method exposes the underlying signing key for cryptographic operations.
    /// The caller must ensure that the key is used appropriately and not leaked.
    pub fn as_signing_key(&self) -> &SigningKey<Secp256k1> {
        &self.0
    }
}

impl Default for PrivateKey {
    fn default() -> Self {
        PrivateKey(SigningKey::random(&mut OsRng))
    }
}

impl Saveable for PrivateKey {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to deserialize PrivateKey"))
    }
    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer).map_err(|_| {
            IoError::new(IoErrorKind::InvalidData, "Failed to serialize PrivateKey")
        })?;
        Ok(())
    }
}

mod signkey_serde {
    use serde::Deserialize;
    pub fn serialize<S>(
        key: &super::SigningKey<super::Secp256k1>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&key.to_bytes())
    }
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<super::SigningKey<super::Secp256k1>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::<u8>::deserialize(deserializer)?;
        Ok(super::SigningKey::from_slice(&bytes).unwrap())
    }
}
