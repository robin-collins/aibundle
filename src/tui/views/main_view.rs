use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::models::constants::ICONS;
use crate::tui::components::{FileList, Modal, StatusBar};
use crate::tui::state::{AppState, SearchState, SelectionState};
use crate::tui::views::{HelpView, MessageView};

pub struct MainView {
    file_list: FileList,
    status_bar: StatusBar,
    help_view: HelpView,
    message_view: MessageView,
}

impl MainView {
    pub fn new() -> Self {
        Self {
            file_list: FileList::new(),
            status_bar: StatusBar::new(),
            help_view: HelpView::new(),
            message_view: MessageView::new(),
        }
    }

    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState,
        selection_state: &SelectionState,
        search_state: &Option<SearchState>,
    ) {
        // Create the main layout with file list and status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // File list takes most of the space
                Constraint::Length(1), // Status bar at bottom
            ])
            .split(area);

        // Render file list
        self.file_list
            .render(f, chunks[0], app_state, selection_state);

        // Render status bar
        self.status_bar
            .render(f, chunks[1], app_state, selection_state);

        // Render search UI if in search mode
        if let Some(search_state) = search_state {
            let search_area = Rect {
                x: area.x + 1,
                y: area.y + area.height - 2,
                width: area.width - 2,
                height: 1,
            };
            let search_text = format!(
                "{} {}",
                if search_state.is_regex {
                    "Regex:"
                } else {
                    "Search:"
                },
                search_state.query
            );
            let search_para = Paragraph::new(search_text).style(Style::default().fg(Color::Yellow));
            f.render_widget(search_para, search_area);
        }

        // Render help view or message view if active
        if app_state.show_help {
            self.help_view.render(f, area, app_state);
        } else if app_state.show_message {
            self.message_view.render(f, area, app_state);
        }

        // Render modal if active
        if let Some(modal) = &app_state.modal {
            let modal_area = centered_rect(60, 20, area);
            modal.render(f, modal_area);
        }
    }
}

// Helper function to create a centered rect
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
