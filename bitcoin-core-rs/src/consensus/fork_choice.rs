use crate::block::Block;
use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Total accumulated work for a chain
///
/// work ≈ Σ (2^256 / (target + 1))
pub fn cumulative_work(chain: &[Block]) -> BigUint {
    let mut total = BigUint::zero();

    for b in chain {
        let target = BigUint::from_bytes_be(&b.header.target);

        // Prevent division by zero
        if target.is_zero() {
            continue;
        }

        let work = (BigUint::one() << 256u32) / (target + BigUint::one());
        total += work;
    }

    total
}
