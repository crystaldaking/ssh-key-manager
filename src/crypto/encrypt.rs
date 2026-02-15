use age::secrecy::SecretString;
use std::io::{Read, Write};

use crate::error::{Result, SkmError};

pub struct EncryptionManager;

impl EncryptionManager {
    /// Encrypt data with a passphrase using age
    pub fn encrypt_with_passphrase(data: &[u8], passphrase: &str) -> Result<Vec<u8>> {
        let encryptor = age::Encryptor::with_user_passphrase(SecretString::from(passphrase));

        let mut encrypted = vec![];
        let mut writer = encryptor
            .wrap_output(&mut encrypted)
            .map_err(|e| SkmError::Encryption(e.to_string()))?;

        writer
            .write_all(data)
            .map_err(|e| SkmError::Encryption(e.to_string()))?;
        writer
            .finish()
            .map_err(|e| SkmError::Encryption(e.to_string()))?;

        Ok(encrypted)
    }

    /// Decrypt data with a passphrase
    pub fn decrypt_with_passphrase(encrypted: &[u8], passphrase: &str) -> Result<Vec<u8>> {
        let decryptor =
            age::Decryptor::new(encrypted).map_err(|e| SkmError::Encryption(e.to_string()))?;

        let mut decrypted = vec![];

        // Create passphrase identity
        let identity = age::scrypt::Identity::new(SecretString::from(passphrase));

        // Decrypt using the passphrase identity
        let mut reader = decryptor
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .map_err(|_| SkmError::InvalidPassphrase)?;

        reader
            .read_to_end(&mut decrypted)
            .map_err(|e| SkmError::Encryption(e.to_string()))?;

        Ok(decrypted)
    }

    /// Encrypt and encode to armor format (ASCII)
    pub fn encrypt_to_armor(data: &[u8], passphrase: &str) -> Result<String> {
        let encrypted = Self::encrypt_with_passphrase(data, passphrase)?;
        let armor =
            age::armor::ArmoredWriter::wrap_output(Vec::new(), age::armor::Format::AsciiArmor)
                .map_err(|e| SkmError::Encryption(e.to_string()))?;

        let mut armor_writer = armor;
        armor_writer
            .write_all(&encrypted)
            .map_err(|e| SkmError::Encryption(e.to_string()))?;
        let result = armor_writer
            .finish()
            .map_err(|e| SkmError::Encryption(e.to_string()))?;

        String::from_utf8(result).map_err(|e| SkmError::Encryption(format!("Invalid UTF-8: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = b"test data for encryption";
        let passphrase = "test_password";

        let encrypted = EncryptionManager::encrypt_with_passphrase(data, passphrase).unwrap();
        assert_ne!(encrypted, data.to_vec());

        let decrypted = EncryptionManager::decrypt_with_passphrase(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, data.to_vec());
    }

    #[test]
    fn test_decrypt_wrong_passphrase() {
        let data = b"test data";
        let encrypted = EncryptionManager::encrypt_with_passphrase(data, "correct").unwrap();

        let result = EncryptionManager::decrypt_with_passphrase(&encrypted, "wrong");
        assert!(result.is_err());
    }
}
