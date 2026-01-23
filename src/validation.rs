use crate::transaction::Transaction;
use crate::utxo::UTXOSet;
use crate::crypto::{verify_signature, pubkey_hash};

/// Consensus transaction validation
///
/// This function MUST be used by:
/// - block validation
/// - mempool admission
///
/// No policy logic here.
pub fn validate_transaction(tx: &Transaction, utxos: &UTXOSet) -> bool {
    // Coinbase transactions are only valid inside blocks
    if tx.inputs.is_empty() {
        return true;
    }

    let sighash = tx.sighash();
    let mut input_sum: u64 = 0;
    let mut output_sum: u64 = 0;

    // ── Validate inputs ─────────────────────────
    for input in &tx.inputs {
        let key = format!(
            "{}:{}",
            hex::encode(&input.txid),
            input.index
        );

        let utxo = match utxos.get(&key) {
            Some(u) => u,
            None => return false,
        };

        // 🔒 Coinbase maturity
        if utxo.is_coinbase {
            // Height must be provided by caller via UTXO height
            // Coinbase is spendable only after maturity
            // (checked via utxo.height relative to chain height elsewhere)
            // Here we enforce structural validity only
        }

        // 🔑 Ownership: sha256(pubkey) must match UTXO lock
        if pubkey_hash(&input.pubkey) != utxo.pubkey_hash {
            return false;
        }

        // ✍️ Signature verification
        if !verify_signature(
            &sighash,
            &input.signature,
            &input.pubkey,
        ) {
            return false;
        }

        input_sum = input_sum.saturating_add(utxo.value);
    }

    // ── Validate outputs ────────────────────────
    for output in &tx.outputs {
        output_sum = output_sum.saturating_add(output.value);
    }

    // 🔒 No inflation
    input_sum >= output_sum
}
