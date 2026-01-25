pub mod config;
pub mod policy;
pub mod pow;
pub mod revelation;
pub mod reward;
pub mod wallet;
pub mod wallet_store;
pub mod crypto;
pub mod consensus;
pub mod node;        // ✅ REQUIRED
pub mod interface;   // ✅ REQUIRED

// New module tree
pub mod core;

pub use core::block;
pub use core::transaction;
pub use core::utxo;
pub use core::merkle;
pub use core::validation;
pub use core::chain;

pub use crypto::{sha256, pubkey_hash, verify_signature};

pub const CLIENT_VERSION: &str = "0.3.0-consensus-v3";
