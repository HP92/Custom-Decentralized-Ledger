use ecdsa::{Signature as ECDSASignature, signature};
use k256::Secp256k1;
use serde::{Deserialize, Serialize};
use signature::{Signer, Verifier};

use crate::{
    crypto::{PrivateKey, PublicKey},
    custom_sha_types::Hash,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Signature(ECDSASignature<Secp256k1>);

impl Signature {
    pub fn sign_output(output_hash: &Hash, private_key: &PrivateKey) -> Self {
        let signing_key = private_key.as_signing_key();
        let signature = signing_key.sign(&output_hash.as_bytes());
        Signature(signature)
    }

    pub fn verify(&self, output_hash: &Hash, public_key: &PublicKey) -> bool {
        public_key
            .as_verifying_key()
            .verify(&output_hash.as_bytes(), &self.0)
            .is_ok()
    }
}
