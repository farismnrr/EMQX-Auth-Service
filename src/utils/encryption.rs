use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::Aead,
};
use base64::{Engine as _, engine::general_purpose};
use std::env;
use rand::Rng;

pub fn encrypt_password(password: &str) -> Result<String, String> {
    let key_hex = env::var("MQTT_PASS_ENCRYPTION_KEY")
        .map_err(|_| "MQTT_PASS_ENCRYPTION_KEY not set".to_string())?;
    let key_bytes = hex::decode(key_hex.trim()).map_err(|e| e.to_string())?;

    if key_bytes.len() != 32 {
        return Err(format!(
            "Encryption key must be 32 bytes (64 hex characters), got {} bytes",
            key_bytes.len()
        ));
    }

    // new_from_slice avoids the deprecated GenericArray::from_slice
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;

    // Generate a unique 96-bit nonce (12 bytes)
    let nonce_bytes: [u8; 12] = rand::thread_rng().r#gen();
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, password.as_bytes())
        .map_err(|e| e.to_string())?;

    // Prepend nonce to ciphertext and Base64-encode
    let mut final_payload = nonce_bytes.to_vec();
    final_payload.extend(ciphertext);

    Ok(general_purpose::STANDARD.encode(final_payload))
}

pub fn decrypt_password(encrypted_payload: &str) -> Result<String, String> {
    let key_hex = env::var("MQTT_PASS_ENCRYPTION_KEY")
        .map_err(|_| "MQTT_PASS_ENCRYPTION_KEY not set".to_string())?;
    let key_bytes = hex::decode(key_hex.trim()).map_err(|e| e.to_string())?;

    if key_bytes.len() != 32 {
        return Err("Encryption key must be 32 bytes (64 hex characters)".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;

    let data = general_purpose::STANDARD
        .decode(encrypted_payload)
        .map_err(|e| e.to_string())?;

    if data.len() < 12 {
        return Err("Invalid encrypted payload (too short)".to_string());
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    // Nonce::clone_from_slice avoids GenericArray::from_slice
    let nonce_arr: [u8; 12] = nonce_bytes.try_into().map_err(|_| "Invalid nonce length".to_string())?;
    let nonce = Nonce::from(nonce_arr);

    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| e.to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}
