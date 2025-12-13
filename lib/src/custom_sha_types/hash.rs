use std::vec;

use ciborium::ser::into_writer;
use serde::{Deserialize, Serialize};
use sha256::digest;

use crate::U256;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Hash(U256);

impl Hash {
    pub fn hash<T: serde::Serialize>(data: &T) -> Self {
        let mut serialized: Vec<u8> = vec![];

        if let Err(e) = into_writer(data, &mut serialized) {
            panic!("Failed to serialize data for hashing: {}", e);
        }

        let hash = digest(&serialized);
        let hash_bytes = hex::decode(hash).expect("Failed to decode hash hex string");
        let hash_array: [u8; 32] = hash_bytes
            .as_slice()
            .try_into()
            .expect("Hash length is not 32 bytes");

        Hash(U256::from_big_endian(&hash_array))
    }

    pub fn matches_target(&self, target: U256) -> bool {
        self.0 <= target
    }

    pub fn zero() -> Self {
        Hash(U256::zero())
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        self.0.to_big_endian()
    }
}
