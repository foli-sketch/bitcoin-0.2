use std::collections::HashMap;

use crate::crypto::pubkey_hash;
use crate::transaction::{Transaction, TxInput, TxOutput};
use crate::utxo::UTXO;

use secp256k1::{Secp256k1, Message, SecretKey, PublicKey};

#[derive(Clone)]
pub struct Wallet {
    pub private_key: SecretKey,
    pub public_key: Vec<u8>,
}

impl Wallet {
    /// 🧪 Deterministic dev wallet (DO NOT USE IN PRODUCTION)
    pub fn new_dev() -> Self {
        let secp = Secp256k1::new();

        // fixed private key for dev/testing
        let sk_bytes = [1u8; 32];
        let private_key =
            SecretKey::from_slice(&sk_bytes).expect("valid dev private key");

        let public_key =
            PublicKey::from_secret_key(&secp, &private_key);

        Wallet {
            private_key,
            public_key: public_key.serialize().to_vec(),
        }
    }

    /// Wallet identifier (Bitcoin-style address = pubkey hash)
    pub fn address(&self) -> Vec<u8> {
        self.pubkey_hash()
    }

    /// sha256(pubkey)
    pub fn pubkey_hash(&self) -> Vec<u8> {
        pubkey_hash(&self.public_key)
    }

    /// Read-only derived balance (UTXO-based)
    pub fn balance(&self, utxos: &HashMap<String, UTXO>) -> u64 {
        let my_pubkey_hash = self.pubkey_hash();

        utxos
            .values()
            .filter(|utxo| utxo.pubkey_hash == my_pubkey_hash)
            .map(|utxo| utxo.value)
            .sum()
    }

    /// 💸 Build + sign a transaction (wallet-side only)
    pub fn send(
        &self,
        utxos: &HashMap<String, UTXO>,
        to_pubkey_hash: Vec<u8>,
        amount: u64,
        fee: u64,
    ) -> Transaction {
        let my_pubkey_hash = self.pubkey_hash();

        // 1️⃣ Collect owned UTXOs: (txid, index, utxo)
        let mut owned = Vec::new();
        for (key, utxo) in utxos {
            if utxo.pubkey_hash == my_pubkey_hash {
                let (txid, index) = parse_utxo_key(key);
                owned.push((txid, index, utxo.clone()));
            }
        }

        // 2️⃣ Largest-first selection
        owned.sort_by(|a, b| b.2.value.cmp(&a.2.value));

        let mut selected = Vec::new();
        let mut input_sum = 0;

        for utxo in owned {
            selected.push(utxo.clone());
            input_sum += utxo.2.value;

            if input_sum >= amount + fee {
                break;
            }
        }

        if input_sum < amount + fee {
            panic!("insufficient funds");
        }

        // 3️⃣ Build inputs
        let mut inputs = Vec::new();
        for (txid, index, _) in &selected {
            inputs.push(TxInput {
                txid: txid.clone(),
                index: *index,
                signature: vec![],
                pubkey: vec![],
            });
        }

        // 4️⃣ Build outputs
        let mut outputs = Vec::new();

        // recipient
        outputs.push(TxOutput {
            value: amount,
            pubkey_hash: to_pubkey_hash,
        });

        // change
        let change = input_sum - amount - fee;
        if change > 0 {
            outputs.push(TxOutput {
                value: change,
                pubkey_hash: my_pubkey_hash,
            });
        }

        let mut tx = Transaction { inputs, outputs };

        // 5️⃣ Sign (tx-wide sighash, matching your verifier)
        let sighash = tx.sighash();
        let signature = sign_sighash(&sighash, &self.private_key);

        for input in tx.inputs.iter_mut() {
            input.signature = signature.clone();
            input.pubkey = self.public_key.clone();
        }

        tx
    }
}

/// 🔐 Wallet-side ECDSA signing (DER-encoded)
fn sign_sighash(sighash: &[u8], private_key: &SecretKey) -> Vec<u8> {
    let secp = Secp256k1::signing_only();

    let msg = Message::from_digest_slice(sighash)
        .expect("invalid sighash length");

    let sig = secp.sign_ecdsa(&msg, private_key);
    sig.serialize_der().to_vec()
}

/// Parse "txid:index" → (txid_bytes, index)
fn parse_utxo_key(key: &str) -> (Vec<u8>, usize) {
    let parts: Vec<&str> = key.split(':').collect();
    if parts.len() != 2 {
        panic!("invalid UTXO key format");
    }

    let txid = hex::decode(parts[0]).expect("invalid txid hex");
    let index = parts[1].parse::<usize>().expect("invalid index");

    (txid, index)
}
