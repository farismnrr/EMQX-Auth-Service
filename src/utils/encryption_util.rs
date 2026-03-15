#![allow(dead_code)]

use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use thiserror::Error;

/// Encryption error type
#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Invalid key format: {0}")]
    InvalidKey(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
}

/// Convert hex string to 32-byte key array
pub fn hex_to_key32(hex: &str) -> Result<[u8; 32], EncryptionError> {
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(EncryptionError::InvalidKey(format!(
            "Key must be 64 hex characters (32 bytes), got {} characters",
            hex.len()
        )));
    }

    let bytes = hex::decode(hex).map_err(|e| EncryptionError::InvalidKey(e.to_string()))?;

    if bytes.len() != 32 {
        return Err(EncryptionError::InvalidKey(format!(
            "Key must be exactly 32 bytes, got {} bytes",
            bytes.len()
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

/// Encrypts data using AES-256-GCM with a random 12-byte nonce.
/// Returns a base64 encoded string containing [nonce + ciphertext].
pub fn aes256_gcm_encrypt(plaintext: &str, key: &[u8; 32]) -> Result<String, EncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

    let nonce_bytes: [u8; 12] = rand::thread_rng().r#gen();
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

    let mut final_payload = nonce_bytes.to_vec();
    final_payload.extend(ciphertext);

    Ok(general_purpose::STANDARD.encode(final_payload))
}

/// Decrypts data using AES-256-GCM.
/// Expects a base64 encoded string containing [nonce + ciphertext].
pub fn aes256_gcm_decrypt(encrypted_payload: &str, key: &[u8; 32]) -> Result<String, EncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

    let data = general_purpose::STANDARD
        .decode(encrypted_payload)
        .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

    if data.len() < 12 {
        return Err(EncryptionError::DecryptionFailed(
            "Invalid encrypted payload (too short)".to_string(),
        ));
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce_arr: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| EncryptionError::DecryptionFailed("Invalid nonce length".to_string()))?;
    let nonce = Nonce::from(nonce_arr);

    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

    String::from_utf8(plaintext).map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
}
