use std::fs;
use tempfile::TempDir;

use ssh_key_manager::{
    config::Config,
    crypto::backup::{BackupManager, ExportOptions, ImportOptions, MergeStrategy},
    ssh::{
        KeyScanner,
        generate::{KeyGenOptions, KeyGenerator},
        keys::KeyType,
    },
};

#[test]
fn test_generate_and_scan_key() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config::from_ssh_dir(temp_dir.path()).unwrap();

    // Generate a key
    let generator = KeyGenerator::new(&config.ssh_dir);
    let opts = KeyGenOptions {
        key_type: KeyType::Ed25519,
        filename: "test_key".to_string(),
        comment: "test@example.com".to_string(),
        passphrase: None,
        bits: None,
    };

    let key = generator.generate(opts).unwrap();
    assert_eq!(key.name, "test_key");
    assert!(key.path.exists());
    assert!(key.public_path.exists());

    // Scan and find the key
    let scanner = KeyScanner::new(&config.ssh_dir);
    let keys = scanner.scan().unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].name, "test_key");
}

#[test]
fn test_export_import_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let export_dir = TempDir::new().unwrap();
    let config = Config::from_ssh_dir(temp_dir.path()).unwrap();

    // Create test key
    let generator = KeyGenerator::new(&config.ssh_dir);
    let opts = KeyGenOptions {
        key_type: KeyType::Ed25519,
        filename: "backup_test".to_string(),
        comment: "backup test".to_string(),
        passphrase: None,
        bits: None,
    };
    generator.generate(opts).unwrap();

    let scanner = KeyScanner::new(&config.ssh_dir);
    let keys = scanner.scan().unwrap();

    // Export
    let manager = BackupManager::new(&config.ssh_dir);
    let backup_path = export_dir.path().join("backup.skm");

    let export_opts = ExportOptions {
        description: Some("Test backup".to_string()),
        include_public_only: false,
        selected_keys: None,
    };

    manager
        .export(&keys, &backup_path, "test_passphrase", export_opts)
        .unwrap();

    assert!(backup_path.exists());

    // Import to new location
    let import_dir = TempDir::new().unwrap();
    let import_config = Config::from_ssh_dir(import_dir.path()).unwrap();
    let import_manager = BackupManager::new(&import_config.ssh_dir);

    let import_opts = ImportOptions {
        merge_strategy: MergeStrategy::SkipExisting,
        dry_run: false,
    };

    let report = import_manager
        .import(&backup_path, "test_passphrase", import_opts)
        .unwrap();

    assert_eq!(report.imported.len(), 1);
    assert!(import_dir.path().join("backup_test").exists());
}

#[test]
fn test_generate_multiple_key_types() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config::from_ssh_dir(temp_dir.path()).unwrap();
    let generator = KeyGenerator::new(&config.ssh_dir);

    // Generate ED25519 key
    let ed25519_opts = KeyGenOptions {
        key_type: KeyType::Ed25519,
        filename: "id_ed25519".to_string(),
        comment: "ed25519 key".to_string(),
        passphrase: None,
        bits: None,
    };
    let key1 = generator.generate(ed25519_opts).unwrap();
    assert_eq!(key1.key_type, KeyType::Ed25519);

    // List keys
    let scanner = KeyScanner::new(&config.ssh_dir);
    let keys = scanner.scan().unwrap();
    assert_eq!(keys.len(), 1);
}

#[test]
fn test_import_wrong_passphrase() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config::from_ssh_dir(temp_dir.path()).unwrap();

    // Create a key
    let generator = KeyGenerator::new(&config.ssh_dir);
    let opts = KeyGenOptions {
        key_type: KeyType::Ed25519,
        filename: "pass_test".to_string(),
        comment: "pass test".to_string(),
        passphrase: None,
        bits: None,
    };
    generator.generate(opts).unwrap();

    let scanner = KeyScanner::new(&config.ssh_dir);
    let keys = scanner.scan().unwrap();

    // Export
    let backup_path = temp_dir.path().join("pass.skm");
    let manager = BackupManager::new(&config.ssh_dir);
    manager
        .export(&keys, &backup_path, "correct", ExportOptions::default())
        .unwrap();

    // Try import with wrong passphrase
    let import_opts = ImportOptions::default();
    let result = manager.import(&backup_path, "wrong", import_opts);
    assert!(result.is_err());
}

#[test]
fn test_key_not_found_error() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config::from_ssh_dir(temp_dir.path()).unwrap();

    let scanner = KeyScanner::new(&config.ssh_dir);
    let result = scanner.find_key_by_name("nonexistent");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_invalid_backup_file() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config::from_ssh_dir(temp_dir.path()).unwrap();

    // Create invalid backup file
    let invalid_backup = temp_dir.path().join("invalid.skm");
    fs::write(&invalid_backup, "not a valid backup").unwrap();

    let manager = BackupManager::new(&config.ssh_dir);
    let import_opts = ImportOptions::default();
    let result = manager.import(&invalid_backup, "pass", import_opts);
    assert!(result.is_err());
}
