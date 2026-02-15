use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::error::Result;
use crate::ssh::generate::KeyGenerator;
use crate::ssh::keys::KeyType;

use crate::tui::app::{App, AppState, DialogState, MessageType};
use crate::crypto::backup::{BackupManager, ExportOptions, ImportOptions, MergeStrategy};

pub fn handle_events(app: &mut App) -> Result<bool> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            return handle_key_event(app, key);
        }
    }
    Ok(false)
}

fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<bool> {
    // Global shortcuts
    if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.state = AppState::Quit;
        return Ok(true);
    }

    if key.code == KeyCode::Char('h') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.show_help = !app.show_help;
        return Ok(true);
    }

    // State-specific handling
    match app.state {
        AppState::KeyList => handle_key_list(app, key),
        AppState::KeyDetail => handle_key_detail(app, key),
        AppState::CreateWizard => handle_create_wizard(app, key),
        AppState::ExportDialog => handle_export_dialog(app, key),
        AppState::ImportDialog => handle_import_dialog(app, key),
        AppState::DeleteConfirm => handle_delete_confirm(app, key),
        AppState::MessageDialog => handle_message_dialog(app, key),
        AppState::Quit => Ok(true),
    }
}

fn handle_key_list(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.state = AppState::Quit;
            Ok(true)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_key();
            Ok(true)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous_key();
            Ok(true)
        }
        KeyCode::Enter => {
            if let Some(key) = app.get_selected_key() {
                app.selected_key = Some(key.clone());
                app.state = AppState::KeyDetail;
            }
            Ok(true)
        }
        KeyCode::Char('n') => {
            app.start_wizard();
            app.state = AppState::CreateWizard;
            Ok(true)
        }
        KeyCode::Char('e') => {
            app.start_export();
            app.state = AppState::ExportDialog;
            Ok(true)
        }
        KeyCode::Char('i') => {
            app.start_import();
            app.state = AppState::ImportDialog;
            Ok(true)
        }
        KeyCode::Char('d') => {
            if app.get_selected_key().is_some() {
                app.confirm_delete = false;
                app.state = AppState::DeleteConfirm;
            }
            Ok(true)
        }
        KeyCode::Char('r') => {
            match app.refresh_keys() {
                Ok(()) => app.set_message("Keys refreshed", MessageType::Success, AppState::KeyList),
                Err(e) => app.set_message(format!("Error: {}", e), MessageType::Error, AppState::KeyList),
            }
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_key_detail(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.state = AppState::KeyList;
            app.selected_key = None;
            Ok(true)
        }
        KeyCode::Char('c') => {
            // TODO: Edit comment - would need an input dialog
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_create_wizard(app: &mut App, key: KeyEvent) -> Result<bool> {
    use crate::tui::components::wizard::WizardStep;

    let current_step = app.get_wizard_step().unwrap_or(WizardStep::SelectType);

    match key.code {
        KeyCode::Esc => {
            app.end_wizard();
            app.state = AppState::KeyList;
            Ok(true)
        }
        KeyCode::Backspace => {
            app.wizard_input.pop();
            Ok(true)
        }
        KeyCode::Enter => {
            app.clear_wizard_error();
            match current_step {
                WizardStep::SelectType => {
                    // Handled by number keys
                }
                WizardStep::EnterFilename | WizardStep::EnterComment => {
                    if !app.wizard_next() {
                        if let Some(err) = app.get_wizard_error() {
                            app.set_message(err, MessageType::Error, AppState::CreateWizard);
                        }
                    }
                }
                WizardStep::EnterPassphrase => {
                    // Store passphrase and move to confirmation
                    if !app.wizard_next() {
                        if let Some(err) = app.get_wizard_error() {
                            app.set_message(err, MessageType::Error, AppState::CreateWizard);
                        }
                    }
                }
                WizardStep::Confirm => {
                    // Generate the key
                    if let Some(options) = app.get_wizard_options() {
                        let generator = KeyGenerator::new(&app.config.ssh_dir);
                        match generator.generate(options) {
                            Ok(_) => {
                                app.refresh_keys()?;
                                app.end_wizard();
                                app.set_message("Key created successfully", MessageType::Success, AppState::KeyList);
                            }
                            Err(e) => {
                                app.set_message(format!("Failed to create key: {}", e), MessageType::Error, AppState::CreateWizard);
                            }
                        }
                    }
                }
            }
            Ok(true)
        }
        KeyCode::Char(c) => {
            app.clear_wizard_error();
            match current_step {
                WizardStep::SelectType => {
                    match c {
                        '1' => app.wizard_select_type(KeyType::Ed25519),
                        '2' => app.wizard_select_type(KeyType::Rsa),
                        _ => {}
                    }
                }
                WizardStep::EnterPassphrase if c == '\t' => {
                    // Tab to switch between passphrase and confirm
                }
                _ => {
                    app.wizard_input.push(c);
                }
            }
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_export_dialog(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.state = AppState::KeyList;
            Ok(true)
        }
        KeyCode::Enter => {
            match app.dialog_state {
                DialogState::EnterPath => {
                    app.dialog_state = DialogState::EnterPassphrase;
                    Ok(true)
                }
                DialogState::EnterPassphrase => {
                    app.dialog_state = DialogState::Confirm;
                    Ok(true)
                }
                DialogState::Confirm => {
                    // Perform export
                    let manager = BackupManager::new(&app.config.ssh_dir);
                    let opts = ExportOptions {
                        description: Some(format!("Backup from {}", chrono::Local::now().format("%Y-%m-%d"))),
                        include_public_only: false,
                        selected_keys: None,
                    };
                    
                    let path = std::path::PathBuf::from(&app.export_path);
                    
                    // Ensure parent directory exists
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                    
                    match manager.export(&app.keys, &path, &app.dialog_passphrase, opts) {
                        Ok(()) => {
                            app.set_message(
                                format!("Exported {} keys to {}", app.keys.len(), app.export_path),
                                MessageType::Success,
                                AppState::KeyList
                            );
                        }
                        Err(e) => {
                            app.set_message(format!("Export failed: {}", e), MessageType::Error, AppState::KeyList);
                        }
                    }
                    Ok(true)
                }
            }
        }
        KeyCode::Backspace => {
            match app.dialog_state {
                DialogState::EnterPath => {
                    app.export_path.pop();
                }
                DialogState::EnterPassphrase => {
                    app.dialog_passphrase.pop();
                }
                _ => {}
            }
            Ok(true)
        }
        KeyCode::Char(c) => {
            match app.dialog_state {
                DialogState::EnterPath => {
                    app.export_path.push(c);
                }
                DialogState::EnterPassphrase => {
                    app.dialog_passphrase.push(c);
                }
                _ => {}
            }
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_import_dialog(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.state = AppState::KeyList;
            Ok(true)
        }
        KeyCode::Enter => {
            match app.dialog_state {
                DialogState::EnterPath => {
                    app.dialog_state = DialogState::EnterPassphrase;
                    Ok(true)
                }
                DialogState::EnterPassphrase => {
                    app.dialog_state = DialogState::Confirm;
                    Ok(true)
                }
                DialogState::Confirm => {
                    // Perform import
                    let manager = BackupManager::new(&app.config.ssh_dir);
                    let opts = ImportOptions {
                        merge_strategy: MergeStrategy::SkipExisting,
                        dry_run: false,
                    };
                    
                    let path = std::path::PathBuf::from(&app.import_path);
                    
                    match manager.import(&path, &app.dialog_passphrase, opts) {
                        Ok(report) => {
                            app.refresh_keys()?;
                            let msg = format!(
                                "Import complete: {} imported, {} skipped, {} overwritten",
                                report.imported.len(),
                                report.skipped.len(),
                                report.overwritten.len()
                            );
                            app.set_message(msg, MessageType::Success, AppState::KeyList);
                        }
                        Err(e) => {
                            app.set_message(format!("Import failed: {}", e), MessageType::Error, AppState::KeyList);
                        }
                    }
                    Ok(true)
                }
            }
        }
        KeyCode::Backspace => {
            match app.dialog_state {
                DialogState::EnterPath => {
                    app.import_path.pop();
                }
                DialogState::EnterPassphrase => {
                    app.dialog_passphrase.pop();
                }
                _ => {}
            }
            Ok(true)
        }
        KeyCode::Char(c) => {
            match app.dialog_state {
                DialogState::EnterPath => {
                    app.import_path.push(c);
                }
                DialogState::EnterPassphrase => {
                    app.dialog_passphrase.push(c);
                }
                _ => {}
            }
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_delete_confirm(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('n') => {
            app.confirm_delete = false;
            app.state = AppState::KeyList;
            Ok(true)
        }
        KeyCode::Char('y') => {
            if let Some(key) = app.get_selected_key().cloned() {
                let private_deleted = std::fs::remove_file(&key.path).is_ok();
                let public_deleted = std::fs::remove_file(&key.public_path).is_ok();
                
                if private_deleted || public_deleted {
                    app.refresh_keys()?;
                    app.set_message(format!("Deleted key '{}'", key.name), MessageType::Success, AppState::KeyList);
                } else {
                    app.set_message(format!("Failed to delete key '{}'", key.name), MessageType::Error, AppState::KeyList);
                }
            }
            app.confirm_delete = false;
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_message_dialog(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Enter | KeyCode::Esc => {
            app.clear_message();
            Ok(true)
        }
        _ => Ok(true),
    }
}
