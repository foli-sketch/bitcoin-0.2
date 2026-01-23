use time::OffsetDateTime;

use crate::{
    block::{Block, BlockHeader},
    transaction::{Transaction, TxOutput},
    reward::block_reward,
    consensus::difficulty::calculate_next_target,
    merkle::merkle_root,
    pow::mine,
    validation::validate_transaction,
    utxo::UTXOSet,
    policy::{MAX_BLOCK_TXS, MAX_BLOCK_TX_BYTES},
};

/// Build and mine a block candidate
///
/// ⚠ POLICY ONLY
/// ⚠ NOT CONSENSUS
/// ⚠ MUST NEVER BE USED FOR VALIDATION
pub fn mine_block(
    prev_block: &Block,
    utxos: &UTXOSet,
    mempool_txs: Vec<Transaction>,
    miner_pubkey_hash: Vec<u8>,
    chain: &[Block],
) -> Block {
    let height = prev_block.header.height + 1;

    // ── Coinbase ───────────────────────────────
    let coinbase = Transaction {
        inputs: vec![],
        outputs: vec![TxOutput {
            value: block_reward(height),
            pubkey_hash: miner_pubkey_hash,
        }],
    };

    let mut selected = vec![coinbase];
    let mut total_bytes = selected[0].serialized_size();

    // ── Transaction selection (policy only) ────
    for tx in mempool_txs {
        if selected.len() >= MAX_BLOCK_TXS {
            break;
        }

        let size = tx.serialized_size();
        if total_bytes + size > MAX_BLOCK_TX_BYTES {
            break;
        }

        if validate_transaction(&tx, utxos) {
            total_bytes += size;
            selected.push(tx);
        }
    }

    // ── Target (Phase-1 PoW) ───────────────────
    let target = calculate_next_target(chain);

    // ── Assemble block ─────────────────────────
    let mut block = Block {
        header: BlockHeader {
            height,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
            prev_hash: prev_block.hash.clone(),
            nonce: 0,
            target,
            merkle_root: merkle_root(&selected),
        },
        transactions: selected,
        hash: vec![],
    };

    // ── Mine ───────────────────────────────────
    mine(&mut block);

    block
}
