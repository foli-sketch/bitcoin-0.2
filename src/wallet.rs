use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroize;
use memsec::mlock;

use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit},
};
use aes_gcm::aead::generic_array::GenericArray;

use sha2::{Sha256, Digest};
use pbkdf2::pbkdf2_hmac;

#[allow(unused_imports)]
use secp256k1::{SecretKey, PublicKey};

use bip39::{Mnemonic, Language};
use hex;

use crate::crypto::{
    secret_key_from_seed,
    public_key,
    pubkey_hash,
    sign,
};

use crate::core::transaction::{Transaction, TxInput, TxOutput};
use crate::core::utxo::UTXOSet;

const WALLET_FILE: &str = "data/wallet.dat";
const AUTO_LOCK_SECS: u64 = 60;

/* ───────── Encrypted Wallet File (DISK) ───────── */

#[derive(serde::Serialize, serde::Deserialize)]
struct WalletFile {
    version: u32,
    encrypted_master_seed: Vec<u8>,
    password_salt: Vec<u8>,
    nonce: Vec<u8>,
    next_index: u32,
}

/* ───────── Memory Lock Helper ───────── */

fn lock_memory(bytes: &mut [u8]) {
    unsafe {
        mlock(bytes.as_mut_ptr(), bytes.len());
    }
}

/* ───────── HD Derivation (Deterministic) ───────── */

fn derive_child_seed(master: &[u8; 32], index: u32) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(master);
    hasher.update(index.to_be_bytes());
    let hash = hasher.finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&hash[..32]);
    out
}

/* ───────── Runtime Wallet (MEMORY ONLY) ───────── */

pub struct Wallet {
    master_seed: Option<[u8; 32]>,
    last_unlock: Option<Instant>,
    next_index: u32,
}

impl Wallet {
    /* ---------- Load or Create ---------- */

    pub fn load_or_create(password: &str) -> Self {
        fs::create_dir_all("data").unwrap();

        if Path::new(WALLET_FILE).exists() {
            let mut w = Wallet {
                master_seed: None,
                last_unlock: None,
                next_index: 0,
            };
            w.unlock(password).expect("wallet unlock failed");
            w
        } else {
            Self::create_new(password)
        }
    }

    /* ---------- Create New Wallet ---------- */

    fn create_new(password: &str) -> Self {
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .expect("mnemonic generation failed");

        println!("\n⚠️ WRITE THIS DOWN — WALLET RECOVERY PHRASE ⚠️");
        println!("{}", mnemonic.to_string());
        println!("⚠️ ANYONE WITH THESE WORDS CAN SPEND YOUR COINS ⚠️\n");

        Self::create_from_mnemonic(password, &mnemonic.to_string())
            .expect("wallet creation failed")
    }

    /* ---------- Restore From Mnemonic ---------- */

    pub fn create_from_mnemonic(
        password: &str,
        mnemonic_phrase: &str,
    ) -> Result<Self, &'static str> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
            .map_err(|_| "invalid mnemonic")?;

        let seed = mnemonic.to_seed("");
        let mut master_seed = [0u8; 32];
        master_seed.copy_from_slice(&seed[..32]);

        let mut password_salt = [0u8; 16];
        OsRng.fill_bytes(&mut password_salt);

