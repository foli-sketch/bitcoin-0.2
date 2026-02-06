// ─────────────────────────────────────────────
// CONSENSUS v4 — FORK CHOICE (HARDFORK)
//
// Defines cumulative Proof-of-Work fork selection.
// Any modification requires a new hard fork.
// ─────────────────────────────────────────────

use crate::core::block::Block;
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::collections::HashMap;

/// Cumulative work of a single block
///
/// work = 2^256 / (target + 1)
pub fn block_work(block: &Block) -> BigUint {
    let target = BigUint::from_bytes_be(&block.header.target);

    if target.is_zero() {
        return BigUint::zero();
    }

    (BigUint::one() << 256u32) / (target + BigUint::one())
}

/// Track cumulative work per block hash
pub fn compute_cumulative_work(
    blocks: &[Block],
) -> HashMap<Vec<u8>, BigUint> {
    let mut map = HashMap::new();

    for block in blocks {
        let work = block_work(block);

        let total = if block.header.height == 0 {
            work
        } else {
            let prev = map
                .get(&block.header.prev_hash)
                .cloned()
                .unwrap_or_else(BigUint::zero);

            prev + work
        };

        map.insert(block.hash.clone(), total);
    }

    map
}

/// Select the best chain tip by cumulative work
pub fn best_tip(blocks: &[Block]) -> Option<Vec<u8>> {
    let work_map = compute_cumulative_work(blocks);

    work_map
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1))
        .map(|(hash, _)| hash)
}
