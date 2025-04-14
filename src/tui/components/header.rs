use ratatui::{
    layout::Rect,
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::models::constants::VERSION;
use crate::tui::state::{AppState, SearchState};

pub struct HeaderView {}

impl HeaderView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState,
        search_state: &SearchState,
    ) {
        let title = format!(
            " AIBundle v{} - {} ",
            VERSION,
            app_state.current_dir.display()
        );
        let block = Block::default().title(title).borders(Borders::ALL);

        // Calculate inner_area *before* rendering the block
        let inner_area = block.inner(area);

        // Render the main block first
        f.render_widget(block, area);

        // If searching, render the search input inside the calculated inner_area
        if app_state.is_searching {
            if inner_area.height > 0 {
                // Check if inner_area has any height
                // Use only the last line of the inner_area for search input
                let search_line_area = Rect {
                    y: inner_area.y + inner_area.height - 1,
                    height: 1,
                    ..inner_area // Inherit x and width from inner_area
                };

                let cursor = if (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
                    / 500)
                    % 2
                    == 0
                {
                    "█"
                } else {
                    " "
                };

                let search_text = format!(
                    "Search: {}{} (Press / to finish, ESC to cancel)",
                    search_state.search_query, cursor
                );
                let search_widget = Paragraph::new(search_text).fg(ratatui::style::Color::Yellow);
                // Render in the calculated search line area
                f.render_widget(search_widget, search_line_area);
            }
        }
    }
}
