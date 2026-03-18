mod app;
mod ollama;
mod ui;
mod model;

use app::{App, Event};
use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

/// Initialize terminal
fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Exit terminal cleanly
fn exit_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

/// Setup raw mode for terminal
fn setup_raw_mode() -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    Ok(())
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize terminal
    let mut terminal = init_terminal()?;

    // Set up raw mode
    setup_raw_mode()?;

    // Create application
    let mut app = App::new();

    // Initialize application
    if let Err(e) = app.initialize().await {
        eprintln!("Failed to initialize: {}", e);
        return Err(e);
    }

    // Setup channel for events
    let (tx, mut rx) = mpsc::channel(100);

    // Clone tx for use in async context
    let tx_clone = tx.clone();

    // Clone app for use in main loop
    let mut app_main = app.clone();

    // Start event handling task (ticker only)
    let _handle = tokio::spawn(async move {
        let mut app_task = app.clone();
        let mut ticker = tokio::time::interval(Duration::from_secs(2));
        loop {
            ticker.tick().await;
            if let Err(_) = app_task.update_stats().await {
                app_task.error = Some("Failed to update stats".to_string());
            }
        }
    });

    // Terminal event loop
    loop {
        // Send tick event to background task
        tx.send(Event::Tick)
            .await
            .expect("Failed to send event");

        // Render
        terminal.draw(|frame| {
            app_main.render(frame);
        })?;

        // Check if we should exit (the async task will detect this)
        if app_main.error.as_deref() == Some("Exit requested") {
            break;
        }
    }

    // Clean exit
    drop(tx);
    let _ = exit_terminal(&mut terminal);

    Ok(())
}