// src/tui/views/main_view.rs
//!
//! # Main View
//!
//! Provides the [`MainView`] component for rendering the main TUI layout, including header, file list, status bar, and overlays.
//!
//! ## Purpose
//!
//! - Render the primary TUI interface and manage all major UI components.
//! - Handle overlays such as help and message modals.
//!
//! ## Organization
//!
//! - [`MainView`]: Main struct for layout and rendering.
//! - `centered_rect`: Utility for modal positioning.
//!
//! ## Usage
//!
//! ```rust
//! use crate::tui::views::MainView;
//! let main_view = MainView::new();
//! main_view.render(f, area, app_state, selection_state, search_state);
//! ```
//!
//! # Doc Aliases
//! - "main view"
//! - "layout"
//! - "tui root"
#![doc(alias = "main view")]
#![doc(alias = "layout")]
#![doc(alias = "tui root")]

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::tui::components::{FileList, HeaderView, StatusBar};
use crate::tui::state::{AppState, SearchState, SelectionState};
use crate::tui::views::{HelpView, MessageView};

/// Main view component for rendering the primary TUI layout and overlays.
///
/// # Fields
///
/// * `header_view` - Renders the header section.
/// * `file_list` - Renders the file list section.
/// * `status_bar` - Renders the status bar.
/// * `help_view` - Renders the help overlay.
/// * `message_view` - Renders the message overlay.
///
/// # Examples
///
/// ```rust
/// use crate::tui::views::MainView;
/// let main_view = MainView::new();
/// ```
pub struct MainView {
    header_view: HeaderView,
    file_list: FileList,
    status_bar: StatusBar,
    help_view: HelpView,
    message_view: MessageView,
}

impl Default for MainView {
    fn default() -> Self {
        Self::new()
    }
}

impl MainView {
    /// Creates a new [`MainView`] with all subcomponents initialized.
    ///
    /// # Returns
    ///
    /// * [`MainView`] - A new main view instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::tui::views::MainView;
    /// let main_view = MainView::new();
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `f` - The TUI frame to render into.
    /// * `area` - The area within which to render the main view.
    /// * `app_state` - The current application state.
    /// * `selection_state` - The current selection state (mutable).
    /// * `search_state` - The current search state.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Typically called within the TUI render loop
    /// main_view.render(f, area, &app_state, &mut selection_state, &search_state);
    /// ```
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
        if let Some(modal) = &app_state.modal {
            // Check if this is a help modal by looking at the message content
            if modal.message.contains("Keyboard Shortcuts") {
                self.help_view.render(f, area, app_state);
            } else {
                // Render modal if active
                let modal_area = centered_rect(modal.width, modal.height, area);
                modal.render(f, modal_area);
            }
        } else if app_state.message.is_some() {
            self.message_view.render(f, area, app_state);
        }
    }
}

/// Helper function to create a centered rectangle for modal overlays.
///
/// # Arguments
///
/// * `width` - The width of the modal in cells.
/// * `height` - The height of the modal in cells.
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
/// let modal = crate::tui::views::main_view::centered_rect(60, 20, area);
/// assert!(modal.width <= area.width && modal.height <= area.height);
/// ```
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

