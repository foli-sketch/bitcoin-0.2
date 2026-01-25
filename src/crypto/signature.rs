use secp256k1::{
    Secp256k1, SecretKey, PublicKey, Message,
    ecdsa::Signature,
};
use sha2::{Sha256, Digest};

/// SHA256 helper
pub fn sha256(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}

/// secp256k1 private key from wallet seed
pub fn secret_key_from_seed(seed: &[u8; 32]) -> SecretKey {
    SecretKey::from_slice(seed)
        .expect("invalid secp256k1 secret key")
}

/// derive public key
pub fn public_key(sk: &SecretKey) -> PublicKey {
    let secp = Secp256k1::new();
    PublicKey::from_secret_key(&secp, sk)
}

/// HASH160-style owner id (simplified = SHA256(pubkey))
pub fn pubkey_hash_from_bytes(pubkey_bytes: &[u8]) -> Vec<u8> {
    sha256(pubkey_bytes)
}

/// keep this for wallet code that already uses PublicKey
pub fn pubkey_hash(pubkey: &PublicKey) -> Vec<u8> {
    sha256(&pubkey.serialize())
}

/// sign message (wallet side)
pub fn sign(msg: &[u8], sk: &SecretKey) -> Vec<u8> {
    let secp = Secp256k1::new();
    let hash = sha256(msg);
    let message = Message::from_digest_slice(&hash).expect("32 bytes");

    secp.sign_ecdsa(&message, sk)
        .serialize_compact()
        .to_vec()
}

/// verify signature using raw pubkey bytes (validation side)
pub fn verify_signature(
    msg: &[u8],
    sig_bytes: &[u8],
    pubkey_bytes: &[u8],
) -> bool {
    let secp = Secp256k1::verification_only();

    let hash = sha256(msg);
    let message = match Message::from_digest_slice(&hash) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let sig = match Signature::from_compact(sig_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let pubkey = match PublicKey::from_slice(pubkey_bytes) {
        Ok(p) => p,
        Err(_) => return false,
    };

    secp.verify_ecdsa(&message, &sig, &pubkey).is_ok()
}
