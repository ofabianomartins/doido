use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::path::Path;
use doido_core::{Result, anyhow::Context as _};

/// Encrypts `plaintext` with `key` using AES-256-GCM with a random nonce.
/// Returns a base64-encoded blob: `nonce(12 bytes) || ciphertext`.
pub fn encrypt_credentials(plaintext: &str, key: &[u8; 32]) -> Result<String> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| doido_core::anyhow::anyhow!("AES-GCM encryption failed"))?;
    let mut out = nonce.to_vec();
    out.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&out))
}

/// Decrypts a base64-encoded blob produced by `encrypt_credentials`.
pub fn decrypt_credentials(encoded: &str, key: &[u8; 32]) -> Result<String> {
    let raw = STANDARD
        .decode(encoded.trim())
        .map_err(|e| doido_core::anyhow::anyhow!("base64 decode failed: {e}"))?;
    if raw.len() < 12 {
        doido_core::anyhow::bail!("credentials blob too short to contain nonce");
    }
    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| doido_core::anyhow::anyhow!("decryption failed — wrong key?"))?;
    String::from_utf8(plaintext)
        .map_err(|e| doido_core::anyhow::anyhow!("credentials are not valid UTF-8: {e}"))
}

/// Resolves the 32-byte master key:
/// 1. `DOIDO_MASTER_KEY` env var (64-char hex string)
/// 2. `config/master.key` file (64-char hex string, trailing whitespace trimmed)
pub(crate) fn load_master_key(root: &Path) -> Result<[u8; 32]> {
    let hex_str = std::env::var("DOIDO_MASTER_KEY").or_else(|_| {
        let key_path = root.join("config/master.key");
        std::fs::read_to_string(&key_path)
            .map(|s| s.trim().to_string())
            .map_err(|e| doido_core::anyhow::anyhow!("cannot read config/master.key: {e}"))
    })?;
    let bytes = hex::decode(hex_str.trim())
        .context("master key is not valid hex")?;
    bytes
        .try_into()
        .map_err(|_| doido_core::anyhow::anyhow!("master key must be 32 bytes (64 hex chars)"))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;

    fn all_zeros_key() -> [u8; 32] {
        [0u8; 32]
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let key = all_zeros_key();
        let plaintext = "[database]\nurl = \"postgres://secret@host/db\"\n";
        let encrypted = super::encrypt_credentials(plaintext, &key).unwrap();
        let decrypted = super::decrypt_credentials(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_each_encryption_produces_unique_ciphertext() {
        let key = all_zeros_key();
        let c1 = super::encrypt_credentials("secret", &key).unwrap();
        let c2 = super::encrypt_credentials("secret", &key).unwrap();
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_decrypt_fails_with_wrong_key() {
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let encrypted = super::encrypt_credentials("secret", &key1).unwrap();
        let result = super::decrypt_credentials(&encrypted, &key2);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("decryption failed"));
    }

    #[test]
    fn test_decrypt_fails_on_garbage_input() {
        let key = all_zeros_key();
        let result = super::decrypt_credentials("not-base64!!!", &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_master_key_from_file() {
        let dir = TempDir::new().unwrap();
        let hex_key = "00".repeat(32);
        let key_path = dir.path().join("config/master.key");
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        fs::write(&key_path, format!("{hex_key}\n")).unwrap();
        let key = super::load_master_key(dir.path()).unwrap();
        assert_eq!(key, [0u8; 32]);
    }

    #[test]
    fn test_load_master_key_rejects_wrong_length() {
        let dir = TempDir::new().unwrap();
        let key_path = dir.path().join("config/master.key");
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        fs::write(&key_path, "deadbeef").unwrap();
        if std::env::var("DOIDO_MASTER_KEY").is_err() {
            let result = super::load_master_key(dir.path());
            assert!(result.is_err());
            let msg = result.unwrap_err().to_string();
            assert!(msg.contains("32 bytes"), "got: {msg}");
        }
    }

    #[test]
    fn test_load_master_key_rejects_invalid_hex() {
        let dir = TempDir::new().unwrap();
        let key_path = dir.path().join("config/master.key");
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        fs::write(&key_path, "not-valid-hex-string-at-all-!!!!").unwrap();
        if std::env::var("DOIDO_MASTER_KEY").is_err() {
            let result = super::load_master_key(dir.path());
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("valid hex"));
        }
    }
}
