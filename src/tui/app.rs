use crate::config::Config;
use crate::error::Result;
use crate::ssh::{KeyScanner, SshKey};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    KeyList,
    KeyDetail,
    CreateWizard,
    ExportDialog,
    ImportDialog,
    DeleteConfirm,
    Quit,
}

#[derive(Debug, Clone)]
pub struct App {
    pub state: AppState,
    pub config: Config,
    pub keys: Vec<SshKey>,
    pub selected_index: usize,
    pub selected_key: Option<SshKey>,
    pub message: Option<(String, MessageType)>,
    pub show_help: bool,
    // Dialog states
    pub export_path: String,
    pub import_path: String,
    pub passphrase_input: String,
    pub confirm_delete: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Success,
    Error,
    Info,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        let scanner = KeyScanner::new(&config.ssh_dir);
        let keys = scanner.scan()?;

        Ok(Self {
            state: AppState::KeyList,
            config,
            keys,
            selected_index: 0,
            selected_key: None,
            message: None,
            show_help: false,
            export_path: String::new(),
            import_path: String::new(),
            passphrase_input: String::new(),
            confirm_delete: false,
        })
    }

    pub fn refresh_keys(&mut self) -> Result<()> {
        let scanner = KeyScanner::new(&self.config.ssh_dir);
        self.keys = scanner.scan()?;
        
        // Adjust selected index if out of bounds
        if !self.keys.is_empty() && self.selected_index >= self.keys.len() {
            self.selected_index = self.keys.len() - 1;
        }
        
        Ok(())
    }

    pub fn next_key(&mut self) {
        if !self.keys.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.keys.len();
        }
    }

    pub fn previous_key(&mut self) {
        if !self.keys.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.keys.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn get_selected_key(&self) -> Option<&SshKey> {
        self.keys.get(self.selected_index)
    }

    pub fn select_key(&mut self, index: usize) {
        if index < self.keys.len() {
            self.selected_index = index;
        }
    }

    pub fn set_message(&mut self, text: impl Into<String>, msg_type: MessageType) {
        self.message = Some((text.into(), msg_type));
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn should_quit(&self) -> bool {
        matches!(self.state, AppState::Quit)
    }

    pub fn get_default_export_path(&self) -> PathBuf {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        self.config.export_dir.join(format!("ssh_backup_{}.skm", timestamp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> Config {
        let temp_dir = TempDir::new().unwrap();
        Config::from_ssh_dir(temp_dir.path()).unwrap()
    }

    #[test]
    fn test_app_new() {
        let config = create_test_config();
        let app = App::new(config).unwrap();
        assert!(matches!(app.state, AppState::KeyList));
    }

    #[test]
    fn test_navigation() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("key1"), "test").unwrap();
        std::fs::write(temp_dir.path().join("key2"), "test").unwrap();
        
        let config = Config::from_ssh_dir(temp_dir.path()).unwrap();
        let mut app = App::new(config).unwrap();
        
        assert_eq!(app.selected_index, 0);
        app.next_key();
        assert_eq!(app.selected_index, 1);
        app.next_key();
        assert_eq!(app.selected_index, 0);
        app.previous_key();
        assert_eq!(app.selected_index, 1);
    }
}
