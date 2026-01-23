use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::transaction::Transaction;
use crate::crypto::sha256;
use crate::policy::{MAX_BLOCK_TX_BYTES, MAX_BLOCK_TXS};

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    pub height: u64,
    pub timestamp: i64,
    pub prev_hash: Vec<u8>,
    pub nonce: u64,
    pub difficulty: u32,
    pub merkle_root: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub hash: Vec<u8>,
}

impl Block {
    /// Construct a block candidate (NO mining here)
    pub fn new(
        prev_hash: Vec<u8>,
        transactions: Vec<Transaction>,
        difficulty: u32,
        height: u64,
    ) -> Block {
        let mut selected = Vec::new();
        let mut total_bytes = 0usize;

        for tx in transactions {
            let tx_size = tx.serialized_size();

            if selected.len() >= MAX_BLOCK_TXS {
                break;
            }

            if total_bytes + tx_size > MAX_BLOCK_TX_BYTES {
                break;
            }

            total_bytes += tx_size;
            selected.push(tx);
        }

        let mut block = Block {
            header: BlockHeader {
                height,
                timestamp: now(),
                prev_hash,
                nonce: 0,
                difficulty,
                merkle_root: vec![],
            },
            transactions: selected,
            hash: vec![],
        };

        block.header.merkle_root = block.calculate_merkle_root();
        block
    }

    /// Double-SHA256 header hash
    pub fn hash_header(&self) -> Vec<u8> {
        sha256(&sha256(
            &bincode::serialize(&self.header).expect("block header serialization"),
        ))
    }

    /// Consensus PoW verification (single source of truth)
    pub fn verify_pow(&self) -> bool {
        self.hash == self.hash_header()
            && crate::pow::valid_pow(&self.hash, self.header.difficulty)
    }

    /// Deterministic Merkle root
    pub fn calculate_merkle_root(&self) -> Vec<u8> {
        if self.transactions.is_empty() {
            return vec![0u8; 32];
        }

        let mut hashes: Vec<Vec<u8>> =
            self.transactions.iter().map(|tx| tx.txid()).collect();

        while hashes.len() > 1 {
            if hashes.len() % 2 != 0 {
                hashes.push(hashes.last().unwrap().clone());
            }

            hashes = hashes
                .chunks(2)
                .map(|pair| {
                    sha256(&[pair[0].as_slice(), pair[1].as_slice()].concat())
                })
                .collect();
        }

        hashes[0].clone()
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_secs() as i64
}
