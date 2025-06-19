// src/tui/components/header.rs
//!
//! # Header View Component
//!
//! Provides the [`HeaderView`] component for rendering the top header bar in the TUI, including version, current directory, and search input.
//!
//! ## Purpose
//!
//! - Display application version and current working directory.
//! - Show search input when searching is active.
//!
//! ## Organization
//!
//! - [`HeaderView`]: Main component for header rendering.
//!
//! ## Example
//! ```rust
//! use crate::tui::components::HeaderView;
//! use crate::tui::state::{AppState, SearchState};
//! # use ratatui::{backend::TestBackend, Terminal, layout::Rect};
//! # let app_state = AppState::default_for_test();
//! # let search_state = SearchState::new();
//! # let backend = TestBackend::new(80, 3);
//! # let mut terminal = Terminal::new(backend).unwrap();
//! # let area = Rect::new(0, 0, 80, 3);
//! let header = HeaderView::new();
//! terminal.draw(|f| {
//!     header.render(f, area, &app_state, &search_state);
//! }).unwrap();
//! ```
//!
//! # Doc Aliases
//! - "header-bar"
//! - "tui-header"
//!
#![doc(alias = "header-bar")]
#![doc(alias = "tui-header")]

use ratatui::{
    layout::Rect,
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::models::constants::VERSION;
use crate::tui::state::{AppState, SearchState};

/// Header view component for rendering the top bar and search input in the TUI.
///
/// # Purpose
/// Displays the application version, current directory, and search input (when active).
///
/// # Examples
/// ```rust
/// use crate::tui::components::HeaderView;
/// let header = HeaderView::new();
/// # // See module-level example for full usage
/// ```
pub struct HeaderView {}

impl Default for HeaderView {
    fn default() -> Self {
        Self::new()
    }
}

impl HeaderView {
    /// Creates a new [`HeaderView`] component.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::HeaderView;
    /// let header = HeaderView::new();
    /// ```
    pub fn new() -> Self {
        Self {}
    }

    /// Renders the header bar and search input (if active).
    ///
    /// # Arguments
    /// * `f` - The TUI frame to render into.
    /// * `area` - The area to render the header in.
    /// * `app_state` - The current application state.
    /// * `search_state` - The current search state.
    ///
    /// # Panics
    /// This function does not panic.
    ///
    /// # Examples
    /// ```rust
    /// # use crate::tui::components::HeaderView;
    /// # use crate::tui::state::{AppState, SearchState};
    /// # use ratatui::{backend::TestBackend, Terminal, layout::Rect};
    /// # let app_state = AppState::default_for_test();
    /// # let search_state = SearchState::new();
    /// # let backend = TestBackend::new(80, 3);
    /// # let mut terminal = Terminal::new(backend).unwrap();
    /// # let area = Rect::new(0, 0, 80, 3);
    /// let header = HeaderView::new();
    /// terminal.draw(|f| {
    ///     header.render(f, area, &app_state, &search_state);
    /// }).unwrap();
    /// ```
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState,
        search_state: &SearchState,
    ) {
        // Compose the header title with version and current directory
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
        if app_state.is_searching && inner_area.height > 0 {
            // Use only the last line of the inner_area for search input
            let search_line_area = Rect {
                y: inner_area.y + inner_area.height - 1,
                height: 1,
                ..inner_area
            };

            // Blinking cursor logic: toggles every 500ms
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

            // Compose the search input text with blinking cursor
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

// TODO: Add support for displaying additional status info in the header.
// TODO: Add breadcrumbs or path shortening for long directories.
