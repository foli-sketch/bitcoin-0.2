use serde::{Serialize, Deserialize};
use crate::transaction::Transaction;
use crate::crypto::sha256;

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    pub height: u64,
    pub timestamp: i64,
    pub prev_hash: Vec<u8>,
    pub nonce: u64,
    pub target: [u8; 32],
    pub merkle_root: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub hash: Vec<u8>,
}

impl Block {
    pub fn hash_header(&self) -> Vec<u8> {
        sha256(&sha256(
            &bincode::serialize(&self.header)
                .expect("block header serialization"),
        ))
    }

    pub fn verify_pow(&self) -> bool {
        self.hash == self.hash_header()
            && crate::pow::valid_pow(
                &self.hash,
                &self.header.target,
            )
    }
}
