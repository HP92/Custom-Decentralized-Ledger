use ecdsa::VerifyingKey;
use k256::Secp256k1;

pub struct PublicKey(VerifyingKey<Secp256k1>);