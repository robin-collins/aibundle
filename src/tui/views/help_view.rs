// src/tui/views/help_view.rs
//!
//! # Help View
//!
//! This module defines the `HelpView` component for rendering the help popup in the TUI.
//! It displays keyboard shortcuts, navigation, and usage instructions.
//!
//! ## Usage
//! Use `HelpView` to show a modal with help and keybindings in the TUI.
//!
//! ## Examples
//! ```rust
//! use crate::tui::views::HelpView;
//! let help_view = HelpView::new();
//! help_view.render(f, area, app_state);
//! ```

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Help view component for rendering the help popup/modal in the TUI.
pub struct HelpView;

impl HelpView {
    /// Creates a new `HelpView` component.
    pub fn new() -> Self {
        Self
    }

    /// Renders the help popup/modal with keyboard shortcuts and usage info.
    pub fn render(&self, f: &mut Frame, area: Rect, _app_state: &crate::tui::state::AppState) {
        // Create a centered area for the help content
        let help_area = centered_rect(80, 90, area);

        // Create a block with a border for the help view
        let help_block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        // Help content
        let help_text = vec![
            Line::from(Span::styled(
                "Keyboard Controls",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Navigation",
                Style::default().fg(Color::Green),
            )),
            Line::from("↑/↓         - Move selection up/down"),
            Line::from("PgUp/PgDown - Move selection by page"),
            Line::from("Enter       - Open directory"),
            Line::from("Backspace   - Go to parent directory"),
            Line::from("Tab         - Expand/collapse directory"),
            Line::from(""),
            Line::from(Span::styled("Selection", Style::default().fg(Color::Green))),
            Line::from("Space       - Select/deselect item"),
            Line::from("*           - Select/deselect all"),
            Line::from(""),
            Line::from(Span::styled("Search", Style::default().fg(Color::Green))),
            Line::from("/           - Start search"),
            Line::from("ESC         - Clear search"),
            Line::from(""),
            Line::from(Span::styled("Actions", Style::default().fg(Color::Green))),
            Line::from("c           - Copy selection to clipboard"),
            Line::from("f           - Toggle output format (XML/MD/JSON/LLM)"),
            Line::from("n           - Toggle line numbers"),
            Line::from(""),
            Line::from(Span::styled("Filters", Style::default().fg(Color::Green))),
            Line::from("i           - Toggle default ignore patterns"),
            Line::from("g           - Toggle .gitignore"),
            Line::from("b           - Toggle binary files"),
            Line::from("r           - Toggle recursive mode"),
            Line::from(""),
            Line::from(Span::styled("Other", Style::default().fg(Color::Green))),
            Line::from("h           - Show/hide this help"),
            Line::from("q           - Quit"),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key to close help",
                Style::default().fg(Color::Yellow),
            )),
        ];

        // Render the help paragraph
        let help_paragraph = Paragraph::new(help_text)
            .block(help_block)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        f.render_widget(help_paragraph, help_area);
    }
}

/// Helper function to create a centered rectangle for the help view
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

// TODO: Add support for paginated help or dynamic keybinding display.
