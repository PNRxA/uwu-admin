use chacha20poly1305::{
    ChaCha20Poly1305, KeyInit, Nonce,
    aead::Aead,
};

pub fn encrypt(key: &[u8], plaintext: &str) -> Result<String, String> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| format!("Invalid encryption key: {e}"))?;

    let nonce_bytes: [u8; 12] = {
        use rand::Rng;
        rand::thread_rng().r#gen()
    };
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {e}"))?;

    // Prepend nonce to ciphertext and hex-encode
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(bytes_to_hex(&combined))
}

pub fn decrypt(key: &[u8], hex_data: &str) -> Result<String, String> {
    let data = hex_to_bytes(hex_data)?;

    if data.len() < 12 {
        return Err("Encrypted data too short".into());
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| format!("Invalid encryption key: {e}"))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {e}"))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8 in decrypted data: {e}"))
}

/// Heuristic to distinguish encrypted (hex-encoded) tokens from raw Matrix tokens.
/// Encrypted tokens are all hex digits and long enough (nonce + ciphertext + tag).
/// Raw Matrix tokens contain non-hex chars like `_`.
pub fn is_encrypted(value: &str) -> bool {
    value.len() >= 58 && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Odd-length hex string".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}
