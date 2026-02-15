use crate::config::Config;
use crate::error::Result;
use crate::ssh::{KeyScanner, SshKey};
use crate::tui::components::wizard::{CreateWizard, WizardStep};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    KeyList,
    KeyDetail,
    CreateWizard,
    ExportDialog,
    ImportDialog,
    DeleteConfirm,
    MessageDialog,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogState {
    EnterPath,
    EnterPassphrase,
    Confirm,
}

#[derive(Debug, Clone)]
pub struct App {
    pub state: AppState,
    pub config: Config,
    pub keys: Vec<SshKey>,
    pub selected_index: usize,
    pub selected_key: Option<SshKey>,
    pub message: Option<(String, MessageType, AppState)>, // (message, type, return_state)
    pub show_help: bool,
    
    // Wizard state
    pub wizard: Option<CreateWizard>,
    pub wizard_input: String,
    pub wizard_confirm_passphrase: String,
    
    // Dialog states
    pub export_path: String,
    pub import_path: String,
    pub dialog_passphrase: String,
    pub dialog_state: DialogState,
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
            wizard: None,
            wizard_input: String::new(),
            wizard_confirm_passphrase: String::new(),
            export_path: String::new(),
            import_path: String::new(),
            dialog_passphrase: String::new(),
            dialog_state: DialogState::EnterPath,
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

    pub fn set_message(&mut self, text: impl Into<String>, msg_type: MessageType, return_state: AppState) {
        self.message = Some((text.into(), msg_type, return_state));
        self.state = AppState::MessageDialog;
    }

    pub fn clear_message(&mut self) {
        if let Some((_, _, return_state)) = self.message {
            self.state = return_state;
        }
        self.message = None;
    }

    pub fn should_quit(&self) -> bool {
        matches!(self.state, AppState::Quit)
    }

    pub fn get_default_export_path(&self) -> PathBuf {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        self.config.export_dir.join(format!("ssh_backup_{}.skm", timestamp))
    }

    // Wizard methods
    pub fn start_wizard(&mut self) {
        self.wizard = Some(CreateWizard::new());
        self.wizard_input = String::new();
        self.wizard_confirm_passphrase = String::new();
    }

    pub fn get_wizard_step(&self) -> Option<WizardStep> {
        self.wizard.as_ref().map(|w| w.step)
    }

    pub fn wizard_next(&mut self) -> bool {
        if let Some(ref mut wizard) = self.wizard {
            match wizard.step {
                WizardStep::SelectType => {
                    // Handled separately with number keys
                    false
                }
                WizardStep::EnterFilename => {
                    if wizard.set_filename(&self.wizard_input) {
                        self.wizard_input.clear();
                        wizard.next_step();
                        true
                    } else {
                        false
                    }
                }
                WizardStep::EnterComment => {
                    wizard.set_comment(&self.wizard_input);
                    self.wizard_input.clear();
                    wizard.next_step();
                    true
                }
                WizardStep::EnterPassphrase => {
                    if wizard.set_passphrase(&self.wizard_input, &self.wizard_confirm_passphrase) {
                        wizard.next_step();
                        true
                    } else {
                        false
                    }
                }
                WizardStep::Confirm => {
                    true
                }
            }
        } else {
            false
        }
    }

    pub fn wizard_previous(&mut self) {
        if let Some(ref mut wizard) = self.wizard {
            wizard.previous_step();
            self.wizard_input.clear();
            self.wizard_confirm_passphrase.clear();
        }
    }

    pub fn wizard_select_type(&mut self, key_type: crate::ssh::keys::KeyType) {
        if let Some(ref mut wizard) = self.wizard {
            wizard.select_type(key_type);
            self.wizard_input = wizard.temp_filename.clone();
        }
    }

    pub fn get_wizard_options(&self) -> Option<crate::ssh::generate::KeyGenOptions> {
        self.wizard.as_ref().map(|w| w.options.clone())
    }

    pub fn get_wizard_error(&self) -> Option<String> {
        self.wizard.as_ref().and_then(|w| w.error_message.clone())
    }

    pub fn clear_wizard_error(&mut self) {
        if let Some(ref mut wizard) = self.wizard {
            wizard.error_message = None;
        }
    }

    pub fn end_wizard(&mut self) {
        self.wizard = None;
        self.wizard_input.clear();
        self.wizard_confirm_passphrase.clear();
    }

    // Dialog helper methods
    pub fn start_export(&mut self) {
        self.export_path = self.get_default_export_path().to_string_lossy().to_string();
        self.dialog_passphrase.clear();
        self.dialog_state = DialogState::EnterPath;
    }

    pub fn start_import(&mut self) {
        self.import_path.clear();
        self.dialog_passphrase.clear();
        self.dialog_state = DialogState::EnterPath;
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

    #[test]
    fn test_wizard_flow() {
        let config = create_test_config();
        let mut app = App::new(config).unwrap();
        
        app.start_wizard();
        assert!(app.wizard.is_some());
        
        // Select type
        app.wizard_select_type(crate::ssh::keys::KeyType::Ed25519);
        assert_eq!(app.get_wizard_step(), Some(WizardStep::EnterFilename));
        
        // Enter filename
        app.wizard_input = "test_key".to_string();
        assert!(app.wizard_next());
        assert_eq!(app.get_wizard_step(), Some(WizardStep::EnterComment));
        
        app.end_wizard();
        assert!(app.wizard.is_none());
    }
}
