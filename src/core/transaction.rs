use serde::{Serialize, Deserialize};
use crate::crypto::sha256;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxInput {
    pub txid: Vec<u8>,
    pub index: u32,
    pub pubkey: Vec<u8>,       // compressed pubkey bytes
    pub signature: Vec<u8>,    // compact 64-byte sig
    pub address_index: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: u64,
    pub pubkey_hash: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

impl Transaction {
    pub fn txid(&self) -> Vec<u8> {
        sha256(&bincode::serialize(self).unwrap())
    }

    /// Message signed by each input
    pub fn sighash(&self) -> Vec<u8> {
        sha256(&bincode::serialize(self).unwrap())
    }

    pub fn serialized_size(&self) -> usize {
        self.inputs.len() * 148 + self.outputs.len() * 34 + 10
    }
}
