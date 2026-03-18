use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Modifier};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Sparkline};
use ratatui::Frame;
use crate::model::stats::{ModelStats, OllamaModel};

/// Main stats component for displaying model statistics
#[derive(Clone)]
pub struct OllamaStatsUI {
    max_context_window: u64,
}

impl OllamaStatsUI {
    pub fn new() -> Self {
        Self {
            max_context_window: 4096, // Default context window size
        }
    }

    /// Render model information
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
        let gauge = Gauge::default()
            .label(format!("{:.1}%", percent))
            .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .block(Block::default().title("Context Usage"));

        frame.render_widget(gauge, area);
    }

    /// Render the main stats widget
    pub fn render(&self, frame: &mut Frame, area: Rect, stats: &ModelStats) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
                .split(area);

        // Render token history sparkline
        let token_usage = vec![
            (stats.usage.total_tokens / 100) as u64,
            ((stats.usage.total_tokens / 2) / 100) as u64,
            ((stats.usage.total_tokens * 3) / 4 / 100) as u64,
            stats.usage.total_tokens / 100,
        ];

        let sparkline = Sparkline::default()
            .data(&token_usage)
            .block(Block::default().title("Token Usage (Last 4 Requests)"));

        frame.render_widget(sparkline, layout[0]);

        // Render usage breakdown
        let mut lines = Vec::new();

        if let Some(prompt_tokens) = stats.usage.prompt_tokens {
            lines.push(vec![
                Span::styled("Prompt: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", prompt_tokens),
                    Style::default().fg(Color::Cyan),
                ),
            ]);
        }

        if let Some(completion_tokens) = stats.usage.completion_tokens {
            lines.push(vec![
                Span::styled("Completion: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", completion_tokens),
                    Style::default().fg(Color::Cyan),
                ),
            ]);
        }

        let completion_paragraph = Paragraph::new(
            lines
                .into_iter()
                .flatten()
                .map(|span| ratatui::text::Line::from(span))
                .collect::<Vec<ratatui::text::Line<'_>>>()
        )
        .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
        .alignment(Alignment::Left);

        frame.render_widget(completion_paragraph, layout[1]);
    }
}