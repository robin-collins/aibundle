// src/tui/components/status_bar.rs
//!
//! # Status Bar Component
//!
//! Provides the [`StatusBar`] component for rendering the status bar at the bottom of the TUI.
//!
//! ## Purpose
//!
//! - Display item counts, selection info, and key command hints.
//! - Show toggle states for ignores, gitignore, binary files, output format, and line numbers.
//!
//! ## Organization
//!
//! - [`StatusBar`]: Main status bar component.
//!
//! ## Example
//! ```rust
//! use crate::tui::components::StatusBar;
//! use crate::tui::state::{AppState, SelectionState};
//! # use ratatui::{backend::TestBackend, Terminal, layout::Rect};
//! # let app_state = AppState::default_for_test();
//! # let selection_state = SelectionState::new();
//! # let backend = TestBackend::new(80, 1);
//! # let mut terminal = Terminal::new(backend).unwrap();
//! # let area = Rect::new(0, 0, 80, 1);
//! let status_bar = StatusBar::new();
//! terminal.draw(|f| {
//!     status_bar.render(f, area, &app_state, &selection_state);
//! }).unwrap();
//! ```
//!
//! # Doc Aliases
//! - "status-bar"
//! - "tui-status"
//!
#![doc(alias = "status-bar")]
#![doc(alias = "tui-status")]

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::tui::state::AppState;

/// Status bar component for rendering item counts, selection info, and key hints.
///
/// # Purpose
/// Shows dynamic information about the current state, including item and selection counts, toggle states, and key command hints.
///
/// # Examples
/// ```rust
/// use crate::tui::components::StatusBar;
/// let status_bar = StatusBar::new();
/// # // See module-level example for full usage
/// ```
pub struct StatusBar;

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusBar {
    /// Creates a new [`StatusBar`] component.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::StatusBar;
    /// let status_bar = StatusBar::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Renders the status bar with dynamic information and key command hints.
    ///
    /// # Arguments
    /// * `f` - The TUI frame to render into.
    /// * `area` - The area to render the status bar in.
    /// * `app_state` - The current application state.
    /// * `_selection_state` - The current selection state (unused).
    ///
    /// # Panics
    /// This function does not panic.
    ///
    /// # Examples
    /// ```rust
    /// # use crate::tui::components::StatusBar;
    /// # use crate::tui::state::{AppState, SelectionState};
    /// # use ratatui::{backend::TestBackend, Terminal, layout::Rect};
    /// # let app_state = AppState::default_for_test();
    /// # let selection_state = SelectionState::new();
    /// # let backend = TestBackend::new(80, 1);
    /// # let mut terminal = Terminal::new(backend).unwrap();
    /// # let area = Rect::new(0, 0, 80, 1);
    /// let status_bar = StatusBar::new();
    /// terminal.draw(|f| {
    ///     status_bar.render(f, area, &app_state, &selection_state);
    /// }).unwrap();
    /// ```
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState,
        _selection_state: &crate::tui::state::SelectionState,
    ) {
        let selection_status = if app_state.is_counting {
            format!("({} selected, ...adding more)", app_state.selected_items.len())
        } else {
            format!("({} selected)", app_state.selected_items.len())
        };

        let status_text = format!(
            // Show item and selection counts, and status of toggles
            " {} items {} - Space: select, Enter: open dir, c: copy, i: ignores [{}], g: gitignore [{}], b: binary [{}], f: format [{}], n: line numbers [{}], /: search, q: quit ",
            app_state.filtered_items.len(),
            selection_status,
            // Show which toggles are active with [x] or [ ]
            if app_state.ignore_config.use_default_ignores { "x" } else { " " },
            if app_state.ignore_config.use_gitignore { "x" } else { " " },
            if app_state.ignore_config.include_binary_files { "x" } else { " " },
            // Show current output format
            match app_state.output_format {
                crate::models::OutputFormat::Xml => "XML",
                crate::models::OutputFormat::Markdown => "Markdown",
                crate::models::OutputFormat::Json => "JSON",
                crate::models::OutputFormat::Llm => "LLM",
            },
            // Show if line numbers are enabled
            if app_state.show_line_numbers { "x" } else { " " },
        );

        let block = Block::default().title(status_text).borders(Borders::ALL);
        f.render_widget(block, area);
    }
}

