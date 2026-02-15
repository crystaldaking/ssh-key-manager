use std::io::{self, Write};

use crate::cli::{Commands, KeyTypeArg, OutputFormat};
use crate::config::Config;
use crate::crypto::backup::{BackupManager, ExportOptions, ImportOptions};
use crate::error::Result;
use crate::ssh::KeyScanner;
use crate::ssh::generate::{KeyGenOptions, KeyGenerator};
use crate::ssh::keys::KeyType;

pub struct CliExecutor {
    config: Config,
}

impl CliExecutor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn execute(&self, command: Commands) -> Result<()> {
        match command {
            Commands::List { format } => self.cmd_list(format),
            Commands::Generate {
                key_type,
                filename,
                comment,
                passphrase,
                bits,
            } => self.cmd_generate(key_type, filename, comment, passphrase, bits),
            Commands::Export {
                output,
                passphrase,
                keys,
                public_only,
                description,
            } => self.cmd_export(output, passphrase, keys, public_only, description),
            Commands::Import {
                file,
                passphrase,
                strategy,
                dry_run,
            } => self.cmd_import(file, passphrase, strategy, dry_run),
            Commands::Delete { name, force } => self.cmd_delete(name, force),
            Commands::Show { name } => self.cmd_show(name),
            Commands::Copy { name, stdout, full } => self.cmd_copy(name, stdout, full),
        }
    }

    fn cmd_list(&self, format: OutputFormat) -> Result<()> {
        let scanner = KeyScanner::new(&self.config.ssh_dir);
        let keys = scanner.scan()?;

        match format {
            OutputFormat::Table => {
                if keys.is_empty() {
                    println!("No SSH keys found.");
                    return Ok(());
                }

                // Print header
                println!("{:<20} {:<10} {:<20} Comment", "Name", "Type", "Status");
                println!("{}", "-".repeat(70));

                // Print keys
                for key in keys {
                    let status = format!("{:?}", key.status);
                    let comment = key.comment.as_deref().unwrap_or("-");
                    println!(
                        "{:<20} {:<10} {:<20} {}",
                        key.name,
                        key.key_type.to_string(),
                        status,
                        comment
                    );
                }
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&keys)?;
                println!("{}", json);
            }
            OutputFormat::Names => {
                for key in keys {
                    println!("{}", key.name);
                }
            }
        }

        Ok(())
    }

    fn cmd_generate(
        &self,
        key_type: KeyTypeArg,
        filename: Option<String>,
        comment: Option<String>,
        passphrase: Option<String>,
        bits: u32,
    ) -> Result<()> {
        let generator = KeyGenerator::new(&self.config.ssh_dir);

        // Get filename
        let filename = filename.unwrap_or_else(|| key_type.default_filename().to_string());

        // Get comment
        let comment = comment.unwrap_or_else(|| {
            format!(
                "{}@{}",
                std::env::var("USER").unwrap_or_else(|_| "user".to_string()),
                get_hostname()
            )
        });

        // Handle passphrase from stdin if needed
        let passphrase = match passphrase.as_deref() {
            Some("-") => {
                read_passphrase_from_stdin("Enter passphrase (empty for no passphrase): ")?
            }
            Some(p) if !p.is_empty() => Some(p.to_string()),
            _ => None,
        };

        let key_type = key_type.to_key_type();
        let bits = if key_type == KeyType::Rsa {
            Some(bits)
        } else {
            None
        };

        let opts = KeyGenOptions {
            key_type,
            filename: filename.clone(),
            comment,
            passphrase,
            bits,
        };

        let key = generator.generate(opts)?;
        println!("Generated key: {}", key.name);
        println!("  Private: {}", key.path.display());
        println!("  Public:  {}", key.public_path.display());

        Ok(())
    }

    fn cmd_export(
        &self,
        output: std::path::PathBuf,
        passphrase: Option<String>,
        selected_keys: Vec<String>,
        public_only: bool,
        description: Option<String>,
    ) -> Result<()> {
        let scanner = KeyScanner::new(&self.config.ssh_dir);
        let keys = scanner.scan()?;

        if keys.is_empty() {
            eprintln!("No keys to export.");
            std::process::exit(1);
        }

        // Handle passphrase
        let passphrase =
            match passphrase.as_deref() {
                Some("-") => read_passphrase_from_stdin("Enter encryption passphrase: ")?
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Passphrase required")
                    })?,
                Some(p) => p.to_string(),
                None => read_passphrase_from_stdin("Enter encryption passphrase: ")?.ok_or_else(
                    || std::io::Error::new(std::io::ErrorKind::InvalidInput, "Passphrase required"),
                )?,
            };

        // Ensure parent directory exists
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let manager = BackupManager::new(&self.config.ssh_dir);
        let opts = ExportOptions {
            description,
            include_public_only: public_only,
            selected_keys: if selected_keys.is_empty() {
                None
            } else {
                Some(selected_keys)
            },
        };

        manager.export(&keys, &output, &passphrase, opts)?;
        println!("Exported {} keys to {}", keys.len(), output.display());

        Ok(())
    }

    fn cmd_import(
        &self,
        file: std::path::PathBuf,
        passphrase: Option<String>,
        strategy: crate::cli::MergeStrategyArg,
        dry_run: bool,
    ) -> Result<()> {
        if !file.exists() {
            eprintln!("Backup file not found: {}", file.display());
            std::process::exit(1);
        }

        // Handle passphrase
        let passphrase =
            match passphrase.as_deref() {
                Some("-") => read_passphrase_from_stdin("Enter decryption passphrase: ")?
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Passphrase required")
                    })?,
                Some(p) => p.to_string(),
                None => read_passphrase_from_stdin("Enter decryption passphrase: ")?.ok_or_else(
                    || std::io::Error::new(std::io::ErrorKind::InvalidInput, "Passphrase required"),
                )?,
            };

        let manager = BackupManager::new(&self.config.ssh_dir);
        let opts = ImportOptions {
            merge_strategy: strategy.to_merge_strategy(),
            dry_run,
        };

        let report = manager.import(&file, &passphrase, opts)?;

        if dry_run {
            println!("Dry run - would import:");
            println!("  {} keys to import", report.imported.len());
            for key in &report.imported {
                println!("    - {}", key);
            }
            if !report.skipped.is_empty() {
                println!("  {} keys to skip (already exist)", report.skipped.len());
                for key in &report.skipped {
                    println!("    - {}", key);
                }
            }
        } else {
            println!("Import complete:");
            println!("  Imported: {}", report.imported.len());
            println!("  Skipped: {}", report.skipped.len());
            println!("  Overwritten: {}", report.overwritten.len());
            if !report.errors.is_empty() {
                eprintln!("  Errors: {}", report.errors.len());
                for (key, err) in &report.errors {
                    eprintln!("    - {}: {}", key, err);
                }
            }
        }

        Ok(())
    }

    fn cmd_delete(&self, name: String, force: bool) -> Result<()> {
        let scanner = KeyScanner::new(&self.config.ssh_dir);

        let key = scanner
            .find_key_by_name(&name)?
            .ok_or_else(|| crate::error::SkmError::KeyNotFound(name.clone()))?;

        if !force {
            print!("Delete key '{}' and its public key? [y/N] ", name);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled.");
                return Ok(());
            }
        }

        // Delete private key if exists
        if key.path.exists() {
            std::fs::remove_file(&key.path)?;
        }

        // Delete public key if exists
        if key.public_path.exists() {
            std::fs::remove_file(&key.public_path)?;
        }

        println!("Deleted key: {}", name);
        Ok(())
    }

    fn cmd_show(&self, name: String) -> Result<()> {
        let scanner = KeyScanner::new(&self.config.ssh_dir);

        let key = scanner
            .find_key_by_name(&name)?
            .ok_or_else(|| crate::error::SkmError::KeyNotFound(name.clone()))?;

        println!("Name:        {}", key.name);
        println!("Type:        {}", key.key_type);
        println!("Status:      {:?}", key.status);
        println!("Private:     {}", key.path.display());
        println!("Public:      {}", key.public_path.display());
        println!(
            "Fingerprint: {}",
            key.fingerprint.as_deref().unwrap_or("N/A")
        );
        println!("Comment:     {}", key.comment.as_deref().unwrap_or("N/A"));
        println!(
            "Created:     {}",
            key.created_at
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        );
        println!(
            "Modified:    {}",
            key.modified_at
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        );

        // Show public key content if available
        if let Some(content) = key.read_public_content()? {
            println!("\nPublic key content:");
            println!("{}", content.trim());
        }

        Ok(())
    }

    fn cmd_copy(&self, name: String, stdout: bool, full: bool) -> Result<()> {
        use arboard::Clipboard;

        let scanner = KeyScanner::new(&self.config.ssh_dir);

        let key = scanner
            .find_key_by_name(&name)?
            .ok_or_else(|| crate::error::SkmError::KeyNotFound(name.clone()))?;

        // Get public key content
        let content = if full {
            key.read_public_content()?.ok_or_else(|| {
                crate::error::SkmError::KeyNotFound(format!("Public key for {}", name))
            })?
        } else {
            // Extract just the key part (without comment)
            let full_content = key.read_public_content()?.ok_or_else(|| {
                crate::error::SkmError::KeyNotFound(format!("Public key for {}", name))
            })?;

            // Parse "type key_base64 comment" -> "type key_base64"
            let parts: Vec<&str> = full_content.split_whitespace().collect();
            if parts.len() >= 2 {
                format!("{} {}", parts[0], parts[1])
            } else {
                full_content
            }
        };

        if stdout {
            // Output to stdout (for piping)
            println!("{}", content.trim());
        } else {
            // Copy to clipboard
            let mut clipboard = Clipboard::new().map_err(|e| {
                crate::error::SkmError::Unknown(format!("Failed to access clipboard: {}", e))
            })?;

            clipboard.set_text(content.trim()).map_err(|e| {
                crate::error::SkmError::Unknown(format!("Failed to copy to clipboard: {}", e))
            })?;

            println!("✓ Public key '{}' copied to clipboard!", name);
            println!(
                "  Fingerprint: {}",
                key.fingerprint.as_deref().unwrap_or("N/A")
            );
            if full {
                println!("  (Full key with comment)");
            } else {
                println!("  (Key only, without comment)");
            }
        }

        Ok(())
    }
}

fn read_passphrase_from_stdin(prompt: &str) -> io::Result<Option<String>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_string();
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h: std::ffi::OsString| h.into_string().ok())
        .unwrap_or_else(|| "localhost".to_string())
}
