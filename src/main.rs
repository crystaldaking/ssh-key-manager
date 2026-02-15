use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tracing::{error, info};

use ssh_key_manager::{
    Result,
    cli::{Cli, CliExecutor},
    config::Config,
    tui::{app::App, events::handle_events, ui::draw},
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    setup_logging(cli.debug)?;

    // Load configuration
    let config = if let Some(ref ssh_dir) = cli.ssh_dir {
        Config::from_ssh_dir(ssh_dir)?
    } else {
        Config::new()
    };

    // Ensure SSH directory exists
    config.ensure_ssh_dir()?;

    // Check if CLI command is provided
    if let Some(command) = cli.command {
        // CLI mode
        info!("Running in CLI mode");
        let executor = CliExecutor::new(config);

        match executor.execute(command) {
            Ok(()) => {
                info!("CLI command completed successfully");
                Ok(())
            }
            Err(e) => {
                error!("CLI command failed: {}", e);
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // TUI mode (default)
        info!("Starting SSH Key Manager in TUI mode");
        run_tui(config)
    }
}

fn run_tui(config: Config) -> Result<()> {
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

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
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
