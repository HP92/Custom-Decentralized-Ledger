use ecdsa::SigningKey;
use k256::Secp256k1;

pub struct PrivateKey(SigningKey<Secp256k1>);