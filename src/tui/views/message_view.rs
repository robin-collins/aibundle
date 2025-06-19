// src/tui/views/message_view.rs
//!
//! # Message View
//!
//! Provides the [`MessageView`] component for rendering temporary messages in the TUI.
//!
//! ## Purpose
//!
//! - Display info, success, warning, or error messages in a centered popup overlay.
//! - Provide user feedback and alerts in a non-intrusive manner.
//!
//! ## Organization
//!
//! - [`MessageView`]: Main struct for message rendering.
//! - `centered_rect`: Utility for modal positioning.
//!
//! ## Usage
//!
//! ```rust
//! use crate::tui::views::MessageView;
//! let mut message_view = MessageView::new();
//! message_view.set_message_duration(std::time::Duration::from_secs(5));
//! message_view.render(f, area, app_state);
//! ```
//!
//! # Doc Aliases
//! - "message"
//! - "popup"
//! - "alert"
#![doc(alias = "message")]
#![doc(alias = "popup")]
#![doc(alias = "alert")]

use crate::tui::state::{AppState, MessageType};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::time::{Duration, Instant};

/// Message view component for rendering temporary messages in a centered popup.
///
/// # Fields
///
/// * `message_duration` - Duration to show messages for.
///
/// # Examples
///
/// ```rust
/// use crate::tui::views::MessageView;
/// let mut message_view = MessageView::new();
/// message_view.set_message_duration(std::time::Duration::from_secs(5));
/// ```
pub struct MessageView {
    /// Duration to show messages for.
    message_duration: Duration,
}

impl Default for MessageView {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageView {
    /// Creates a new [`MessageView`] with a default message duration.
    ///
    /// # Returns
    ///
    /// * [`MessageView`] - A new message view instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::tui::views::MessageView;
    /// let message_view = MessageView::new();
    /// ```
    pub fn new() -> Self {
        Self {
            message_duration: Duration::from_secs(3),
        }
    }

    /// Renders the message popup if a message is present and not expired.
    ///
    /// # Arguments
    ///
    /// * `f` - The TUI frame to render into.
    /// * `area` - The area within which to render the message popup.
    /// * `app_state` - The current application state.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Typically called within the TUI render loop
    /// message_view.render(f, area, &app_state);
    /// ```
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
}

/// Helper function to create a centered rectangle for the message popup.
///
/// # Arguments
///
/// * `percent_x` - Width as a percentage of the parent area (0-100).
/// * `percent_y` - Height as a percentage of the parent area (0-100).
/// * `r` - The parent area rectangle.
///
/// # Returns
///
/// * `Rect` - The centered rectangle.
///
/// # Examples
///
/// ```rust
/// let area = ratatui::layout::Rect::new(0, 0, 100, 40);
/// let popup = crate::tui::views::message_view::centered_rect(60, 15, area);
/// assert!(popup.width <= area.width && popup.height <= area.height);
/// ```
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

// TODO: Add support for message queueing or stacking.
