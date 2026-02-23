use log::debug;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    debug!("[Utils] Password hashed using SHA-256.");
    hex::encode(result)
}

pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let hashed_input = hash_password(password);
    debug!("[Utils] Password verification completed.");
    hashed_input.as_bytes().ct_eq(stored_hash.as_bytes()).into()
}
