use chacha20poly1305::{
    ChaCha20Poly1305, KeyInit, Nonce,
    aead::Aead,
};

const ENC_PREFIX: &str = "enc:";

pub fn encrypt(key: &[u8], plaintext: &str) -> Result<String, String> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| format!("Invalid encryption key: {e}"))?;

    let nonce_bytes: [u8; 12] = {
        use rand::Rng;
        rand::rngs::OsRng.r#gen()
    };
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {e}"))?;

    // Prepend nonce to ciphertext, hex-encode, and add prefix
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(format!("{}{}", ENC_PREFIX, bytes_to_hex(&combined)))
}

pub fn decrypt(key: &[u8], token: &str) -> Result<String, String> {
    // Accept both prefixed and unprefixed for backward compat
    let hex_data = token.strip_prefix(ENC_PREFIX).unwrap_or(token);

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

/// Deterministic check: encrypted tokens always start with the `enc:` prefix.
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENC_PREFIX)
}

/// Legacy heuristic for migration: all-hex and long enough (nonce + ciphertext + tag).
pub fn is_legacy_encrypted(value: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: [u8; 32] = [0xAA; 32];

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let plaintext = "hello world";
        let encrypted = encrypt(&TEST_KEY, plaintext).unwrap();
        let decrypted = decrypt(&TEST_KEY, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypted_has_enc_prefix() {
        let encrypted = encrypt(&TEST_KEY, "test").unwrap();
        assert!(encrypted.starts_with("enc:"));
    }

    #[test]
    fn decrypt_without_prefix() {
        let encrypted = encrypt(&TEST_KEY, "backward compat").unwrap();
        // Strip the prefix to simulate legacy format
        let legacy = encrypted.strip_prefix("enc:").unwrap();
        let decrypted = decrypt(&TEST_KEY, legacy).unwrap();
        assert_eq!(decrypted, "backward compat");
    }

    #[test]
    fn different_nonces_each_time() {
        let a = encrypt(&TEST_KEY, "same").unwrap();
        let b = encrypt(&TEST_KEY, "same").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn wrong_key_fails() {
        let encrypted = encrypt(&TEST_KEY, "secret").unwrap();
        let wrong_key = [0xBB; 32];
        assert!(decrypt(&wrong_key, &encrypted).is_err());
    }

    #[test]
    fn truncated_data_fails() {
        // 12 bytes nonce = 24 hex chars minimum; provide less
        assert!(decrypt(&TEST_KEY, "enc:aabb").is_err());
    }

    #[test]
    fn corrupted_data_fails() {
        let encrypted = encrypt(&TEST_KEY, "data").unwrap();
        let mut chars: Vec<char> = encrypted.chars().collect();
        // Flip a byte in the ciphertext area (after "enc:" prefix + 24 hex nonce chars)
        let idx = 30;
        if idx < chars.len() {
            chars[idx] = if chars[idx] == 'a' { 'b' } else { 'a' };
        }
        let corrupted: String = chars.into_iter().collect();
        assert!(decrypt(&TEST_KEY, &corrupted).is_err());
    }

    #[test]
    fn is_encrypted_and_legacy_checks() {
        assert!(is_encrypted("enc:deadbeef"));
        assert!(!is_encrypted("deadbeef"));

        // Legacy: all-hex and >= 58 chars
        let long_hex = "a".repeat(60);
        assert!(is_legacy_encrypted(&long_hex));
        assert!(!is_legacy_encrypted("short"));
        assert!(!is_legacy_encrypted(&"g".repeat(60))); // non-hex
    }
}
