use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::Result;
use crate::ssh::keys::SshKey;

pub struct KeyScanner {
    ssh_dir: PathBuf,
}

impl KeyScanner {
    pub fn new<P: AsRef<Path>>(ssh_dir: P) -> Self {
        Self {
            ssh_dir: ssh_dir.as_ref().to_path_buf(),
        }
    }

    pub fn scan(&self) -> Result<Vec<SshKey>> {
        if !self.ssh_dir.exists() {
            return Ok(Vec::new());
        }

        let mut keys = Vec::new();
        let mut processed = std::collections::HashSet::new();

        for entry in WalkDir::new(&self.ssh_dir)
            .max_depth(1)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip public key files (we'll pair them with private keys)
            if file_name.ends_with(".pub") {
                continue;
            }

            // Skip known non-key files
            if Self::is_non_key_file(file_name) {
                continue;
            }

            // Skip if already processed (handles symlinks)
            let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            if !processed.insert(canonical) {
                continue;
            }

            match SshKey::from_path(path) {
                Ok(key) => keys.push(key),
                Err(e) => {
                    tracing::warn!("Failed to parse key {}: {}", path.display(), e);
                }
            }
        }

        // Sort by name for consistent display
        keys.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(keys)
    }

    fn is_non_key_file(filename: &str) -> bool {
        const NON_KEY_FILES: &[&str] = &[
            "authorized_keys",
            "authorized_keys2",
            "known_hosts",
            "known_hosts.old",
            "config",
            "agent",
        ];

        NON_KEY_FILES.iter().any(|&pattern| {
            if pattern == "agent" {
                filename.starts_with("agent.")
            } else {
                filename == pattern
            }
        })
    }

    pub fn find_key_by_name(&self, name: &str) -> Result<Option<SshKey>> {
        let keys = self.scan()?;
        Ok(keys.into_iter().find(|k| k.name == name))
    }

    pub fn get_key_count(&self) -> Result<usize> {
        self.scan().map(|keys| keys.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scan_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let scanner = KeyScanner::new(temp_dir.path());
        let keys = scanner.scan().unwrap();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_scan_with_keys() {
        let temp_dir = TempDir::new().unwrap();

        // Create test keys
        std::fs::write(temp_dir.path().join("id_rsa"), "private").unwrap();
        std::fs::write(temp_dir.path().join("id_rsa.pub"), "public").unwrap();
        std::fs::write(temp_dir.path().join("id_ed25519"), "private").unwrap();
        std::fs::write(temp_dir.path().join("id_ed25519.pub"), "public").unwrap();

        let scanner = KeyScanner::new(temp_dir.path());
        let keys = scanner.scan().unwrap();

        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_skip_non_key_files() {
        let temp_dir = TempDir::new().unwrap();

        std::fs::write(temp_dir.path().join("id_rsa"), "private").unwrap();
        std::fs::write(temp_dir.path().join("known_hosts"), "hosts").unwrap();
        std::fs::write(temp_dir.path().join("config"), "config").unwrap();

        let scanner = KeyScanner::new(temp_dir.path());
        let keys = scanner.scan().unwrap();

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].name, "id_rsa");
    }

    #[test]
    fn test_find_key_by_name() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("my_key"), "private").unwrap();

        let scanner = KeyScanner::new(temp_dir.path());
        let key = scanner.find_key_by_name("my_key").unwrap();

        assert!(key.is_some());
        assert_eq!(key.unwrap().name, "my_key");
    }
}
