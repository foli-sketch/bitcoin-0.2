use crate::block::Block;
use crate::consensus::params::*;

fn clamp_target(target: [u8; 32]) -> [u8; 32] {
    if target > MAX_TARGET {
        MAX_TARGET
    } else if target < MIN_TARGET {
        MIN_TARGET
    } else {
        target
    }
}

/// Calculate expected PoW target for NEXT block
pub fn calculate_next_target(chain: &[Block]) -> [u8; 32] {
    if chain.is_empty() {
        return MAX_TARGET;
    }

    let height = chain.len();

    let last = chain.last().unwrap();

    if height < DIFFICULTY_ADJUSTMENT_INTERVAL + 1 {
        return last.header.target;
    }

    if height % DIFFICULTY_ADJUSTMENT_INTERVAL != 0 {
        return last.header.target;
    }

    let first =
        &chain[height - DIFFICULTY_ADJUSTMENT_INTERVAL - 1];

    let actual_time =
        last.header.timestamp - first.header.timestamp;

    let expected_time =
        TARGET_BLOCK_TIME * DIFFICULTY_ADJUSTMENT_INTERVAL as i64;

    let mut new_target = last.header.target;

    for i in 0..32 {
        let scaled =
            (new_target[i] as i128 * actual_time as i128)
                / expected_time as i128;

        new_target[i] =
            scaled.clamp(0, 255) as u8;
    }

    clamp_target(new_target)
}
