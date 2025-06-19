// src/tui/components/file_list.rs
//!
//! # File List Component
//!
//! Provides the [`FileList`] component for rendering a navigable, selectable list of files and folders in the TUI.
//!
//! ## Purpose
//!
//! - Display directory contents with icons, indentation, and selection state.
//! - Support navigation, selection, and visual feedback for file/folder operations.
//!
//! ## Organization
//!
//! - [`FileList`]: Main component for rendering file/folder lists.
//! - Helper: `get_icon` for icon selection.
//!
//! ## Example
//! ```rust
//! use crate::tui::components::FileList;
//! use crate::tui::state::{AppState, SelectionState};
//! # use ratatui::{backend::TestBackend, Terminal, layout::Rect, Frame};
//! # let mut app_state = AppState::default_for_test();
//! # let mut selection_state = SelectionState::new();
//! # let backend = TestBackend::new(80, 24);
//! # let mut terminal = Terminal::new(backend).unwrap();
//! # let area = Rect::new(0, 0, 80, 24);
//! let file_list = FileList::new();
//! terminal.draw(|f| {
//!     file_list.render(f, area, &app_state, &mut selection_state);
//! }).unwrap();
//! ```
//!
//! # Doc Aliases
//! - "file-list"
//! - "file-browser"
//!
#![doc(alias = "file-list")]
#![doc(alias = "file-browser")]

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use std::path::Path;

use crate::models::constants::ICONS;
use crate::tui::state::{AppState, SelectionState};

/// File list component for rendering files and folders in the TUI.
///
/// # Purpose
/// Provides a scrollable, selectable list of files and folders, with icons and indentation reflecting directory structure.
///
/// # Examples
/// ```rust
/// use crate::tui::components::FileList;
/// let file_list = FileList::new();
/// # // See module-level example for full usage
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct FileList;

impl FileList {
    /// Creates a new [`FileList`] component.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::FileList;
    /// let file_list = FileList::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Renders the file list in the given area, using the current app and selection state.
    ///
    /// # Arguments
    /// * `f` - The TUI frame to render into.
    /// * `area` - The area to render the file list in.
    /// * `app_state` - The current application state.
    /// * `selection_state` - The current selection state.
    ///
    /// # Panics
    /// This function does not panic.
    ///
    /// # Examples
    /// ```rust
    /// # use crate::tui::components::FileList;
    /// # use crate::tui::state::{AppState, SelectionState};
    /// # use ratatui::{backend::TestBackend, Terminal, layout::Rect};
    /// # let mut app_state = AppState::default_for_test();
    /// # let mut selection_state = SelectionState::new();
    /// # let backend = TestBackend::new(80, 24);
    /// # let mut terminal = Terminal::new(backend).unwrap();
    /// # let area = Rect::new(0, 0, 80, 24);
    /// let file_list = FileList::new();
    /// terminal.draw(|f| {
    ///     file_list.render(f, area, &app_state, &mut selection_state);
    /// }).unwrap();
    /// ```
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState,
        selection_state: &mut SelectionState,
    ) {
        let display_items = app_state.get_display_items();

        let items: Vec<ListItem> = display_items
            .iter()
            .enumerate()
            .map(|(index, path)| {
                // Calculate indentation based on directory depth
                let depth = path
                    .strip_prefix(&app_state.current_dir)
                    .map(|p| p.components().count())
                    .unwrap_or(0)
                    .saturating_sub(1);
                let indent = "  ".repeat(depth);

                let is_dot_dot = path.ends_with("..");

                let display_name = if is_dot_dot {
                    let folder_icon = ICONS
                        .iter()
                        .find(|(k, _)| *k == "folder")
                        .map(|(_, v)| *v)
                        .unwrap_or("📁");
                    // folder_icon (1 char) + 3 spaces for alignment with "[ ] " part of other entries
                    format!("{}{}   ../", indent, folder_icon)
                } else {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("???")
                        .to_string();

                    let icon = Self::get_icon(path);
                    let prefix = if app_state.selected_items.contains(path) {
                        "[X] "
                    } else {
                        "[ ] "
                    };

                    if path.is_dir() {
                        format!("{}{}{} {}/", indent, prefix, icon, name)
                    } else {
                        format!("{}{}{} {}", indent, prefix, icon, name)
                    }
                };

                let is_selected_line = selection_state.list_state.selected() == Some(index);
                let style = if is_selected_line {
                    Style::default()
                        .bg(Color::Gray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(display_name).style(style)
            })
            .collect();

        let list_widget =
            List::new(items).block(Block::default().title("Files").borders(Borders::ALL));

        f.render_stateful_widget(list_widget, area, &mut selection_state.list_state);
    }

    /// Returns the icon for a given path, based on file extension or directory status.
    ///
    /// # Arguments
    /// * `path` - The file or directory path.
    ///
    /// # Returns
    /// * `&'static str` - The icon string for the file or directory.
    ///
    /// # Examples
    /// ```rust
    /// # use std::path::Path;
    /// # use crate::tui::components::FileList;
    /// let icon = FileList::get_icon(Path::new("foo.rs"));
    /// assert!(icon.len() > 0);
    /// ```
    fn get_icon(path: &Path) -> &'static str {
        if path.is_dir() {
            return ICONS
                .iter()
                .find(|(k, _)| *k == "folder")
                .map(|(_, v)| *v)
                .unwrap_or("📁");
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| ICONS.iter().find(|(k, _)| *k == ext))
            .map(|(_, v)| *v)
            .unwrap_or(
                ICONS
                    .iter()
                    .find(|(k, _)| *k == "default")
                    .map(|(_, v)| *v)
                    .unwrap_or("📄"),
            )
    }
}

// TODO: Add support for custom icons or color themes.
// TODO: Add file size or modified date display in the file list.
