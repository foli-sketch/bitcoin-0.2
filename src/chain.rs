use std::collections::HashMap;
use std::fs;
use std::env;
use std::path::PathBuf;

use time::OffsetDateTime;

use crate::consensus::{
    fork_choice::cumulative_work,
    difficulty::calculate_next_difficulty,
};

use crate::{
    block::{Block, BlockHeader},
    pow::mine,
    utxo::{UTXOSet, UTXO},
    transaction::{Transaction, TxOutput},
    reward::block_reward,
    revelation::revelation_tx,
    merkle::merkle_root,
    validation::validate_transaction,
};

/// ─────────────────────────────────────────────
/// Consensus parameters
/// ─────────────────────────────────────────────

/// Coinbase outputs may only be spent after this many blocks
#[allow(dead_code)] // Enforced starting at CONSENSUS_V2_HEIGHT
const COINBASE_MATURITY: u64 = 100;

/// Height at which consensus v2 activates
/// (coinbase maturity becomes strictly enforced)
const CONSENSUS_V2_HEIGHT: u64 = 1000;

/* -------------------- BLOCKCHAIN -------------------- */

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub utxos: UTXOSet,
}

/* -------------------- DATA FILES -------------------- */

fn data_dir() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.push("data");
    path
}

fn blocks_file() -> PathBuf {
    let mut path = data_dir();
    path.push("blocks.json");
    path
}

fn utxos_file() -> PathBuf {
    let mut path = data_dir();
    path.push("utxos.json");
    path
}

/* -------------------- IMPLEMENTATION -------------------- */

