use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},

    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph,
    },
    Frame,
};

use crate::ssh::keys::KeyStatus;
use crate::tui::app::{App, AppState, MessageType};

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
        AppState::Quit => {}
    }
    
    draw_footer(f, app, chunks[2]);

    if app.show_help {
        draw_help_popup(f);
    }

    // Draw message overlay if present
    if let Some((ref msg, ref msg_type)) = app.message {
        draw_message(f, msg, *msg_type);
        // Clear message after drawing once
        app.clear_message();
    }
}

fn draw_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new("SSH Key Manager")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, area);
}

fn draw_key_list(f: &mut Frame, app: &App, area: Rect) {
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
        .block(Block::default().title("SSH Keys").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_symbol("> ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(if app.keys.is_empty() { None } else { Some(app.selected_index) });
    
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

fn draw_create_wizard(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title("Create New Key")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    
    let text = "Create Key Wizard\n\n\
                [1] ED25519 (Recommended)\n\
                [2] RSA (4096 bits)\n\n\
                Press number to select, ESC to cancel";
    
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_export_dialog(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Export Keys")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    
    let text = format!(
        "Export path: {}\n\n\
         Enter to confirm, ESC to cancel",
        app.export_path
    );
    
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_import_dialog(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title("Import Keys")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let text = "Import Key Backup\n\n\
                  Enter path to .skm file:\n\n\
                  Enter to confirm, ESC to cancel";
    
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
        AppState::CreateWizard => "ESC: Cancel",
        AppState::ExportDialog => "Enter: Confirm | ESC: Cancel",
        AppState::ImportDialog => "Enter: Confirm | ESC: Cancel",
        AppState::DeleteConfirm => "y: Yes | n: No",
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

    let paragraph = Paragraph::new(text)
        .block(
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
