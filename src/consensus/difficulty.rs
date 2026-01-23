use crate::block::Block;
use crate::consensus::params::*;

/// Calculate the expected difficulty for the NEXT block
///
/// Rules:
/// - Difficulty only changes at fixed intervals
/// - Change is bounded (±1)
/// - Based on actual vs expected wall-clock time
pub fn calculate_next_difficulty(chain: &[Block]) -> u32 {
    let height = chain.len();

    // Genesis / early blocks
    if height < DIFFICULTY_ADJUSTMENT_INTERVAL + 1 {
        return chain
            .last()
            .map(|b| b.header.difficulty)
            .unwrap_or(MIN_DIFFICULTY);
    }

    // Only adjust at interval boundaries
    if height % DIFFICULTY_ADJUSTMENT_INTERVAL != 0 {
        return chain.last().unwrap().header.difficulty;
    }

    let last = chain.last().unwrap();
    let first = &chain[height - DIFFICULTY_ADJUSTMENT_INTERVAL - 1];

    let actual_time = last.header.timestamp - first.header.timestamp;
    let expected_time =
        TARGET_BLOCK_TIME * DIFFICULTY_ADJUSTMENT_INTERVAL as i64;

    let mut difficulty = last.header.difficulty;

    // Bounded adjustment (anti-shock)
    if actual_time < expected_time / 2 {
        difficulty = difficulty.saturating_add(1);
    } else if actual_time > expected_time * 2 {
        difficulty = difficulty.saturating_sub(1);
    }

    difficulty.clamp(MIN_DIFFICULTY, MAX_DIFFICULTY)
}
