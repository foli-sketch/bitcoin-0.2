use crate::core::transaction::{Transaction, TxOutput};
use crate::crypto::sha256;

pub fn revelation_tx() -> Transaction {
    Transaction {
        inputs: vec![],
        outputs: vec![TxOutput {
            value: 0,
            pubkey_hash: sha256(
                "REVELATION BLOCK 0 — \
WEF Agenda 2030 sealed into Proof-of-Work time. \
No authority. No reversal. No governance. \
Truth revealed by computation."
                    .as_bytes(),
            ),
        }],
    }
}
