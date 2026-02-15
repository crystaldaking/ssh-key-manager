use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::error::Result;
use crate::tui::app::{App, AppState, MessageType};

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
            app.state = AppState::CreateWizard;
            Ok(true)
        }
        KeyCode::Char('e') => {
            app.export_path = app.get_default_export_path().to_string_lossy().to_string();
            app.state = AppState::ExportDialog;
            Ok(true)
        }
        KeyCode::Char('i') => {
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
            app.refresh_keys()?;
            app.set_message("Keys refreshed", MessageType::Success);
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
            // TODO: Edit comment
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_create_wizard(_app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            // Return to list without creating
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_export_dialog(_app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Cancel export
            Ok(true)
        }
        KeyCode::Enter => {
            // Confirm export
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_import_dialog(_app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Cancel import
            Ok(true)
        }
        KeyCode::Enter => {
            // Confirm import
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
            app.confirm_delete = true;
            // TODO: Actually delete
            app.state = AppState::KeyList;
            Ok(true)
        }
        _ => Ok(true),
    }
}
