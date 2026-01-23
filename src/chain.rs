// ─────────────────────────────────────────────
// CONSENSUS v3 — FROZEN
//
// The rules in this module define L1 consensus.
// Any modification requires a version-gated fork.
// Do NOT refactor, optimize, or "clean up" casually.
// ─────────────────────────────────────────────

use std::collections::HashMap;
use std::fs;
use std::env;
use std::path::PathBuf;

use time::OffsetDateTime;

use crate::consensus::{
    fork_choice::cumulative_work,
    difficulty::calculate_next_target,
    params::*,
};

use crate::{
    block::{Block, BlockHeader},
    pow::mine,
    utxo::{UTXOSet, UTXO},
    transaction::Transaction,
    reward::block_reward,
    revelation::revelation_tx,
    merkle::merkle_root,
    validation::validate_transaction,
};

/// ─────────────────────────────────────────────
/// Consensus parameters
/// ─────────────────────────────────────────────

/// Coinbase outputs may only be spent after this many blocks
#[allow(dead_code)]
const COINBASE_MATURITY: u64 = 100;

/// Height at which consensus v2 activates
const CONSENSUS_V2_HEIGHT: u64 = 1000;

/// ─────────────────────────────────────────────
/// Blockchain
/// ─────────────────────────────────────────────

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub utxos: UTXOSet,
}

/// ─────────────────────────────────────────────
/// Data files
/// ─────────────────────────────────────────────

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

/// ─────────────────────────────────────────────
/// Helpers
/// ─────────────────────────────────────────────

fn median_time_past(chain: &[Block]) -> i64 {
    let mut times: Vec<i64> = chain
        .iter()
        .rev()
        .take(MTP_WINDOW)
        .map(|b| b.header.timestamp)
        .collect();

    times.sort();
    times[times.len() / 2]
}

/// ─────────────────────────────────────────────
/// Implementation
/// ─────────────────────────────────────────────

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

    /* ───────────── BLOCK VALIDATION ───────────── */

    pub fn validate_and_add_block(&mut self, block: Block) -> bool {
        let expected_height = self.height();

        // ── Height & linkage ──────────────────────
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
        }

        // ── Median Time Past + future drift ───────
        if !self.blocks.is_empty() {
            let mtp = median_time_past(&self.blocks);

            if block.header.timestamp <= mtp {
                return false;
            }

            if block.header.timestamp >
                OffsetDateTime::now_utc().unix_timestamp()
                    + MAX_FUTURE_DRIFT
            {
                return false;
            }
        }

        // ── Expected PoW target (CONSENSUS) ───────
        let expected_target =
            calculate_next_target(&self.blocks);

        if block.header.target != expected_target {
            return false;
        }

        // ── Proof-of-Work ─────────────────────────
        if !block.verify_pow() {
            return false;
        }

        // ── Merkle root ───────────────────────────
        if merkle_root(&block.transactions)
            != block.header.merkle_root
        {
            return false;
        }

        // ── Block size (CONSENSUS) ─────────────────
        let block_size =
            bincode::serialize(&block).unwrap().len();

        if block_size > MAX_BLOCK_SIZE {
            return false;
        }

        // ── Coinbase rules ────────────────────────
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

        // ── Transaction validation ────────────────
        for tx in &block.transactions {
            if !validate_transaction(tx, &self.utxos) {
                return false;
            }

            // Consensus v2: coinbase maturity
            if block.header.height >= CONSENSUS_V2_HEIGHT {
                if !self.enforce_coinbase_maturity(
                    tx,
                    block.header.height,
                ) {
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

    fn enforce_coinbase_maturity(
        &self,
        tx: &Transaction,
        current_height: u64,
    ) -> bool {
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
                if current_height
                    < utxo.height + COINBASE_MATURITY
                {
                    return false;
                }
            }
        }

        true
    }

    /* ───────────── REORG LOGIC ───────────── */

    pub fn maybe_reorg(
        &mut self,
        candidate: Vec<Block>,
    ) -> Option<Vec<Block>> {
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

        let orphaned =
            self.disconnect_to_height(fork_height);

        self.blocks = candidate;
        self.rebuild_utxos();
        self.save_all();

        Some(orphaned)
    }

    pub fn disconnect_to_height(
        &mut self,
        height: u64,
    ) -> Vec<Block> {
        let mut orphaned = Vec::new();

        while self.height() > height {
            if let Some(b) = self.blocks.pop() {
                orphaned.push(b);
            }
        }

        self.rebuild_utxos();
        orphaned
    }

    /* ───────────── INITIALIZATION ───────────── */

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
            let target =
                calculate_next_target(&self.blocks);

            let mut genesis = Block {
                header: BlockHeader {
                    height: 0,
                    timestamp: 1730000000,
                    prev_hash: vec![0u8; 32],
                    nonce: 0,
                    target,
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

    /* ───────────── UTXO ───────────── */

    pub fn rebuild_utxos(&mut self) {
        self.utxos.clear();

        for block in &self.blocks {
            for (tx_index, tx) in
                block.transactions.iter().enumerate()
            {
                let txid = hex::encode(tx.txid());

                for input in &tx.inputs {
                    let key = format!(
                        "{}:{}",
                        hex::encode(&input.txid),
                        input.index
                    );
                    self.utxos.remove(&key);
                }

                let is_coinbase =
                    tx_index == 0 && tx.inputs.is_empty();

                for (i, o) in
                    tx.outputs.iter().enumerate()
                {
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

    /* ───────────── PERSISTENCE ───────────── */

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

    /* ───────────── FULL CHAIN VALIDATION ───────────── */

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

            let expected =
                calculate_next_target(&chain[..i]);

            if b.header.target != expected {
                return false;
            }

            if !b.verify_pow() {
                return false;
            }

            if merkle_root(&b.transactions)
                != b.header.merkle_root
            {
                return false;
            }
        }

        true
    }
}