impl Blockchain {
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            utxos: HashMap::new(),
        }
    }

    pub fn height(&self) -> u64 {
        self.blocks.len() as u64
    }

    /* ---------- BLOCK VALIDATION / ADD ---------- */

    pub fn validate_and_add_block(&mut self, block: Block) -> bool {
        let expected_height = self.height();

        // ── Height + linkage ──────────────────────
        if expected_height == 0 {
            if block.header.height != 0 {
                return false;
            }
        } else {
            let prev = self.blocks.last().unwrap();

            if block.header.height != expected_height {
                return false;
            }

            if block.header.prev_hash != prev.hash {
                return false;
            }

            // ⏱ Timestamp sanity (anti time-warp)
            if block.header.timestamp <= prev.header.timestamp {
                return false;
            }
        }

        // ⛏ Enforce expected difficulty
        let expected_difficulty =
            calculate_next_difficulty(&self.blocks);

        if block.header.difficulty != expected_difficulty {
            return false;
        }

        // ⛏ Proof-of-Work
        if !block.verify_pow() {
            return false;
        }

        // 🌳 Merkle root
        if merkle_root(&block.transactions) != block.header.merkle_root {
            return false;
        }

        // 💰 Coinbase rules
        if let Some(cb) = block.transactions.first() {
            if !cb.inputs.is_empty() {
                return false;
            }

            let expected_reward =
                block_reward(block.header.height);

            let actual_reward: u64 =
                cb.outputs.iter().map(|o| o.value).sum();

            if actual_reward > expected_reward {
                return false;
            }
        }

        // 🔐 Transaction validation + consensus v2 rules
        for tx in &block.transactions {
            // Base consensus (signatures, ownership, inflation)
            if !validate_transaction(tx, &self.utxos) {
                return false;
            }

            // ── Consensus v2: coinbase maturity enforcement ──
            if block.header.height >= CONSENSUS_V2_HEIGHT {
                if !self.enforce_coinbase_maturity(tx, block.header.height) {
                    return false;
                }
            }
        }

        // ✅ ACCEPT BLOCK
        self.blocks.push(block);
        self.rebuild_utxos();
        self.save_all();

        true
    }

    /* ---------- CONSENSUS V2 HELPERS ---------- */

    fn enforce_coinbase_maturity(
        &self,
        tx: &Transaction,
        current_height: u64,
    ) -> bool {
        // Coinbase tx itself is always valid
        if tx.inputs.is_empty() {
            return true;
        }

        for input in &tx.inputs {
            let key = format!(
                "{}:{}",
                hex::encode(&input.txid),
                input.index
            );

            let utxo = match self.utxos.get(&key) {
                Some(u) => u,
                None => return false,
            };

            if utxo.is_coinbase {
                if current_height < utxo.height + COINBASE_MATURITY {
                    return false;
                }
            }
        }

        true
    }

    /* ---------- REORG ---------- */

    pub fn maybe_reorg(&mut self, candidate: Vec<Block>) -> Option<Vec<Block>> {
        if !self.validate_chain(&candidate) {
            return None;
        }

        if cumulative_work(&candidate)
            <= cumulative_work(&self.blocks)
        {
            return None;
        }

        let mut fork_height = 0;
        for i in 0..self.blocks.len().min(candidate.len()) {
            if self.blocks[i].hash != candidate[i].hash {
                break;
            }
            fork_height = i as u64;
        }

        let orphaned = self.disconnect_to_height(fork_height);
        self.blocks = candidate;
        self.rebuild_utxos();
        self.save_all();

        Some(orphaned)
    }

    pub fn disconnect_to_height(&mut self, height: u64) -> Vec<Block> {
        let mut orphaned = Vec::new();

        while self.height() > height {
            if let Some(b) = self.blocks.pop() {
                orphaned.push(b);
            }
        }

        self.rebuild_utxos();
        orphaned
    }

    /* ---------- INIT ---------- */

    pub fn initialize(&mut self) {
        fs::create_dir_all(data_dir()).unwrap();

        if blocks_file().exists() {
            let data = fs::read_to_string(blocks_file()).unwrap();
            if !data.trim().is_empty() {
                self.blocks = serde_json::from_str(&data).unwrap();
            }
        }

        if self.blocks.is_empty() {
            let txs = vec![revelation_tx()];
            let difficulty =
                calculate_next_difficulty(&self.blocks);

            let mut genesis = Block {
                header: BlockHeader {
                    height: 0,
                    timestamp: 1730000000,
                    prev_hash: vec![0u8; 32],
                    nonce: 0,
                    difficulty,
                    merkle_root: merkle_root(&txs),
                },
                transactions: txs,
                hash: vec![],
            };

            mine(&mut genesis);
            self.blocks.push(genesis);
        }

        self.rebuild_utxos();
        self.save_all();
    }

    /* ---------- MINING ---------- */

    pub fn mine_block(
        &mut self,
        miner_pubkey_hash: Vec<u8>,
        mempool_txs: Vec<Transaction>,
    ) {
        let height = self.height();
        let prev = self.blocks.last().unwrap();

        let coinbase = Transaction {
            inputs: vec![],
            outputs: vec![TxOutput {
                value: block_reward(height),
                pubkey_hash: miner_pubkey_hash,
            }],
        };

        let mut txs = vec![coinbase];

        for tx in mempool_txs {
            if validate_transaction(&tx, &self.utxos) {
                txs.push(tx);
            }
        }

        let difficulty =
            calculate_next_difficulty(&self.blocks);

        let mut block = Block {
            header: BlockHeader {
                height,
                timestamp: OffsetDateTime::now_utc().unix_timestamp(),
                prev_hash: prev.hash.clone(),
                nonce: 0,
                difficulty,
                merkle_root: merkle_root(&txs),
            },
            transactions: txs,
            hash: vec![],
        };

        mine(&mut block);

        // Always re-validate our own block
        let _ = self.validate_and_add_block(block);
    }

    /* ---------- UTXO ---------- */

    pub fn rebuild_utxos(&mut self) {
        self.utxos.clear();

        for block in &self.blocks {
            for (tx_index, tx) in block.transactions.iter().enumerate() {
                let txid = hex::encode(tx.txid());

                for input in &tx.inputs {
                    let key =
                        format!("{}:{}", hex::encode(&input.txid), input.index);
                    self.utxos.remove(&key);
                }

                let is_coinbase = tx_index == 0 && tx.inputs.is_empty();

                for (i, o) in tx.outputs.iter().enumerate() {
                    self.utxos.insert(
                        format!("{}:{}", txid, i),
                        UTXO {
                            value: o.value,
                            pubkey_hash: o.pubkey_hash.clone(),
                            height: block.header.height,
                            is_coinbase,
                        },
                    );
                }
            }
        }
    }

    /* ---------- PERSIST ---------- */

    pub fn save_all(&self) {
        fs::create_dir_all(data_dir()).unwrap();
        fs::write(
            blocks_file(),
            serde_json::to_string_pretty(&self.blocks).unwrap(),
        )
        .unwrap();
        fs::write(
            utxos_file(),
            serde_json::to_string_pretty(&self.utxos).unwrap(),
        )
        .unwrap();
    }

    /* ---------- FULL CHAIN VALIDATION ---------- */

    pub fn validate_chain(&self, chain: &[Block]) -> bool {
        if chain.is_empty() {
            return false;
        }

        for i in 1..chain.len() {
            let prev = &chain[i - 1];
            let b = &chain[i];

            if b.header.height != prev.header.height + 1 {
                return false;
            }

            if b.header.prev_hash != prev.hash {
                return false;
            }

            if b.header.timestamp <= prev.header.timestamp {
                return false;
            }

            if !b.verify_pow() {
                return false;
            }

            if merkle_root(&b.transactions) != b.header.merkle_root {
                return false;
            }
        }

        true
    }
}
