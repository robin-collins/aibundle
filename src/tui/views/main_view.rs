// src/tui/views/main_view.rs
//!
//! # Main View
//!
//! This module defines the `MainView` component for rendering the main TUI layout.
//! It manages the header, file list, status bar, help, message, and modal overlays.
//!
//! ## Usage
//! Use `MainView` to render the primary TUI interface, including all major UI components.
//!
//! ## Examples
//! ```rust
//! use crate::tui::views::MainView;
//! let main_view = MainView::new();
//! main_view.render(f, area, app_state, selection_state, search_state);
//! ```

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::tui::components::{FileList, HeaderView, StatusBar};
use crate::tui::state::{AppState, SearchState, SelectionState};
use crate::tui::views::{HelpView, MessageView};

/// Main view component for rendering the primary TUI layout and overlays.
pub struct MainView {
    header_view: HeaderView,
    file_list: FileList,
    status_bar: StatusBar,
    help_view: HelpView,
    message_view: MessageView,
}

impl MainView {
    /// Creates a new `MainView` with all subcomponents initialized.
    pub fn new() -> Self {
        Self {
            header_view: HeaderView::new(),
            file_list: FileList::new(),
            status_bar: StatusBar::new(),
            help_view: HelpView::new(),
            message_view: MessageView::new(),
        }
    }

    /// Renders the main TUI layout, including header, file list, status bar, overlays, and modals.
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState,
        selection_state: &mut SelectionState,
        search_state: &SearchState,
    ) {
        // Create the main layout with header, file list, and status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header takes 3 lines
                Constraint::Min(1),    // File list takes rest of the space
                Constraint::Length(1), // Status bar at bottom
            ])
            .split(area);

        // Render Header
        self.header_view
            .render(f, chunks[0], app_state, search_state);

        // Render file list
        self.file_list
            .render(f, chunks[1], app_state, selection_state);

        // Render status bar
        self.status_bar
            .render(f, chunks[2], app_state, selection_state);

        // Render help view or message view if active
        if app_state.show_help {
            self.help_view.render(f, area, app_state);
        } else if app_state.show_message {
            self.message_view.render(f, area, app_state);
        }

        // Render modal if active
        if let Some(modal) = &app_state.modal {
            let modal_area = centered_rect(modal.width, modal.height, area);
            modal.render(f, modal_area);
        }
    }
}

/// Helper function to create a centered rectangle for modal overlays
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height.min(100)) / 2),
            Constraint::Length(height.min(r.height)),
            Constraint::Percentage((100 - height.min(100)) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width.min(100)) / 2),
            Constraint::Length(width.min(r.width)),
            Constraint::Percentage((100 - width.min(100)) / 2),
        ])
        .split(popup_layout[1])[1]
}

// TODO: Add support for additional overlays or popups.
