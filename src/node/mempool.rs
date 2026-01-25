use crate::transaction::Transaction;
use crate::utxo::UTXOSet;
use crate::policy::MAX_TX_SIZE;
use crate::validation::validate_transaction;
use crate::block::Block;

use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct MempoolEntry {
    pub tx: Transaction,
    pub fee: i64,
    pub size: usize,
    pub timestamp: i64,
}

pub struct Mempool {
    entries: Vec<MempoolEntry>,
    spent_outpoints: HashSet<(Vec<u8>, u32)>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            spent_outpoints: HashSet::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn add_transaction(
        &mut self,
        tx: Transaction,
        utxos: &UTXOSet,
        chain_height: u64,   // 🔥 NEW
    ) -> bool {
        if tx.inputs.is_empty() {
            return false;
        }

        let size = tx.serialized_size();
        if size > MAX_TX_SIZE {
            return false;
        }

        // 🔥 pass current chain height
        if !validate_transaction(&tx, utxos, chain_height) {
            return false;
        }

        for input in &tx.inputs {
            let key = (input.txid.clone(), input.index);
            if self.spent_outpoints.contains(&key) {
                return false;
            }
        }

        let fee = match calculate_fee(&tx, utxos) {
            Some(f) => f,
            None => return false,
        };

        if fee <= 0 {
            return false;
        }

        for input in &tx.inputs {
            self.spent_outpoints.insert((input.txid.clone(), input.index));
        }

        self.entries.push(MempoolEntry {
            tx,
            fee,
            size,
            timestamp: now(),
        });

        true
    }

    pub fn sorted_for_mining(&self) -> Vec<Transaction> {
        let mut entries = self.entries.clone();

        entries.sort_by(|a, b| {
            let lhs = a.fee * b.size as i64;
            let rhs = b.fee * a.size as i64;
            rhs.cmp(&lhs)
        });

        entries.into_iter().map(|e| e.tx).collect()
    }

    pub fn remove_confirmed(&mut self, confirmed: &[Transaction]) {
        self.entries.retain(|entry| {
            !confirmed.iter().any(|tx| tx.txid() == entry.tx.txid())
        });

        self.rebuild_spent_outpoints();
    }

    pub fn resurrect_from_orphans(
        &mut self,
        orphaned: Vec<Block>,
        utxos: &UTXOSet,
        chain_height: u64, // 🔥 NEW
    ) {
        for block in orphaned {
            for tx in block.transactions.into_iter().skip(1) {
                let _ = self.add_transaction(tx, utxos, chain_height);
            }
        }
    }

    fn rebuild_spent_outpoints(&mut self) {
        self.spent_outpoints.clear();

        for entry in &self.entries {
            for input in &entry.tx.inputs {
                self.spent_outpoints.insert((input.txid.clone(), input.index));
            }
        }
    }
}

fn calculate_fee(tx: &Transaction, utxos: &UTXOSet) -> Option<i64> {
    let mut input_sum = 0i64;
    let mut output_sum = 0i64;

    for input in &tx.inputs {
        let key = format!("{}:{}", hex::encode(&input.txid), input.index);
        let utxo = utxos.get(&key)?;
        input_sum += utxo.value as i64;
    }

    for output in &tx.outputs {
        output_sum += output.value as i64;
    }

    Some(input_sum - output_sum)
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_secs() as i64
}
