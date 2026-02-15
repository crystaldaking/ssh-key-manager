use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "skm")]
#[command(about = "SSH Key Manager - TUI/CLI application for managing SSH keys")]
#[command(version)]
pub struct Cli {
    /// Path to SSH directory (default: ~/.ssh)
    #[arg(short, long, global = true)]
    pub ssh_dir: Option<PathBuf>,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    pub debug: bool,

    /// CLI mode - run command without TUI
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all SSH keys
    List {
        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Generate a new SSH key
    Generate {
        /// Key type (ed25519 or rsa)
        #[arg(short, long, value_enum, default_value = "ed25519")]
        key_type: KeyTypeArg,

        /// Key filename
        #[arg(short, long)]
        filename: Option<String>,

        /// Comment for the key
        #[arg(short, long)]
        comment: Option<String>,

        /// Passphrase for the key (use '-' for stdin)
        #[arg(short, long)]
        passphrase: Option<String>,

        /// Key bits (for RSA only)
        #[arg(short, long, default_value = "4096")]
        bits: u32,
    },

    /// Export keys to encrypted backup
    Export {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Passphrase for encryption (use '-' for stdin)
        #[arg(short, long)]
        passphrase: Option<String>,

        /// Export only specific keys (by name)
        #[arg(short, long)]
        keys: Vec<String>,

        /// Export public keys only (no private keys)
        #[arg(long)]
        public_only: bool,

        /// Description for the backup
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Import keys from encrypted backup
    Import {
        /// Backup file path
        #[arg(short, long)]
        file: PathBuf,

        /// Passphrase for decryption (use '-' for stdin)
        #[arg(short, long)]
        passphrase: Option<String>,

        /// Merge strategy when key exists
        #[arg(short, long, value_enum, default_value = "skip")]
        strategy: MergeStrategyArg,

        /// Dry run - show what would be imported without actually importing
        #[arg(long)]
        dry_run: bool,
    },

    /// Delete an SSH key
    Delete {
        /// Key name to delete
        name: String,

        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Show details of a specific key
    Show {
        /// Key name
        name: String,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Names,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum KeyTypeArg {
    Ed25519,
    Rsa,
}

impl KeyTypeArg {
    pub fn to_key_type(self) -> crate::ssh::keys::KeyType {
        match self {
            KeyTypeArg::Ed25519 => crate::ssh::keys::KeyType::Ed25519,
            KeyTypeArg::Rsa => crate::ssh::keys::KeyType::Rsa,
        }
    }

    pub fn default_filename(&self) -> &'static str {
        match self {
            KeyTypeArg::Ed25519 => "id_ed25519",
            KeyTypeArg::Rsa => "id_rsa",
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum MergeStrategyArg {
    /// Skip keys that already exist
    Skip,
    /// Overwrite existing keys
    Overwrite,
    /// Rename with timestamp suffix
    Rename,
}

impl MergeStrategyArg {
    pub fn to_merge_strategy(self) -> crate::crypto::backup::MergeStrategy {
        use crate::crypto::backup::MergeStrategy;
        match self {
            MergeStrategyArg::Skip => MergeStrategy::SkipExisting,
            MergeStrategyArg::Overwrite => MergeStrategy::Overwrite,
            MergeStrategyArg::Rename => MergeStrategy::Rename,
        }
    }
}

pub mod commands;
pub use commands::CliExecutor;
