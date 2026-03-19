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

/// Application state
#[derive(Clone)]
pub struct App {
    /// List of available models
    pub models: Vec<OllamaModel>,
    /// Selected model index
    pub selected_model: usize,
    /// Model stats keyed by model name position in `models`
    pub stats: Vec<ModelStats>,
    /// UI components
    stats_ui: OllamaStatsUI,
    /// Last error message
    pub error: Option<String>,
    /// Loading state
    pub loading: bool,
}

impl App {
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

    /// Initialize the application by fetching models and their initial stats
    pub async fn initialize(&mut self) -> Result<()> {
        self.loading = true;
        self.error = None;

        let client = OllamaClient::new()?;

        match client.list_models().await {
            Ok(models) => {
                self.models = models;
                self.loading = false;

                for model in &self.models {
                    match client.get_model_stats(&model.name).await {
                        Ok(mut stats) => {
                            stats.token_history.push(stats.current_token_count);
                            self.stats.push(stats);
                        }
                        Err(e) => {
                            eprintln!("Failed to get initial stats for {}: {}", model.name, e);
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

    /// Merge freshly-fetched stats into the stored list, preserving history
    pub fn apply_stats(&mut self, mut new_stats: ModelStats) {
        let model_name = new_stats.name.clone();
        if let Some(existing) = self.stats.iter_mut().find(|s| s.name == model_name) {
            new_stats.completion_count = existing.completion_count + 1;
            new_stats.token_history = std::mem::take(&mut existing.token_history);
            new_stats.token_history.push(new_stats.current_token_count);
            if new_stats.token_history.len() > 20 {
                new_stats.token_history.remove(0);
            }
            *existing = new_stats;
        } else {
            new_stats.token_history.push(new_stats.current_token_count);
            self.stats.push(new_stats);
        }
        self.error = None;
    }

    /// Return the name of the currently selected model (if any)
    pub fn selected_model_name(&self) -> Option<&str> {
        self.models.get(self.selected_model).map(|m| m.name.as_str())
    }

    /// Handle a key event; returns true if the app should quit
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return true,
            KeyCode::Up | KeyCode::Char('k') => self.previous_model(),
            KeyCode::Down | KeyCode::Char('j') => self.next_model(),
            _ => {}
        }
        false
    }

    fn previous_model(&mut self) {
        if !self.models.is_empty() {
            self.selected_model = self.selected_model.saturating_sub(1);
        }
    }

    fn next_model(&mut self) {
        if !self.models.is_empty() {
            self.selected_model = (self.selected_model + 1) % self.models.len();
        }
    }

    /// Render the full UI
    pub fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(frame.area());

        self.render_header(frame, layout[0]);

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

        let hint = Span::styled(
            "  [↑/↓ or j/k] select  [q] quit",
            Style::default().fg(Color::DarkGray),
        );

        let line = Line::from(vec![title, status, hint]);
        let paragraph = Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_empty_state(&self, frame: &mut Frame, area: Rect) {
        let msg = self
            .error
            .as_deref()
            .unwrap_or("No Ollama models found. Make sure Ollama is running.");

        let paragraph = Paragraph::new(msg)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_stats(&self, frame: &mut Frame, area: Rect) {
        let selected_model = &self.models[self.selected_model];

        let Some(stats) = self.stats.iter().find(|s| s.name == selected_model.name) else {
            let paragraph = Paragraph::new("Fetching stats…")
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
            return;
        };

        // model info (1 line + top border = 2 rows)
        // context gauge (3 rows)
        // numeric stats block (4 lines + 2 borders = 6 rows)
        // sparkline / token breakdown fills the rest
        let stats_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Fill(1),
            ])
            .split(area);

        self.stats_ui.render_model_info(frame, stats_layout[0], selected_model);

        self.stats_ui
            .render_context_usage(frame, stats_layout[1], stats.context_usage_percent());

        // Numeric stats
        let response_time = stats
            .response_time_ms
            .map(|t| format!("{:.2} ms", t))
            .unwrap_or_else(|| "N/A".to_string());

        let info_lines = vec![
            Line::from(vec![
                Span::styled("Response Time: ", Style::default().fg(Color::White)),
                Span::styled(response_time, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Completions:   ", Style::default().fg(Color::White)),
                Span::styled(
                    stats.completion_count.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Tokens:  ", Style::default().fg(Color::White)),
                Span::styled(
                    stats.usage.total_tokens.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Current Tokens:", Style::default().fg(Color::White)),
                Span::styled(
                    stats.current_token_count.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let info_paragraph = Paragraph::new(info_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Statistics"),
        );
        frame.render_widget(info_paragraph, stats_layout[2]);

        self.stats_ui.render(frame, stats_layout[3], stats);
    }
}
