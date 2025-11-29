use ecdsa::Signature as ECDSASignature;
use k256::Secp256k1;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Signature(ECDSASignature<Secp256k1>);
