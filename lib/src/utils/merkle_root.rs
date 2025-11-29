use serde::{Deserialize, Serialize};

use crate::{custom_sha_types::Hash, types::Transaction};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MerkleRoot(Hash);

impl MerkleRoot {
    pub fn calculate(transactions: &[Transaction]) -> Self {
        let mut layer: Vec<Hash> = vec![];
        for transaction in transactions {
            layer.push(Hash::hash(transaction));
        }

        while layer.len() > 1 {
            let mut next_layer: Vec<Hash> = vec![];
            for pair in layer.chunks(2) {
                let left = pair[0];
                let right = pair.get(1).unwrap_or(&pair[0]);
                next_layer.push(Hash::hash(&[left, *right]));
            }
            layer = next_layer;
        }

        MerkleRoot(layer[0])
    }
}
