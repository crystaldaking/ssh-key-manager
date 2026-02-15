use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkmError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSH key error: {0}")]
    SshKey(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Invalid passphrase")]
    InvalidPassphrase,

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Key already exists: {0}")]
    KeyAlreadyExists(String),

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Import/Export error: {0}")]
    ImportExport(String),

    #[error("TUI error: {0}")]
    Tui(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, SkmError>;
