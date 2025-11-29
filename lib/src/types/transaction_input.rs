use serde::{Deserialize, Serialize};

use crate::{crypto::Signature, custom_sha_types::Hash};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionInput {
    pub prev_transaction_output_hash: Hash,
    pub signature: Signature,
}
