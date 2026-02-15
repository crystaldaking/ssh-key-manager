use crate::ssh::keys::KeyType;
use crate::ssh::generate::KeyGenOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    SelectType,
    EnterFilename,
    EnterComment,
    EnterPassphrase,
    Confirm,
}

#[derive(Debug, Clone)]
pub struct CreateWizard {
    pub step: WizardStep,
    pub options: KeyGenOptions,
    pub temp_filename: String,
    pub temp_comment: String,
    pub temp_passphrase: String,
    pub confirm_passphrase: String,
    pub error_message: Option<String>,
}

impl Default for CreateWizard {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateWizard {
    pub fn new() -> Self {
        Self {
            step: WizardStep::SelectType,
            options: KeyGenOptions::default(),
            temp_filename: "id_ed25519".to_string(),
            temp_comment: format!("{}@{}", get_username(), get_hostname()),
            temp_passphrase: String::new(),
            confirm_passphrase: String::new(),
            error_message: None,
        }
    }

    pub fn select_type(&mut self, key_type: KeyType) {
        self.options.key_type = key_type;
        self.temp_filename = key_type.default_filename().to_string();
        self.step = WizardStep::EnterFilename;
    }

    pub fn set_filename(&mut self, filename: &str) -> bool {
        if filename.is_empty() {
            self.error_message = Some("Filename cannot be empty".to_string());
            return false;
        }
        
        if filename.contains('/') || filename.contains('\\') {
            self.error_message = Some("Filename cannot contain path separators".to_string());
            return false;
        }

        self.options.filename = filename.to_string();
        self.step = WizardStep::EnterComment;
        self.error_message = None;
        true
    }

    pub fn set_comment(&mut self, comment: &str) {
        self.options.comment = if comment.is_empty() {
            format!("{}@{}", get_username(), get_hostname())
        } else {
            comment.to_string()
        };
        self.step = WizardStep::EnterPassphrase;
    }

    pub fn set_passphrase(&mut self, passphrase: &str, confirm: &str) -> bool {
        if !passphrase.is_empty() && passphrase != confirm {
            self.error_message = Some("Passphrases do not match".to_string());
            return false;
        }

        self.options.passphrase = if passphrase.is_empty() {
            None
        } else {
            Some(passphrase.to_string())
        };
        
        self.error_message = None;
        true
    }

    pub fn next_step(&mut self) {
        self.step = match self.step {
            WizardStep::SelectType => WizardStep::EnterFilename,
            WizardStep::EnterFilename => WizardStep::EnterComment,
            WizardStep::EnterComment => WizardStep::EnterPassphrase,
            WizardStep::EnterPassphrase => WizardStep::Confirm,
            WizardStep::Confirm => WizardStep::Confirm,
        };
    }

    pub fn previous_step(&mut self) {
        self.step = match self.step {
            WizardStep::SelectType => WizardStep::SelectType,
            WizardStep::EnterFilename => WizardStep::SelectType,
            WizardStep::EnterComment => WizardStep::EnterFilename,
            WizardStep::EnterPassphrase => WizardStep::EnterComment,
            WizardStep::Confirm => WizardStep::EnterPassphrase,
        };
    }

    pub fn get_options(self) -> KeyGenOptions {
        self.options
    }

    pub fn get_step_description(&self) -> &'static str {
        match self.step {
            WizardStep::SelectType => "Select key type",
            WizardStep::EnterFilename => "Enter filename",
            WizardStep::EnterComment => "Enter comment (optional)",
            WizardStep::EnterPassphrase => "Enter passphrase (optional)",
            WizardStep::Confirm => "Confirm settings",
        }
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Key Type: {}\n\
             Filename: {}\n\
             Comment: {}\n\
             Passphrase: {}",
            self.options.key_type,
            self.options.filename,
            self.options.comment,
            if self.options.passphrase.is_some() { "Yes" } else { "No" }
        )
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

    #[test]
    fn test_wizard_new() {
        let wizard = CreateWizard::new();
        assert!(matches!(wizard.step, WizardStep::SelectType));
        assert_eq!(wizard.options.key_type, KeyType::Ed25519);
    }

    #[test]
    fn test_select_type() {
        let mut wizard = CreateWizard::new();
        wizard.select_type(KeyType::Rsa);
        assert_eq!(wizard.options.key_type, KeyType::Rsa);
        assert_eq!(wizard.temp_filename, "id_rsa");
        assert!(matches!(wizard.step, WizardStep::EnterFilename));
    }

    #[test]
    fn test_set_filename_validation() {
        let mut wizard = CreateWizard::new();
        
        assert!(!wizard.set_filename(""));
        assert!(wizard.error_message.is_some());
        
        assert!(!wizard.set_filename("path/to/key"));
        assert!(wizard.error_message.is_some());
        
        assert!(wizard.set_filename("my_key"));
        assert!(wizard.error_message.is_none());
        assert_eq!(wizard.options.filename, "my_key");
    }

    #[test]
    fn test_passphrase_validation() {
        let mut wizard = CreateWizard::new();
        
        // Mismatched passphrases
        assert!(!wizard.set_passphrase("secret", "different"));
        assert!(wizard.error_message.is_some());
        
        // Matching passphrases
        assert!(wizard.set_passphrase("secret", "secret"));
        assert!(wizard.error_message.is_none());
        assert_eq!(wizard.options.passphrase, Some("secret".to_string()));
        
        // Empty passphrase (no encryption)
        assert!(wizard.set_passphrase("", ""));
        assert_eq!(wizard.options.passphrase, None);
    }

    #[test]
    fn test_step_navigation() {
        let mut wizard = CreateWizard::new();
        
        assert!(matches!(wizard.step, WizardStep::SelectType));
        
        wizard.next_step();
        assert!(matches!(wizard.step, WizardStep::EnterFilename));
        
        wizard.next_step();
        assert!(matches!(wizard.step, WizardStep::EnterComment));
        
        wizard.previous_step();
        assert!(matches!(wizard.step, WizardStep::EnterFilename));
    }
}
