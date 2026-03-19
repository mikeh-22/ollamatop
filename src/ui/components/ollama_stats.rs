use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Sparkline};
use ratatui::Frame;
use crate::model::stats::{ModelStats, OllamaModel};

/// Main stats component for displaying model statistics
#[derive(Clone)]
pub struct OllamaStatsUI;

impl OllamaStatsUI {
    pub fn new() -> Self {
        Self
    }

    /// Render model information line
    pub fn render_model_info(&self, frame: &mut Frame, area: Rect, model: &OllamaModel) {
        let info = format!(
            "{} | {}B | {} | Modified: {}",
            model.name,
            model.parameters,
            model.quantization.as_deref().unwrap_or("unknown"),
            model.modified_at
        );

        let paragraph = Paragraph::new(info)
            .block(Block::default().borders(Borders::TOP))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    /// Render context usage gauge
    pub fn render_context_usage(&self, frame: &mut Frame, area: Rect, percent: f64) {
        let clamped = percent.clamp(0.0, 100.0) as u16;
        let gauge = Gauge::default()
            .label(format!("{:.1}%", percent))
            .percent(clamped)
            .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::NONE).title("Context Usage"));

        frame.render_widget(gauge, area);
    }

    /// Render the token history sparkline and per-token breakdown
    pub fn render(&self, frame: &mut Frame, area: Rect, stats: &ModelStats) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(area);

        // Sparkline uses real token history collected across refreshes
        let sparkline = Sparkline::default()
            .data(&stats.token_history)
            .block(Block::default().borders(Borders::NONE).title("Token History"));

        frame.render_widget(sparkline, layout[0]);

        // Per-token breakdown (prompt / completion)
        let mut lines: Vec<Line> = Vec::new();

        if let Some(prompt_tokens) = stats.usage.prompt_tokens {
            lines.push(Line::from(vec![
                Span::styled("Prompt:     ", Style::default().fg(Color::White)),
                Span::styled(
                    prompt_tokens.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }

        if let Some(completion_tokens) = stats.usage.completion_tokens {
            lines.push(Line::from(vec![
                Span::styled("Completion: ", Style::default().fg(Color::White)),
                Span::styled(
                    completion_tokens.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::BOTTOM)
                    .title("Token Breakdown"),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, layout[1]);
    }
}
