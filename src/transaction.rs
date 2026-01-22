use serde::{Serialize, Deserialize};
use crate::crypto::sha256;

#[derive(Serialize, Deserialize, Clone)]
pub struct TxInput {
    pub txid: Vec<u8>,
    pub index: usize,
    pub signature: Vec<u8>,
    pub pubkey: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TxOutput {
    pub value: u64,
    pub pubkey_hash: Vec<u8>, // sha256(pubkey)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

impl Transaction {
    pub fn txid(&self) -> Vec<u8> {
        sha256(&sha256(&bincode::serialize(self).unwrap()))
    }

    pub fn sighash(&self) -> Vec<u8> {
        let mut stripped = self.clone();
        for input in &mut stripped.inputs {
            input.signature.clear();
        }
        sha256(&bincode::serialize(&stripped).unwrap())
    }
}
