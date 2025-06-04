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
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Help view component for rendering the help popup/modal in the TUI.
pub struct HelpView;

impl Default for HelpView {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpView {
    /// Creates a new `HelpView` component.
    pub fn new() -> Self {
        Self
    }

    /// Renders the help popup/modal with keyboard shortcuts and usage info.
    pub fn render(&self, f: &mut Frame, area: Rect, _app_state: &crate::tui::state::AppState) {
        // Create a centered area for the help content
        let help_area = centered_rect(85, 95, area);

        // Create a block with a border for the help view
        let help_block = Block::default()
            .title(" 📋 AIBundle Help ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        // Add padding inside the modal
        let inner_area = help_area.inner(Margin {
            vertical: 1,
            horizontal: 2,
        });

        // Help content with enhanced styling
        let help_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("🧭 ", Style::default().fg(Color::Blue)),
                Span::styled(
                    "Navigation",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "─────────────",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(vec![
                Span::styled("  ↑/↓        ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Move selection up/down", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  PgUp/PgDn  ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Move by 10 items", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Enter      ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Open directory", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Backspace  ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Go to parent directory", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Tab        ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Expand/collapse folder", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Home/End   ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Jump to first/last item", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🎯 ", Style::default().fg(Color::Green)),
                Span::styled(
                    "Selection",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "────────────",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(vec![
                Span::styled("  Space      ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Select/deselect item", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  a          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "Select/deselect all items",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("⚡ ", Style::default().fg(Color::Magenta)),
                Span::styled(
                    "Actions",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "─────────",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(vec![
                Span::styled("  c          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Copy to clipboard", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  f          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "Toggle format (XML/MD/JSON/LLM)",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("  n          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Toggle line numbers", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  /          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Search (ESC to cancel)", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🔍 ", Style::default().fg(Color::Red)),
                Span::styled(
                    "Filters",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "──────────",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(vec![
                Span::styled("  d          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Toggle default ignores", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  g          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Toggle .gitignore rules", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  b          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Toggle binary files", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  r          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Toggle recursive mode", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🛠️  ", Style::default().fg(Color::LightBlue)),
                Span::styled(
                    "Other",
                    Style::default()
                        .fg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "────────",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(vec![
                Span::styled("  h / ?      ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Show/hide this help", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  S          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Save current config", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  q          ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "Quit (copies if items selected)",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+C     ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Quit immediately", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("💡 ", Style::default().fg(Color::LightYellow)),
                Span::styled(
                    "Help Navigation",
                    Style::default()
                        .fg(Color::LightYellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "─────────────────",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(vec![
                Span::styled("  PgUp/PgDn  ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Scroll help pages", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Esc / q    ", Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Close help", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "                    Press the [Escape] key to close help",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::ITALIC),
            )),
        ];

        // Render the help paragraph with enhanced styling
        let help_paragraph = Paragraph::new(help_text)
            .block(help_block)
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(Color::Black));

        f.render_widget(help_paragraph, inner_area);
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
