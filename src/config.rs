use std::fs;
use serde::{Serialize, Deserialize};

const CONFIG_FILE: &str = "data/miner_config.json";

/// Miner configuration (POLICY ONLY)
#[derive(Serialize, Deserialize)]
pub struct MinerConfig {
    /// Wallet name used for coinbase rewards
    pub coinbase_wallet: String,
}

/// Load miner configuration from disk
pub fn load_miner_config() -> MinerConfig {
    fs::create_dir_all("data").unwrap();

    if let Ok(data) = fs::read_to_string(CONFIG_FILE) {
        if !data.trim().is_empty() {
            return serde_json::from_str(&data)
                .expect("invalid miner_config.json");
        }
    }

    let default = MinerConfig {
        coinbase_wallet: "default".to_string(),
    };

    fs::write(
        CONFIG_FILE,
        serde_json::to_string_pretty(&default).unwrap(),
    ).unwrap();

    default
}
