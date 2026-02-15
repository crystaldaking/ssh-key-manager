use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph},
};

use crate::ssh::keys::KeyStatus;
use crate::tui::app::{App, AppState, DialogState, MessageType};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_header(f, chunks[0]);

    match app.state {
        AppState::KeyList => draw_key_list(f, app, chunks[1]),
        AppState::KeyDetail => draw_key_detail(f, app, chunks[1]),
        AppState::CreateWizard => draw_create_wizard(f, app, chunks[1]),
        AppState::ExportDialog => draw_export_dialog(f, app, chunks[1]),
        AppState::ImportDialog => draw_import_dialog(f, app, chunks[1]),
        AppState::DeleteConfirm => draw_delete_confirm(f, app, chunks[1]),
        AppState::MessageDialog => {
            draw_key_list(f, app, chunks[1]);
            if let Some((ref msg, ref msg_type, _)) = app.message {
                draw_message(f, msg, *msg_type);
            }
        }
        AppState::Quit => {}
    }

    draw_footer(f, app, chunks[2]);

    if app.show_help {
        draw_help_popup(f);
    }
}

fn draw_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new("SSH Key Manager (skm)")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, area);
}

fn draw_key_list(f: &mut Frame, app: &App, area: Rect) {
    if app.keys.is_empty() {
        let paragraph = Paragraph::new("No SSH keys found.\n\nPress 'n' to create a new key.")
            .block(Block::default().title("SSH Keys").borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .keys
        .iter()
        .map(|key| {
            let status_symbol = match key.status {
                KeyStatus::Valid => "[OK]",
                KeyStatus::Encrypted => "[LOCKED]",
                _ => "[!]",
            };

            let content = format!(
                " {} {} - {} [{}]",
                status_symbol,
                key.name,
                key.key_type,
                key.comment.as_deref().unwrap_or("no comment")
            );

            ListItem::new(content).style(Style::default())
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!("SSH Keys ({})", app.keys.len()))
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_symbol("> ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected_index));

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_key_detail(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ref key) = app.selected_key {
        let text = format!(
            "Name: {}\n\
             Type: {}\n\
             Status: {}\n\
             Path: {}\n\
             Public Path: {}\n\
             Fingerprint: {}\n\
             Comment: {}\n\
             Created: {}\n\
             Modified: {}",
            key.name,
            key.key_type,
            key.status,
            key.path.display(),
            key.public_path.display(),
            key.fingerprint.as_deref().unwrap_or("N/A"),
            key.comment.as_deref().unwrap_or("N/A"),
            key.created_at
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            key.modified_at
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
        );

        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Key Details").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

fn draw_create_wizard(f: &mut Frame, app: &App, area: Rect) {
    use crate::tui::components::wizard::WizardStep;

    let wizard = match &app.wizard {
        Some(w) => w,
        None => return,
    };

    let (title, content) = match wizard.step {
        WizardStep::SelectType => (
            "Create New Key - Step 1/5",
            "Select key type:\n\n\
             [1] ED25519 (Recommended - modern, fast, secure)\n\
             [2] RSA (4096 bits - for legacy compatibility)\n\n\
             Press 1 or 2 to select, ESC to cancel"
                .to_string(),
        ),
        WizardStep::EnterFilename => (
            "Create New Key - Step 2/5",
            format!(
                "Enter filename for the key:\n\n\
                 > {}\n\n\
                 Press Enter to continue, ESC to go back",
                app.wizard_input
            ),
        ),
        WizardStep::EnterComment => (
            "Create New Key - Step 3/5",
            format!(
                "Enter comment (or leave empty for default):\n\n\
                 > {}\n\n\
                 Default: {}\n\
                 Press Enter to continue, ESC to go back",
                app.wizard_input, wizard.options.comment
            ),
        ),
        WizardStep::EnterPassphrase => (
            "Create New Key - Step 4/5",
            format!(
                "Enter passphrase (or leave empty for no passphrase):\n\n\
                 > {}\n\n\
                 Press Enter to continue, ESC to go back",
                "*".repeat(app.wizard_input.len())
            ),
        ),
        WizardStep::Confirm => (
            "Create New Key - Step 5/5",
            format!(
                "Please confirm:\n\n\
                 {}\n\n\
                 Press Enter to create, ESC to go back",
                wizard.get_summary()
            ),
        ),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

fn draw_export_dialog(f: &mut Frame, app: &App, area: Rect) {
    let (title, prompt, value) = match app.dialog_state {
        DialogState::EnterPath => (
            "Export Keys - Path",
            "Enter export path:",
            app.export_path.clone(),
        ),
        DialogState::EnterPassphrase => (
            "Export Keys - Passphrase",
            "Enter encryption passphrase:",
            "*".repeat(app.dialog_passphrase.len()),
        ),
        DialogState::Confirm => (
            "Export Keys - Confirm",
            "Press Enter to export or ESC to cancel",
            format!("Path: {} | Keys: {}", app.export_path, app.keys.len()),
        ),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let text = format!("{}\n\n> {}", prompt, value);
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_import_dialog(f: &mut Frame, app: &App, area: Rect) {
    let (title, prompt, value) = match app.dialog_state {
        DialogState::EnterPath => (
            "Import Keys - Path",
            "Enter path to .skm file:",
            app.import_path.clone(),
        ),
        DialogState::EnterPassphrase => (
            "Import Keys - Passphrase",
            "Enter decryption passphrase:",
            "*".repeat(app.dialog_passphrase.len()),
        ),
        DialogState::Confirm => (
            "Import Keys - Confirm",
            "Press Enter to import or ESC to cancel",
            format!("Path: {}", app.import_path),
        ),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let text = format!("{}\n\n> {}", prompt, value);
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_delete_confirm(f: &mut Frame, app: &App, area: Rect) {
    let name = app
        .get_selected_key()
        .map(|k| k.name.as_str())
        .unwrap_or("selected key");

    let text = format!(
        "Are you sure you want to delete '{}'?\n\n\
         This action cannot be undone!\n\n\
         [y] Yes, delete\n\
         [n] No, cancel",
        name
    );

    let block = Block::default()
        .title("Confirm Delete")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.state {
        AppState::KeyList => {
            "j/k or ↑/↓: Navigate | Enter: Details | n: New | e: Export | i: Import | d: Delete | r: Refresh | q: Quit"
        }
        AppState::KeyDetail => "ESC: Back | c: Edit Comment",
        AppState::CreateWizard => "ESC: Cancel | Enter: Continue",
        AppState::ExportDialog => "Enter: Continue | ESC: Cancel",
        AppState::ImportDialog => "Enter: Continue | ESC: Cancel",
        AppState::DeleteConfirm => "y: Yes | n: No",
        AppState::MessageDialog => "Enter/ESC: OK",
        AppState::Quit => "",
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));

    f.render_widget(footer, area);
}

fn draw_help_popup(f: &mut Frame) {
    let text = "SSH Key Manager Help\n\n\
                  Global Shortcuts:\n\
                  Ctrl+H - Toggle this help\n\
                  Ctrl+Q - Quit application\n\n\
                  Navigation:\n\
                  j or ↓ - Move down\n\
                  k or ↑ - Move up\n\
                  Enter - Select/Confirm\n\
                  ESC - Cancel/Back\n\n\
                  Key List:\n\
                  n - Create new key\n\
                  e - Export keys\n\
                  i - Import keys\n\
                  d - Delete selected key\n\
                  r - Refresh list";

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title("Help")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn draw_message(f: &mut Frame, msg: &str, msg_type: MessageType) {
    let color = match msg_type {
        MessageType::Success => Color::Green,
        MessageType::Error => Color::Red,
        MessageType::Info => Color::Blue,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));

    let paragraph = Paragraph::new(msg)
        .block(block)
        .alignment(Alignment::Center);

    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
