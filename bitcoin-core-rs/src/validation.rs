use crate::transaction::Transaction;
use crate::utxo::UTXOSet;
use crate::crypto::{verify_signature, pubkey_hash};

/// ⚠️ CONSENSUS — MUST NOT CHANGE WITHOUT A VERSIONED FORK
///
/// Transaction validation rules enforced by consensus.
///
/// This function MUST be used by:
/// - block validation
/// - mempool admission
///
/// ❗ No policy logic is allowed here.
/// ❗ Any modification is a consensus change.
pub fn validate_transaction(tx: &Transaction, utxos: &UTXOSet) -> bool {
    // ── Coinbase transactions ─────────────────────────────
    //
    // Coinbase transactions are only valid inside blocks.
    // Structural validity is enforced elsewhere.
    if tx.inputs.is_empty() {
        return true;
    }

    let sighash = tx.sighash();
    let mut input_sum: u64 = 0;
    let mut output_sum: u64 = 0;

    // ── Validate inputs ───────────────────────────────────
    for input in &tx.inputs {
        let key = format!(
            "{}:{}",
            hex::encode(&input.txid),
            input.index
        );

        let utxo = match utxos.get(&key) {
            Some(u) => u,
            None => return false, // referenced UTXO must exist
        };

        // 🔑 Ownership check:
        // sha256(pubkey) must match the UTXO lock
        if pubkey_hash(&input.pubkey) != utxo.pubkey_hash {
            return false;
        }

        // ✍️ Signature verification (ECDSA, deterministic)
        //
        // The sighash commits to the entire transaction.
        // This is a simplified but safe script-less model.
        if !verify_signature(
            &sighash,
            &input.signature,
            &input.pubkey,
        ) {
            return false;
        }

        input_sum = input_sum.saturating_add(utxo.value);
    }

    // ── Validate outputs ──────────────────────────────────
    for output in &tx.outputs {
        output_sum = output_sum.saturating_add(output.value);
    }

    // 🔒 No inflation:
    // total output value must not exceed total input value
    input_sum >= output_sum
}
