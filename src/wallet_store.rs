use std::collections::HashMap;
use std::fs;

use serde::{Serialize, Deserialize};

const WALLET_FILE: &str = "data/wallets.json";

/// Wallet registry (POLICY ONLY)
#[derive(Serialize, Deserialize)]
pub struct WalletStore {
    /// wallet name → wallet file path
    pub wallets: HashMap<String, String>,
}

impl WalletStore {
    pub fn new() -> Self {
        Self {
            wallets: HashMap::new(),
        }
    }

    pub fn list(&self) -> Vec<String> {
        self.wallets.keys().cloned().collect()
    }

    pub fn get_path(&self, name: &str) -> Option<&String> {
        self.wallets.get(name)
    }
}

/// Load wallet store from disk
pub fn load_wallet_store() -> WalletStore {
    fs::create_dir_all("data").unwrap();

    let mut store = if let Ok(data) = fs::read_to_string(WALLET_FILE) {
        if !data.trim().is_empty() {
            serde_json::from_str(&data).expect("invalid wallets.json")
        } else {
            WalletStore::new()
        }
    } else {
        WalletStore::new()
    };

    if !store.wallets.contains_key("default") {
        store.wallets.insert(
            "default".to_string(),
            "data/wallet.dat".to_string(),
        );
        save_wallet_store(&store);
    }

    store
}

pub fn save_wallet_store(store: &WalletStore) {
    fs::write(
        WALLET_FILE,
        serde_json::to_string_pretty(store).unwrap(),
    ).unwrap();
}
