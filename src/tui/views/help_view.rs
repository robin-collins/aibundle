use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct HelpView;

impl HelpView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, area: Rect, app_state: &crate::tui::state::AppState) {
        // Create a centered area for the help content
        let help_area = centered_rect(80, 90, area);

        // Create a block with a border for the help view
        let help_block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        // Help content
        let help_text = vec![
            Spans::from(Span::styled(
                "Keyboard Controls",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Spans::from(""),
            Spans::from(Span::styled(
                "Navigation",
                Style::default().fg(Color::Green),
            )),
            Spans::from("↑/↓         - Move selection up/down"),
            Spans::from("PgUp/PgDown - Move selection by page"),
            Spans::from("Enter       - Open directory"),
            Spans::from("Backspace   - Go to parent directory"),
            Spans::from("Tab         - Expand/collapse directory"),
            Spans::from(""),
            Spans::from(Span::styled("Selection", Style::default().fg(Color::Green))),
            Spans::from("Space       - Select/deselect item"),
            Spans::from("*           - Select/deselect all"),
            Spans::from(""),
            Spans::from(Span::styled("Search", Style::default().fg(Color::Green))),
            Spans::from("/           - Start search"),
            Spans::from("ESC         - Clear search"),
            Spans::from(""),
            Spans::from(Span::styled("Actions", Style::default().fg(Color::Green))),
            Spans::from("c           - Copy selection to clipboard"),
            Spans::from("f           - Toggle output format (XML/MD/JSON/LLM)"),
            Spans::from("n           - Toggle line numbers"),
            Spans::from(""),
            Spans::from(Span::styled("Filters", Style::default().fg(Color::Green))),
            Spans::from("i           - Toggle default ignore patterns"),
            Spans::from("g           - Toggle .gitignore"),
            Spans::from("b           - Toggle binary files"),
            Spans::from("r           - Toggle recursive mode"),
            Spans::from(""),
            Spans::from(Span::styled("Other", Style::default().fg(Color::Green))),
            Spans::from("h           - Show/hide this help"),
            Spans::from("q           - Quit"),
            Spans::from(""),
            Spans::from(Span::styled(
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

// Helper function to create a centered rectangle for the help view
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
