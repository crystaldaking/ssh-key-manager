use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::crypto::encrypt::EncryptionManager;
use crate::error::{Result, SkmError};
use crate::ssh::keys::SshKey;

const BACKUP_VERSION: u32 = 1;
const BACKUP_EXTENSION: &str = "skm";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub version: u32,
    pub created_at: DateTime<Local>,
    pub hostname: String,
    pub username: String,
    pub key_count: usize,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub name: String,
    pub key_type: String,
    pub comment: Option<String>,
    pub private_key: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub metadata: BackupMetadata,
    pub keys: Vec<BackupEntry>,
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub description: Option<String>,
    pub include_public_only: bool,
    pub selected_keys: Option<Vec<String>>, // None = all keys
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            description: None,
            include_public_only: false,
            selected_keys: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub merge_strategy: MergeStrategy,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    SkipExisting, // Skip keys that already exist
    Overwrite,    // Overwrite existing keys
    Rename,       // Rename with timestamp suffix
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            merge_strategy: MergeStrategy::SkipExisting,
            dry_run: false,
        }
    }
}

pub struct BackupManager {
    ssh_dir: PathBuf,
}

impl BackupManager {
    pub fn new<P: AsRef<Path>>(ssh_dir: P) -> Self {
        Self {
            ssh_dir: ssh_dir.as_ref().to_path_buf(),
        }
    }

    /// Export keys to encrypted backup file
    pub fn export(
        &self,
        keys: &[SshKey],
        output_path: &Path,
        passphrase: &str,
        options: ExportOptions,
    ) -> Result<()> {
        let mut backup_keys = Vec::new();

        for key in keys {
            // Filter if specific keys selected
            if let Some(ref selected) = options.selected_keys {
                if !selected.contains(&key.name) {
                    continue;
                }
            }

            let entry = BackupEntry {
                name: key.name.clone(),
                key_type: key.key_type.to_string(),
                comment: key.comment.clone(),
                private_key: if options.include_public_only {
                    None
                } else {
                    self.read_file_if_exists(&key.path)?
                },
                public_key: self.read_file_if_exists(&key.public_path)?,
            };

            backup_keys.push(entry);
        }

        let backup = BackupData {
            metadata: BackupMetadata {
                version: BACKUP_VERSION,
                created_at: Local::now(),
                hostname: get_hostname(),
                username: get_username(),
                key_count: backup_keys.len(),
                description: options.description,
            },
            keys: backup_keys,
        };

        // Serialize to JSON
        let json =
            serde_json::to_vec(&backup).map_err(|e| SkmError::ImportExport(e.to_string()))?;

        // Encrypt
        let encrypted = EncryptionManager::encrypt_with_passphrase(&json, passphrase)?;

        // Write to file
        let mut file = fs::File::create(output_path).map_err(SkmError::Io)?;
        file.write_all(&encrypted).map_err(SkmError::Io)?;

        Ok(())
    }

    /// Import keys from encrypted backup file
    pub fn import(
        &self,
        backup_path: &Path,
        passphrase: &str,
        options: ImportOptions,
    ) -> Result<ImportReport> {
        // Read encrypted file
        let encrypted = fs::read(backup_path).map_err(SkmError::Io)?;

        // Decrypt
        let decrypted = EncryptionManager::decrypt_with_passphrase(&encrypted, passphrase)?;

        // Parse JSON
        let backup: BackupData = serde_json::from_slice(&decrypted)
            .map_err(|e| SkmError::ImportExport(format!("Invalid backup format: {}", e)))?;

        let mut report = ImportReport {
            imported: Vec::new(),
            skipped: Vec::new(),
            overwritten: Vec::new(),
            errors: Vec::new(),
        };

        if options.dry_run {
            // Just report what would happen
            for entry in backup.keys {
                let target_path = self.ssh_dir.join(&entry.name);
                if target_path.exists() {
                    match options.merge_strategy {
                        MergeStrategy::SkipExisting => report.skipped.push(entry.name),
                        MergeStrategy::Overwrite => report.overwritten.push(entry.name),
                        MergeStrategy::Rename => report
                            .imported
                            .push(format!("{} -> {}_{{timestamp}}", entry.name, entry.name)),
                    }
                } else {
                    report.imported.push(entry.name);
                }
            }
            return Ok(report);
        }

        // Actually import
        for entry in backup.keys {
            match self.import_entry(&entry, options.merge_strategy) {
                Ok(ImportResult::Imported(name)) => report.imported.push(name),
                Ok(ImportResult::Skipped(name)) => report.skipped.push(name),
                Ok(ImportResult::Overwritten(name)) => report.overwritten.push(name),
                Err(e) => report.errors.push((entry.name, e.to_string())),
            }
        }

        Ok(report)
    }

