use crate::ollama::OllamaClient;
use crate::ui::OllamaStatsUI;
use crate::model::stats::{ModelStats, OllamaModel};
use anyhow::Result;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Duration;
use std::clone::Clone;

/// Application state
#[derive(Clone)]
pub struct App {
    /// List of available models
    models: Vec<OllamaModel>,
    /// Selected model index
    selected_model: usize,
    /// Model stats for the selected model
    stats: Vec<ModelStats>,
    /// UI components
    stats_ui: OllamaStatsUI,
    /// Last error message
    pub error: Option<String>,
    /// Loading state
    loading: bool,
}

impl App {
    /// Create a new application
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            selected_model: 0,
            stats: Vec::new(),
            stats_ui: OllamaStatsUI::new(),
            error: None,
            loading: false,
        }
    }

    /// Initialize the application by fetching models
    pub async fn initialize(&mut self) -> Result<()> {
        self.loading = true;
        self.error = None;

        let client = OllamaClient::new()?;

        match client.list_models().await {
            Ok(models) => {
                self.models = models;
                self.loading = false;

                // Fetch stats for all models
                for model in &self.models {
                    match client.get_model_stats(&model.name).await {
                        Ok(stats) => {
                            self.stats.push(stats);
                        }
                        Err(e) => {
                            eprintln!("Failed to get stats for {}: {}", model.name, e);
                        }
                    }
                }
            }
            Err(e) => {
                self.error = Some(format!("Failed to fetch models: {}", e));
                self.loading = false;
            }
        }

        Ok(())
    }

    /// Update stats for the selected model
    pub async fn update_stats(&mut self) -> Result<()> {
        if self.models.is_empty() {
            return Ok(());
        }

        let client = OllamaClient::new()?;
        let model_name = &self.models[self.selected_model].name;

        match client.get_model_stats(model_name).await {
            Ok(stats) => {
                // Update the stats for this model
                if let Some(existing) = self.stats.iter_mut().find(|s| s.name == *model_name) {
                    *existing = stats;
                } else {
                    self.stats.push(stats);
                }
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Failed to update stats: {}", e));
            }
        }

        Ok(())
    }

    /// Navigate to the previous model
    pub fn previous_model(&mut self) {
        if self.models.is_empty() {
            return;
        }
        self.selected_model = self.selected_model.saturating_sub(1);
    }

    /// Navigate to the next model
    pub fn next_model(&mut self) {
        if self.models.is_empty() {
            return;
        }
        self.selected_model = (self.selected_model + 1) % self.models.len();
    }

    /// Render the application
    pub fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(frame.area());

        // Render header
        self.render_header(frame, layout[0]);

        // Render main content
        if self.models.is_empty() {
            self.render_empty_state(frame, layout[1]);
        } else {
            self.render_stats(frame, layout[1]);
        }
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title = Span::styled(
            "Ollama Top",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        );
        let status = if self.loading {
            Span::styled(" (Loading...)", Style::default().fg(Color::Yellow))
        } else if self.error.is_some() {
            Span::styled(" (Error)", Style::default().fg(Color::Red))
        } else {
            Span::styled(" (Ready)", Style::default().fg(Color::Green))
        };

        let line = Line::from(vec![title, status]);
        let paragraph = Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_empty_state(&self, frame: &mut Frame, area: Rect) {
        let error_msg = self
            .error
            .as_deref()
            .unwrap_or("No Ollama models found. Make sure Ollama is running.");

        let paragraph = Paragraph::new(error_msg)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_stats(&self, frame: &mut Frame, area: Rect) {
        let selected_model = &self.models[self.selected_model];
        let stats = self
            .stats
            .iter()
            .find(|s| s.name == selected_model.name)
            .unwrap();

        // Create a block with borders
        let _ = Block::default()
            .borders(Borders::ALL)
            .title(format!("Model: {}", selected_model.name));

        // Calculate layout for stats
        let stats_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(area);

        // Display model information
        self.stats_ui.render_model_info(frame, stats_layout[0], selected_model);

        // Display context usage
        self.stats_ui.render_context_usage(
            frame,
            stats_layout[1],
            stats.context_usage_percent(),
        );

        // Display response time
        let response_time = if let Some(time) = stats.response_time_ms {
            format!("{:.2} ms", time)
        } else {
            "N/A".to_string()
        };

        let completion_count = stats.completion_count.to_string();
        let total_tokens = stats.usage.total_tokens.to_string();
        let current_tokens = stats.current_token_count.to_string();

        let info_lines = vec![
            Line::from(vec![
                Span::styled("Response Time: ", Style::default().fg(Color::White)),
                Span::styled(&response_time, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Completions: ", Style::default().fg(Color::White)),
                Span::styled(&completion_count, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Total Tokens: ", Style::default().fg(Color::White)),
                Span::styled(&total_tokens, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Current Tokens: ", Style::default().fg(Color::White)),
                Span::styled(&current_tokens, Style::default().fg(Color::Cyan)),
            ]),
        ];

        let info_paragraph = Paragraph::new(info_lines).block(
            Block::default()
                .borders(Borders::TOP | Borders::BOTTOM)
                .title("Statistics"),
        );

        frame.render_widget(info_paragraph, stats_layout[2]);

        // Render the main stats component
        self.stats_ui.render(frame, stats_layout[3], stats);
    }
}

/// Application event
#[derive(Debug, Clone, Copy)]
pub enum Event {
    Tick,
    Key {
        code: KeyEvent,
    },
}

/// Event loop
pub async fn run_event_loop(app: &mut App, mut rx: tokio::sync::mpsc::Receiver<Event>) {
    // Setup ticker
    let mut ticker = tokio::time::interval(Duration::from_secs(2));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Err(_) = app.update_stats().await {
                    app.error = Some("Failed to update stats".to_string());
                }
            }
            msg = rx.recv() => {
                match msg {
                    Some(Event::Key { code }) => {
                        match code.code {
                            KeyCode::Char('q') => {
                                return;
                            }
                            KeyCode::Char('r') => {
                                if let Err(_) = app.update_stats().await {
                                    app.error = Some("Failed to refresh".to_string());
                                }
                            }
                            KeyCode::Up => {
                                app.previous_model();
                            }
                            KeyCode::Down => {
                                app.next_model();
                            }
                            _ => {}
                        }
                    }
                    Some(Event::Tick) => {}
                    None => break,
                }
            }
        }
    }
}