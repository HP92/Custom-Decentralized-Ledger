use ecdsa::VerifyingKey;
use k256::Secp256k1;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicKey(pub VerifyingKey<Secp256k1>);