    fn import_entry(&self, entry: &BackupEntry, strategy: MergeStrategy) -> Result<ImportResult> {
        let private_path = self.ssh_dir.join(&entry.name);
        let public_path = private_path.with_extension("pub");

        // Check if exists
        let exists = private_path.exists() || public_path.exists();

        if exists {
            match strategy {
                MergeStrategy::SkipExisting => {
                    return Ok(ImportResult::Skipped(entry.name.clone()));
                }
                MergeStrategy::Rename => {
                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                    let new_name = format!("{}_{}", entry.name, timestamp);
                    return self
                        .write_key_files(&new_name, entry)
                        .map(|_| ImportResult::Imported(new_name));
                }
                MergeStrategy::Overwrite => {
                    // Continue to write
                }
            }
        }

        self.write_key_files(&entry.name, entry)?;

        if exists {
            Ok(ImportResult::Overwritten(entry.name.clone()))
        } else {
            Ok(ImportResult::Imported(entry.name.clone()))
        }
    }

    fn write_key_files(&self, name: &str, entry: &BackupEntry) -> Result<()> {
        let private_path = self.ssh_dir.join(name);
        let public_path = private_path.with_extension("pub");

        // Write private key if present
        if let Some(ref private_data) = entry.private_key {
            fs::write(&private_path, private_data).map_err(SkmError::Io)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&private_path)?.permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&private_path, perms)?;
            }
        }

        // Write public key if present
        if let Some(ref public_data) = entry.public_key {
            fs::write(&public_path, public_data).map_err(SkmError::Io)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&public_path)?.permissions();
                perms.set_mode(0o644);
                fs::set_permissions(&public_path, perms)?;
            }
        }

        Ok(())
    }

    fn read_file_if_exists(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        if path.exists() {
            fs::read(path).map(Some).map_err(SkmError::Io)
        } else {
            Ok(None)
        }
    }

    pub fn get_backup_extension() -> &'static str {
        BACKUP_EXTENSION
    }
}

#[derive(Debug, Clone)]
pub struct ImportReport {
    pub imported: Vec<String>,
    pub skipped: Vec<String>,
    pub overwritten: Vec<String>,
    pub errors: Vec<(String, String)>,
}

enum ImportResult {
    Imported(String),
    Skipped(String),
    Overwritten(String),
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

    fn create_test_key(temp_dir: &TempDir, name: &str) -> SshKey {
        let key_path = temp_dir.path().join(name);
        let pub_path = temp_dir.path().join(format!("{}.pub", name));
        fs::write(&key_path, "private").unwrap();
        fs::write(&pub_path, "public").unwrap();

        SshKey::from_path(&key_path).unwrap()
    }

    #[test]
    fn test_export_import_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let export_dir = TempDir::new().unwrap();

        // Create test key
        let key = create_test_key(&temp_dir, "test_key");

        // Export
        let manager = BackupManager::new(temp_dir.path());
        let backup_path = export_dir.path().join("backup.skm");

        manager
            .export(&[key], &backup_path, "test_pass", ExportOptions::default())
            .unwrap();

        assert!(backup_path.exists());

        // Import to new location
        let import_dir = TempDir::new().unwrap();
        let import_manager = BackupManager::new(import_dir.path());

        let report = import_manager
            .import(&backup_path, "test_pass", ImportOptions::default())
            .unwrap();

        assert_eq!(report.imported.len(), 1);
        assert!(import_dir.path().join("test_key").exists());
    }

    #[test]
    fn test_import_wrong_passphrase() {
        let temp_dir = TempDir::new().unwrap();
        let key = create_test_key(&temp_dir, "test_key");

        let manager = BackupManager::new(temp_dir.path());
        let backup_path = temp_dir.path().join("backup.skm");

        manager
            .export(&[key], &backup_path, "correct", ExportOptions::default())
            .unwrap();

        let result = manager.import(&backup_path, "wrong", ImportOptions::default());
        assert!(result.is_err());
    }
}
