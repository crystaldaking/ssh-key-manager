use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::error::{Result, SkmError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    Rsa,
    Ed25519,
    Ecdsa,
    Dsa,
    #[serde(other)]
    Unknown,
}

impl fmt::Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyType::Rsa => write!(f, "RSA"),
            KeyType::Ed25519 => write!(f, "ED25519"),
            KeyType::Ecdsa => write!(f, "ECDSA"),
            KeyType::Dsa => write!(f, "DSA"),
            KeyType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl KeyType {
    pub fn from_filename(filename: &str) -> Self {
        if filename.contains("rsa") {
            KeyType::Rsa
        } else if filename.contains("ed25519") {
            KeyType::Ed25519
        } else if filename.contains("ecdsa") {
            KeyType::Ecdsa
        } else if filename.contains("dsa") {
            KeyType::Dsa
        } else {
            KeyType::Unknown
        }
    }

    pub const fn default_filename(&self) -> &'static str {
        match self {
            KeyType::Rsa => "id_rsa",
            KeyType::Ed25519 => "id_ed25519",
            KeyType::Ecdsa => "id_ecdsa",
            KeyType::Dsa => "id_dsa",
            KeyType::Unknown => "id_unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStatus {
    Valid,
    MissingPublic,
    MissingPrivate,
    Corrupted,
    Encrypted,
}

impl fmt::Display for KeyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyStatus::Valid => write!(f, "Valid"),
            KeyStatus::MissingPublic => write!(f, "Missing Public"),
            KeyStatus::MissingPrivate => write!(f, "Missing Private"),
            KeyStatus::Corrupted => write!(f, "Corrupted"),
            KeyStatus::Encrypted => write!(f, "Encrypted"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKey {
    pub name: String,
    pub path: PathBuf,
    pub public_path: PathBuf,
    pub key_type: KeyType,
    pub status: KeyStatus,
    pub fingerprint: Option<String>,
    pub comment: Option<String>,
    pub created_at: Option<DateTime<Local>>,
    pub modified_at: Option<DateTime<Local>>,
    pub size: Option<u32>,
}

impl SshKey {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let key_type = KeyType::from_filename(&name);
        let public_path = path.with_extension("pub");

        let status = Self::determine_status(path, &public_path);
        let metadata = std::fs::metadata(path).ok();

        let created_at = metadata
            .as_ref()
            .and_then(|m| m.created().ok())
            .map(|t| t.into());

        let modified_at = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(|t| t.into());

        let (fingerprint, comment) = if public_path.exists() {
            Self::parse_public_key(&public_path).unwrap_or((None, None))
        } else {
            (None, None)
        };

        Ok(Self {
            name,
            path: path.to_path_buf(),
            public_path,
            key_type,
            status,
            fingerprint,
            comment,
            created_at,
            modified_at,
            size: None,
        })
    }

    fn determine_status(private_path: &Path, public_path: &Path) -> KeyStatus {
        let private_exists = private_path.exists();
        let public_exists = public_path.exists();

        if private_exists && public_exists {
            // TODO: Check if key is encrypted or corrupted
            KeyStatus::Valid
        } else if private_exists && !public_exists {
            KeyStatus::MissingPublic
        } else if !private_exists && public_exists {
            KeyStatus::MissingPrivate
        } else {
            KeyStatus::Corrupted
        }
    }

    fn parse_public_key(path: &Path) -> Result<(Option<String>, Option<String>)> {
        let content = std::fs::read_to_string(path)?;
        let parts: Vec<&str> = content.trim().split_whitespace().collect();

        if parts.len() >= 2 {
            let fingerprint = Some(format!("{}...", &parts[1][..parts[1].len().min(16)]));
            let comment = if parts.len() >= 3 {
                Some(parts[2..].join(" "))
            } else {
                None
            };
            Ok((fingerprint, comment))
        } else {
            Ok((None, None))
        }
    }

    pub fn has_private(&self) -> bool {
        self.path.exists()
    }

    pub fn has_public(&self) -> bool {
        self.public_path.exists()
    }

    pub fn read_public_content(&self) -> Result<Option<String>> {
        if self.public_path.exists() {
            Ok(Some(std::fs::read_to_string(&self.public_path)?))
        } else {
            Ok(None)
        }
    }

    pub fn update_comment(&mut self, new_comment: &str) -> Result<()> {
        if !self.public_path.exists() {
            return Err(SkmError::KeyNotFound(
                self.public_path.to_string_lossy().to_string(),
            ));
        }

        let content = std::fs::read_to_string(&self.public_path)?;
        let parts: Vec<&str> = content.trim().split_whitespace().collect();

        if parts.len() >= 2 {
            let new_content = format!("{} {} {}", parts[0], parts[1], new_comment);
            std::fs::write(&self.public_path, new_content)?;
            self.comment = Some(new_comment.to_string());
            Ok(())
        } else {
            Err(SkmError::InvalidKeyFormat(
                "Invalid public key format".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_key_type_from_filename() {
        assert_eq!(KeyType::from_filename("id_rsa"), KeyType::Rsa);
        assert_eq!(KeyType::from_filename("id_ed25519"), KeyType::Ed25519);
        assert_eq!(KeyType::from_filename("id_ecdsa"), KeyType::Ecdsa);
        assert_eq!(KeyType::from_filename("id_dsa"), KeyType::Dsa);
        assert_eq!(KeyType::from_filename("unknown"), KeyType::Unknown);
    }

    #[test]
    fn test_key_type_display() {
        assert_eq!(KeyType::Rsa.to_string(), "RSA");
        assert_eq!(KeyType::Ed25519.to_string(), "ED25519");
    }

    #[test]
    fn test_ssh_key_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("id_rsa");
        std::fs::write(&key_path, "test private key").unwrap();

        let key = SshKey::from_path(&key_path).unwrap();
        assert_eq!(key.name, "id_rsa");
        assert_eq!(key.key_type, KeyType::Rsa);
        assert_eq!(key.status, KeyStatus::MissingPublic);
    }

    #[test]
    fn test_parse_public_key() {
        let temp_dir = TempDir::new().unwrap();
        let pub_path = temp_dir.path().join("test.pub");
        std::fs::write(&pub_path, "ssh-rsa AAAAB3NzaC1 user@example.com").unwrap();

        let result = SshKey::parse_public_key(&pub_path).unwrap();
        assert!(result.0.is_some());
        assert_eq!(result.1, Some("user@example.com".to_string()));
    }
}
