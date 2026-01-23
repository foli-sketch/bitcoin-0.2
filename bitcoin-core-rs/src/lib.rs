pub mod block;
pub mod chain;
pub mod pow;
pub mod utxo;
pub mod transaction;
pub mod crypto;
pub mod merkle;
pub mod reward;
pub mod revelation;
pub mod validation;
pub mod consensus;
pub mod mempool;
pub mod policy;
pub mod network;
pub mod p2p;
pub mod api;
pub mod wallet;
pub mod miner;

pub use crypto::{sha256, pubkey_hash};
pub use crypto::verify_signature;

/// ⚠️ CLIENT VERSION
///
/// This identifier reflects the frozen L1 consensus version.
/// It MUST NOT be changed without a version-gated fork.
pub const CLIENT_VERSION: &str = "0.3.0-consensus-v3";
