use rand::rngs::OsRng;
use ssh_key::{Algorithm, PrivateKey};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crate::error::{Result, SkmError};
use crate::ssh::keys::{KeyType, SshKey};

pub struct KeyGenerator {
    ssh_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct KeyGenOptions {
    pub key_type: KeyType,
    pub bits: Option<u32>,
    pub comment: String,
    pub passphrase: Option<String>,
    pub filename: String,
}

impl Default for KeyGenOptions {
    fn default() -> Self {
        Self {
            key_type: KeyType::Ed25519,
            bits: None,
            comment: format!("{}@{}", get_username(), get_hostname()),
            passphrase: None,
            filename: "id_ed25519".to_string(),
        }
    }
}

impl KeyGenerator {
    pub fn new<P: AsRef<Path>>(ssh_dir: P) -> Self {
        Self {
            ssh_dir: ssh_dir.as_ref().to_path_buf(),
        }
    }

    pub fn generate(&self, options: KeyGenOptions) -> Result<SshKey> {
        let private_path = self.ssh_dir.join(&options.filename);
        let public_path = private_path.with_extension("pub");

        if private_path.exists() {
            return Err(SkmError::KeyAlreadyExists(
                private_path.to_string_lossy().to_string(),
            ));
        }

        let (private_key, public_key) = match options.key_type {
            KeyType::Ed25519 => self.generate_ed25519()?,
            KeyType::Rsa => {
                return Err(SkmError::SshKey(
                    "RSA generation not yet implemented".to_string(),
                ));
            }
            _ => {
                return Err(SkmError::SshKey(format!(
                    "Key type {} not yet supported for generation",
                    options.key_type
                )));
            }
        };

        // Write private key
        self.write_private_key(&private_path, &private_key, options.passphrase.as_deref())?;

        // Write public key
        let public_key_openssh = public_key
            .to_openssh()
            .map_err(|e| SkmError::SshKey(e.to_string()))?;
        let public_content = format!("{} {}", public_key.algorithm(), public_key_openssh);
        self.write_public_key(&public_path, &public_content, &options.comment)?;

        SshKey::from_path(&private_path)
    }

    fn generate_ed25519(&self) -> Result<(PrivateKey, ssh_key::PublicKey)> {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)
            .map_err(|e| SkmError::SshKey(e.to_string()))?;
        let public_key = private_key.public_key().clone();
        Ok((private_key, public_key))
    }

    fn write_private_key(
        &self,
        path: &Path,
        key: &PrivateKey,
        _passphrase: Option<&str>,
    ) -> Result<()> {
        let pem = key
            .to_openssh(ssh_key::LineEnding::default())
            .map_err(|e| SkmError::SshKey(e.to_string()))?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .map_err(SkmError::Io)?;

        file.write_all(pem.as_bytes()).map_err(SkmError::Io)?;

        Ok(())
    }

    fn write_public_key(&self, path: &Path, key_data: &str, comment: &str) -> Result<()> {
        let content = format!("{} {}", key_data, comment);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o644)
            .open(path)
            .map_err(SkmError::Io)?;

        file.write_all(content.as_bytes()).map_err(SkmError::Io)?;

        Ok(())
    }
}

fn get_username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string())
}

fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h: std::ffi::OsString| h.into_string().ok())
        .unwrap_or("localhost".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_key_gen_options_default() {
        let opts = KeyGenOptions::default();
        assert_eq!(opts.key_type, KeyType::Ed25519);
        assert_eq!(opts.filename, "id_ed25519");
    }

    #[test]
    fn test_generate_ed25519() {
        let temp_dir = TempDir::new().unwrap();
        let generator = KeyGenerator::new(temp_dir.path());

        let opts = KeyGenOptions {
            key_type: KeyType::Ed25519,
            filename: "id_ed25519".to_string(),
            comment: "test@example.com".to_string(),
            passphrase: None,
            bits: None,
        };

        let key = generator.generate(opts).unwrap();

        assert_eq!(key.name, "id_ed25519");
        assert_eq!(key.key_type, KeyType::Ed25519);
        assert!(key.path.exists());
        assert!(key.public_path.exists());
    }

    #[test]
    fn test_generate_duplicate_key_fails() {
        let temp_dir = TempDir::new().unwrap();
        let generator = KeyGenerator::new(temp_dir.path());

        let opts = KeyGenOptions {
            filename: "test_key".to_string(),
            ..Default::default()
        };

        generator.generate(opts.clone()).unwrap();

        let result = generator.generate(opts);
        assert!(matches!(result, Err(SkmError::KeyAlreadyExists(_))));
    }
}
