use crate::tui::state::{AppState, MessageType};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::time::{Duration, Instant};

pub struct MessageView {
    // Duration to show messages for
    message_duration: Duration,
}

impl MessageView {
    pub fn new() -> Self {
        Self {
            message_duration: Duration::from_secs(3),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, app_state: &AppState) {
        // Only render if there's a message to show
        if let Some(message) = &app_state.message {
            // Check if the message should be expired
            if Instant::now().duration_since(message.timestamp) > self.message_duration {
                return;
            }

            // Calculate a smaller centered area for the message
            let message_area = centered_rect(60, 15, area);

            // Create a block with a border for the message
            let message_block = Block::default()
                .title(" Message ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow));

            // Determine message style based on message type
            let message_style = match message.message_type {
                MessageType::Info => Style::default().fg(Color::Cyan),
                MessageType::Success => Style::default().fg(Color::Green),
                MessageType::Warning => Style::default().fg(Color::Yellow),
                MessageType::Error => Style::default().fg(Color::Red),
            };

            // Create the message paragraph with styled spans within Lines
            let message_content = Line::from(vec![
                Span::styled(
                    match message.message_type {
                        MessageType::Info => "INFO: ",
                        MessageType::Success => "SUCCESS: ",
                        MessageType::Warning => "WARNING: ",
                        MessageType::Error => "ERROR: ",
                    },
                    message_style.add_modifier(Modifier::BOLD),
                ),
                Span::styled(&message.content, message_style),
            ]);

            let message_paragraph = Paragraph::new(vec![
                Line::from(""),
                message_content,
                Line::from(""),
                Line::from(Span::styled(
                    "Press any key to dismiss",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(message_block)
            .wrap(Wrap { trim: true });

            f.render_widget(message_paragraph, message_area);
        }
    }

    pub fn set_message_duration(&mut self, duration: Duration) {
        self.message_duration = duration;
    }
}

// Helper function to create a centered rectangle for the message
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
