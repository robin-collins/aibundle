// src/tui/components/status_bar.rs
//!
//! # Status Bar Component
//!
//! This module defines the `StatusBar` component for rendering the status bar at the bottom of the TUI.
//! It displays item counts, selection info, and key command hints.
//!
//! ## Usage
//! Use `StatusBar` in the main TUI view to render the bottom status bar with dynamic information.
//!
//! ## Examples
//! ```rust
//! use crate::tui::components::StatusBar;
//! let status_bar = StatusBar::new();
//! status_bar.render(f, area, app_state, selection_state);
//! ```

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::tui::state::AppState;

/// Status bar component for rendering item counts, selection info, and key hints.
pub struct StatusBar;

impl StatusBar {
    /// Creates a new `StatusBar` component.
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
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState,
        _selection_state: &crate::tui::state::SelectionState,
    ) {
        let status_text = format!(
            // Show item and selection counts, and status of toggles
            " {} items ({} selected) - Space: select, Enter: open dir, c: copy, i: ignores [{}], g: gitignore [{}], b: binary [{}], f: format [{}], n: line numbers [{}], /: search, q: quit ",
            app_state.filtered_items.len(),
            app_state.selected_items.len(),
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

// TODO: Add support for displaying error or warning messages in the status bar.
// TODO: Add customizable key command hints or help popups.
