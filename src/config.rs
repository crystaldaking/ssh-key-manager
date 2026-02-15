use directories::BaseDirs;
use std::path::{Path, PathBuf};

use crate::error::{Result, SkmError};

#[derive(Debug, Clone)]
pub struct Config {
    pub ssh_dir: PathBuf,
    pub export_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let home_dir = BaseDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("~"));

        let ssh_dir = home_dir.join(".ssh");
        let export_dir = home_dir.join(".skm");

        Self {
            ssh_dir,
            export_dir,
        }
    }

    pub fn from_ssh_dir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let ssh_dir = path.as_ref().to_path_buf();

        if !ssh_dir.exists() {
            return Err(SkmError::Config(format!(
                "SSH directory does not exist: {}",
                ssh_dir.display()
            )));
        }

        Ok(Self {
            ssh_dir,
            export_dir: Self::new().export_dir,
        })
    }

    pub fn ssh_dir_exists(&self) -> bool {
        self.ssh_dir.exists()
    }

    pub fn ensure_ssh_dir(&self) -> Result<()> {
        if !self.ssh_dir.exists() {
            std::fs::create_dir_all(&self.ssh_dir).map_err(SkmError::Io)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&self.ssh_dir)?.permissions();
                perms.set_mode(0o700);
                std::fs::set_permissions(&self.ssh_dir, perms)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::new();
        assert!(config.ssh_dir.to_string_lossy().contains(".ssh"));
    }

    #[test]
    fn test_from_ssh_dir() {
        let temp_dir = TempDir::new().unwrap();
        let ssh_dir = temp_dir.path().join(".ssh");
        std::fs::create_dir(&ssh_dir).unwrap();

        let config = Config::from_ssh_dir(&ssh_dir).unwrap();
        assert_eq!(config.ssh_dir, ssh_dir);
    }

    #[test]
    fn test_from_nonexistent_ssh_dir() {
        let temp_dir = TempDir::new().unwrap();
        let ssh_dir = temp_dir.path().join("nonexistent");

        let result = Config::from_ssh_dir(&ssh_dir);
        assert!(result.is_err());
    }
}
