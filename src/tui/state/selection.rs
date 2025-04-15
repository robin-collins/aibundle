// src/tui/state/selection.rs
//!
//! # Selection State Module
//!
//! This module defines the selection state and logic for managing file/folder selection in the TUI.
//! It provides utilities for toggling selection, multi-select, and folder selection logic.
//!
//! ## Usage
//! Use `SelectionState` to manage selection state and implement selection-related UI actions.
//!
//! ## Examples
//! ```rust
//! use crate::tui::state::selection::SelectionState;
//! let mut sel = SelectionState::new();
//! sel.move_selection(1, items.len());
//! ```

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use crate::fs as crate_fs;
use crate::tui::state::AppState;

/// Represents the selection state for the file list, including UI and local selection.
pub struct SelectionState {
    pub list_state: ratatui::widgets::ListState,
    // Tracking selected paths in a HashSet for efficient lookups
    pub local_selected: HashSet<PathBuf>,
}

impl Default for SelectionState {
    /// Returns a default-initialized `SelectionState` with the first item selected.
    fn default() -> Self {
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(0));
        Self {
            list_state,
            local_selected: HashSet::new(),
        }
    }
}

impl SelectionState {
    /// Creates a new, default `SelectionState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves the selection up or down by the given delta, clamped to the item count.
    pub fn move_selection(&mut self, delta: i32, item_count: usize) {
        if item_count == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_selected = (current as i32 + delta).clamp(0, item_count as i32 - 1) as usize;
        self.list_state.select(Some(new_selected));
    }

    /// Toggles selection of the currently highlighted item, handling folders recursively.
    pub fn toggle_selection(&mut self, app_state: &mut AppState) -> io::Result<()> {
        if let Some(selected_index) = self.list_state.selected() {
            if selected_index >= app_state.filtered_items.len() {
                return Ok(());
            }

            let path = app_state.filtered_items[selected_index].clone();
            if path.file_name().is_some_and(|n| n == "..") {
                return Ok(());
            }

            let is_selected = app_state.selected_items.contains(&path);

            // If already selected, unselect immediately (no counting needed)
            if is_selected {
                if path.is_dir() {
                    Self::update_folder_selection(app_state, &path, false)?;
                } else {
                    app_state.selected_items.remove(&path);
                }
                return Ok(());
            }

            // If not selected, start an async count
            if !app_state.is_counting {
                let (tx, rx) = mpsc::channel();
                let base_path = app_state.current_dir.clone();
                let ignore_config = app_state.ignore_config.clone();
                let path_clone = path.clone();
                let selection_limit = app_state.selection_limit;

                // Use IgnoreConfig (from app_config.rs) to check if path should be ignored before counting
                if !app_state.is_path_ignored(&path) {
                    thread::spawn(move || {
                        let result = crate_fs::count_selection_items(
                            &path_clone,
                            &base_path,
                            &ignore_config,
                            selection_limit,
                        );
                        let _ = tx.send(result);
                    });

                    app_state.pending_count = Some(rx);
                    app_state.counting_path = Some(path);
                    app_state.is_counting = true;
                }
            }
        }

        Ok(())
    }

    /// Recursively selects or deselects a folder and its contents.
    pub fn update_folder_selection(
        app_state: &mut AppState,
        path: &Path,
        selected: bool,
    ) -> io::Result<()> {
        if path.is_dir() {
            if selected {
                app_state.selected_items.insert(path.to_path_buf());
            } else {
                app_state.selected_items.remove(path);
            }

            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let child_path = entry.path();
                    if child_path.is_dir() {
                        Self::update_folder_selection(app_state, &child_path, selected)?;
                    } else if selected {
                        app_state.selected_items.insert(child_path);
                    } else {
                        app_state.selected_items.remove(&child_path);
                    }
                }
            }
        } else if selected {
            app_state.selected_items.insert(path.to_path_buf());
        } else {
            app_state.selected_items.remove(path);
        }

        Ok(())
    }

    /// Toggles selection of all items in the filtered list, handling folders recursively and selection limits.
    pub fn toggle_select_all(&mut self, app_state: &mut AppState) -> io::Result<()> {
        // Check if all items are already selected
        let all_selected = app_state
            .filtered_items
            .iter()
            .filter(|path| path.file_name().is_none_or(|n| n != ".."))
            .all(|path| app_state.selected_items.contains(path));

        if all_selected {
            app_state.selected_items.clear();
        } else {
            // Collect paths to process before modifying app_state to avoid borrow issues
            let paths_to_process: Vec<PathBuf> = app_state
                .filtered_items
                .iter()
                .filter(|path| path.file_name().is_none_or(|n| n != ".."))
                .filter(|path| {
                    path.is_dir()
                        && (app_state.recursive || app_state.expanded_folders.contains(*path))
                })
                .cloned()
                .collect();

            // Select all items in filtered_items, except ".."
            for path in &app_state.filtered_items {
                if path.file_name().is_some_and(|n| n == "..") {
                    continue;
                }
                app_state.selected_items.insert(path.clone());
            }

            // Process collected directory paths
            for path in &paths_to_process {
                Self::update_folder_selection(app_state, path, true)?;
            }

            // Count total selected items for the warning
            if !app_state.is_counting && !app_state.filtered_items.is_empty() {
                let (tx, rx) = mpsc::channel();
                let counting_path = app_state.current_dir.clone();
                let base_dir = app_state.current_dir.clone();
                let ignore_config = app_state.ignore_config.clone();
                let selection_limit = app_state.selection_limit;

                // Clone before moving into the closure
                let counting_path_for_closure = counting_path.clone();

                // Spawn a background thread to count items
                thread::spawn(move || {
                    let result = crate_fs::count_selection_items(
                        &counting_path_for_closure,
                        &base_dir,
                        &ignore_config,
                        selection_limit,
                    );
                    let _ = tx.send(result);
                });

                app_state.pending_count = Some(rx);
                app_state.counting_path = Some(counting_path);
                app_state.is_counting = true;
            }
        }

        Ok(())
    }
}

// TODO: Add support for range selection (shift+click or shift+arrow).
// TODO: Add visual feedback for partially selected folders.
// TODO: Add undo/redo for selection changes.
