use ecdsa::Signature as ECDSASignature;
use k256::Secp256k1;

pub struct Signature(ECDSASignature<Secp256k1>);