mod app;
mod ollama;
mod ui;
mod model;

use app::App;
use ollama::OllamaClient;
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;

    let result = run(&mut terminal).await;

    // Always restore terminal, even on error
    let _ = restore_terminal(&mut terminal);

    result
}

async fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let app = Arc::new(Mutex::new(App::new()));

    {
        let mut a = app.lock().await;
        if let Err(e) = a.initialize().await {
            return Err(e);
        }
    }

    // Background task: fetch stats for the selected model every 2 seconds.
    // The network request is made *without* holding the lock so rendering
    // is never blocked by a slow HTTP call.
    let app_bg = app.clone();
    tokio::spawn(async move {
        let client = match OllamaClient::new() {
            Ok(c) => c,
            Err(e) => {
                let mut a = app_bg.lock().await;
                a.error = Some(format!("Client init failed: {}", e));
                return;
            }
        };

        let mut ticker = tokio::time::interval(Duration::from_secs(2));
        ticker.tick().await; // skip the immediate first tick

        loop {
            ticker.tick().await;

            let model_name = {
                let a = app_bg.lock().await;
                match a.selected_model_name() {
                    Some(n) => n.to_string(),
                    None => continue,
                }
            };

            match client.get_model_stats(&model_name).await {
                Ok(new_stats) => {
                    let mut a = app_bg.lock().await;
                    a.apply_stats(new_stats);
                }
                Err(e) => {
                    let mut a = app_bg.lock().await;
                    a.error = Some(format!("Failed to update stats: {}", e));
                }
            }
        }
    });

    // Main loop: render + handle keyboard input
    let mut events = EventStream::new();

    loop {
        {
            let mut a = app.lock().await;
            terminal.draw(|frame| a.render(frame))?;
        }

        // Wait up to 100 ms for the next terminal event so the render loop
        // stays responsive without burning CPU.
        let timeout = tokio::time::sleep(Duration::from_millis(100));
        tokio::select! {
            _ = timeout => {}
            maybe_event = events.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    // Only act on key-press, not release events (Windows sends both)
                    if key.kind == KeyEventKind::Press {
                        let mut a = app.lock().await;
                        if a.handle_key(key) {
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
