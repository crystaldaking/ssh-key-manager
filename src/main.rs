use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tracing::{error, info};

use ssh_key_manager::{
    config::Config,
    tui::{app::App, events::handle_events, ui::draw},
    Result,
};

#[derive(Parser, Debug)]
#[command(name = "skm")]
#[command(about = "SSH Key Manager - TUI application for managing SSH keys")]
#[command(version)]
struct Cli {
    /// Path to SSH directory (default: ~/.ssh)
    #[arg(short, long)]
    ssh_dir: Option<std::path::PathBuf>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    setup_logging(cli.debug)?;

    info!("Starting SSH Key Manager");

    // Load configuration
    let config = if let Some(ssh_dir) = cli.ssh_dir {
        Config::from_ssh_dir(ssh_dir)?
    } else {
        Config::new()
    };

    // Ensure SSH directory exists
    config.ensure_ssh_dir()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(config)?;

    // Main event loop
    let result = run_app(&mut terminal, &mut app);

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle result
    match result {
        Ok(()) => {
            info!("Application exited normally");
            Ok(())
        }
        Err(e) => {
            error!("Application error: {}", e);
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    let mut last_tick = std::time::Instant::now();
    let tick_rate = std::time::Duration::from_millis(250);

    loop {
        // Draw UI
        terminal.draw(|f| draw(f, app))?;

        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| std::time::Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if handle_events(app)? {
                if app.should_quit() {
                    break;
                }
            }
        }

        // Handle tick events
        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }

    Ok(())
}

fn setup_logging(debug: bool) -> Result<()> {
    let level = if debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| ssh_key_manager::SkmError::Unknown(e.to_string()))?;

    Ok(())
}
