// src/tui/components/file_list.rs
//!
//! # File List Component
//!
//! This module defines the `FileList` component for rendering the file/folder list in the TUI.
//! It handles display, selection highlighting, icons, and indentation for directory structure.
//!
//! ## Usage
//! Use `FileList` in the main TUI view to render the current directory contents and selection state.
//!
//! ## Examples
//! ```rust
//! use crate::tui::components::FileList;
//! let file_list = FileList::new();
//! file_list.render(f, area, app_state, selection_state);
//! ```

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
pub struct FileList;

impl Default for FileList {
    fn default() -> Self {
        Self::new()
    }
}

impl FileList {
    /// Creates a new `FileList` component.
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