        let mut enc_key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            &password_salt,
            300_000,
            &mut enc_key,
        );

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&enc_key));

        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);

        let encrypted_master_seed = cipher
            .encrypt(GenericArray::from_slice(&nonce), &master_seed[..])
            .map_err(|_| "seed encryption failed")?;

        let wf = WalletFile {
            version: 3,
            encrypted_master_seed,
            password_salt: password_salt.to_vec(),
            nonce: nonce.to_vec(),
            next_index: 0,
        };

        fs::write(WALLET_FILE, bincode::serialize(&wf).unwrap()).unwrap();

        lock_memory(&mut master_seed);

        Ok(Wallet {
            master_seed: Some(master_seed),
            last_unlock: Some(Instant::now()),
            next_index: 0,
        })
    }

    /* ---------- Unlock ---------- */

    pub fn unlock(&mut self, password: &str) -> Result<(), ()> {
        let data = fs::read(WALLET_FILE).map_err(|_| ())?;
        let wf: WalletFile = bincode::deserialize(&data).map_err(|_| ())?;

        let mut enc_key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            &wf.password_salt,
            300_000,
            &mut enc_key,
        );

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&enc_key));

        let seed_bytes = cipher
            .decrypt(GenericArray::from_slice(&wf.nonce), wf.encrypted_master_seed.as_ref())
            .map_err(|_| ())?;

        let mut master_seed = [0u8; 32];
        master_seed.copy_from_slice(&seed_bytes[..32]);

        lock_memory(&mut master_seed);

        self.master_seed = Some(master_seed);
        self.last_unlock = Some(Instant::now());
        self.next_index = wf.next_index;

        Ok(())
    }

    /* ---------- Lock ---------- */

    pub fn lock(&mut self) {
        if let Some(mut s) = self.master_seed.take() {
            s.zeroize();
        }
        self.last_unlock = None;
    }

    pub fn is_unlocked(&mut self) -> bool {
        if let Some(t) = self.last_unlock {
            if t.elapsed() > Duration::from_secs(AUTO_LOCK_SECS) {
                self.lock();
                return false;
            }
            true
        } else {
            false
        }
    }

    /* ---------- Address #0 ---------- */

    pub fn address(&self) -> Result<Vec<u8>, &'static str> {
        let master = self.master_seed.ok_or("wallet locked")?;
        let child = derive_child_seed(&master, 0);
        let sk = secret_key_from_seed(&child);
        let pk = public_key(&sk);
        Ok(pubkey_hash(&pk))
    }

    /* ---------- Generate New HD Address ---------- */

    pub fn next_address(&mut self) -> Result<Vec<u8>, &'static str> {
        let master = self.master_seed.ok_or("wallet locked")?;

        let index = self.next_index;
        let child = derive_child_seed(&master, index);

        let sk = secret_key_from_seed(&child);
        let pk = public_key(&sk);

        self.next_index += 1;

        Ok(pubkey_hash(&pk))
    }

    /* ---------- Sign With Address Index ---------- */

    pub fn sign_with_index(
        &self,
        msg: &[u8],
        index: u32,
    ) -> Result<Vec<u8>, &'static str> {
        let master = self.master_seed.ok_or("wallet locked")?;
        let child = derive_child_seed(&master, index);
        let sk = secret_key_from_seed(&child);
        Ok(sign(msg, &sk))
    }

    /* ---------- CREATE TRANSACTION ---------- */

    pub fn create_transaction(
        &mut self,
        utxos: &UTXOSet,
        to_pubkey_hash: Vec<u8>,
        amount: u64,
    ) -> Result<Transaction, &'static str> {
        let master_seed = self.master_seed.ok_or("wallet locked")?;

        let mut collected = 0u64;
        let mut selected = Vec::new();

        for (key, utxo) in utxos {
            for index in 0..20 {
                let child = derive_child_seed(&master_seed, index);
                let sk = secret_key_from_seed(&child);
                let pk = public_key(&sk);
                let hash = pubkey_hash(&pk);

                if hash == utxo.pubkey_hash {
                    let parts: Vec<&str> = key.split(':').collect();
                    let txid = hex::decode(parts[0]).unwrap();
                    let vout = parts[1].parse::<u32>().unwrap();

                    selected.push((txid, vout, index, utxo.value));
                    collected += utxo.value;

                    if collected >= amount {
                        break;
                    }
                }
            }
            if collected >= amount {
                break;
            }
        }

        if collected < amount {
            return Err("not enough funds");
        }

        let mut outputs = vec![TxOutput {
            value: amount,
            pubkey_hash: to_pubkey_hash,
        }];

        let change = collected - amount;
        if change > 0 {
            let change_addr = self.next_address()?;
            outputs.push(TxOutput {
                value: change,
                pubkey_hash: change_addr,
            });
        }

        let mut tx = Transaction {
            inputs: Vec::new(),
            outputs,
        };

        let sighash = tx.sighash();

        for (txid, vout, index, _) in selected {
            let sig = self.sign_with_index(&sighash, index)?;
            let child = derive_child_seed(&master_seed, index);
            let sk = secret_key_from_seed(&child);
            let pk = public_key(&sk);

            tx.inputs.push(TxInput {
                txid,
                index: vout,
                signature: sig,
                pubkey: pk.serialize().to_vec(),
                address_index: index,
            });
        }

        Ok(tx)
    }
}
