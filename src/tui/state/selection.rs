// src/tui/state/selection.rs
//!
//! # Selection State Module
//!
//! Defines the selection state and logic for managing file/folder selection in the TUI. Provides utilities for toggling selection, multi-select, and folder selection logic. Enables robust selection workflows for file operations.
//!
//! ## Organization
//! - [`SelectionState`]: State struct for selection and UI list state.
//!
//! ## Usage
//! Use [`SelectionState`] to manage selection state and implement selection-related UI actions.
//!
//! # Examples
//! ```rust
//! use crate::tui::state::selection::SelectionState;
//! let mut sel = SelectionState::new();
//! sel.move_selection(1, items.len());
//! ```

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;

use crate::fs as crate_fs;
use crate::tui::state::app_state::{AppState, MessageType};
use crate::tui::app::{AppEvent, OperationId};

/// Represents the selection state for the file list, including UI and local selection.
///
/// # Fields
/// * `list_state` - The list selection state for the UI.
/// * `local_selected` - Set of locally selected paths.
///
/// # Examples
/// ```rust
/// use crate::tui::state::selection::SelectionState;
/// let mut sel = SelectionState::new();
/// sel.move_selection(1, 10);
/// ```
#[doc(alias = "selection-state")]
pub struct SelectionState {
    pub list_state: ratatui::widgets::ListState,
    // Tracking selected paths in a HashSet for efficient lookups
    #[allow(dead_code)]
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
            self.list_state.select(None);
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_selected = (current as i32 + delta).clamp(0, item_count.saturating_sub(1) as i32) as usize;
        self.list_state.select(Some(new_selected));
    }

    /// Toggles selection of the currently highlighted item, handling folders recursively.
    pub fn toggle_selection(&mut self, app_state: &mut AppState) -> io::Result<()> {
        if let Some(selected_index) = self.list_state.selected() {
            let display_items = app_state.get_display_items();
            if selected_index >= display_items.len() {
                return Ok(());
            }

            let path = display_items[selected_index].clone();

            if path.file_name().is_some_and(|name| name == "..") {
                return Ok(());
            }

            let is_selected = app_state.selected_items.contains(&path);
            let is_ignored = app_state.is_path_ignored(&path);
            let limit = app_state.selection_limit;

            if is_ignored {
                // No action
            } else if is_selected {
                app_state.optimistically_added_folder = None;
                app_state.optimistically_added_children.clear();
                if path.is_dir() {
                    // For deselection, limit is not a concern for removal
                    Self::update_folder_selection_recursive(app_state, &path, false, usize::MAX)?;
                } else {
                    app_state.selected_items.remove(&path);
                }
            } else { // Selecting a new item
                if path.is_dir() {
                    // Pre-check: Ensure we can at least add the folder itself
                    if app_state.selected_items.len() >= limit {
                        app_state.set_message(
                            format!("Selection limit ({}) reached. Cannot select folder.", limit),
                            MessageType::Warning,
                        );
                        return Ok(());
                    }

                    // Store optimistic additions for the event
                    let optimistic_folder_for_event = Some(path.clone());
                    let mut optimistic_children_for_event = HashSet::new();

                    app_state.selected_items.insert(path.clone());
                    app_state.optimistically_added_folder = Some(path.clone());
                    app_state.optimistically_added_children.clear();

                    // Pre-check for expanded folder children
                    if app_state.expanded_folders.contains(&path) {
                        let mut children_to_add_optimistically = Vec::new();
                        for item_in_list in app_state.get_display_items() {
                            if item_in_list.starts_with(&path) && item_in_list != &path {
                                children_to_add_optimistically.push(item_in_list.clone());
                            }
                        }
                        
                        // Pre-check: Ensure we don't exceed limit with visible children
                        if app_state.selected_items.len() + children_to_add_optimistically.len() > limit {
                            app_state.set_message(
                                format!(
                                    "Cannot select folder: {} visible children would exceed selection limit ({})",
                                    children_to_add_optimistically.len(), limit
                                ),
                                MessageType::Warning,
                            );
                            // Remove the folder we just added
                            app_state.selected_items.remove(&path);
                            app_state.optimistically_added_folder = None;
                            return Ok(());
                        }
                        
                        // Add children optimistically (already pre-checked)
                        for child_path_opt in children_to_add_optimistically {
                            if app_state.selected_items.insert(child_path_opt.clone()) {
                                app_state.optimistically_added_children.insert(child_path_opt.clone());
                                optimistic_children_for_event.insert(child_path_opt); // Capture for event
                            }
                        }
                    }

                    // Async counting for this folder, sending AppEvent
                    let operation_id = OperationId::new();
                    app_state.is_counting = true; // General flag for UI status
                    app_state.counting_path = Some(path.clone()); // Path being counted for UI status
                    app_state.current_operation_id = Some(operation_id); // Track operation ID

                    let p_clone = path.clone();
                    let base_dir_clone = app_state.current_dir.clone();
                    let ignore_config_clone = app_state.ignore_config.clone();
                    let limit_clone = app_state.selection_limit;
                    let event_tx_clone = app_state.tx.clone(); // Use the AppState's event sender

                    thread::spawn(move || {
                        let count_result = crate_fs::count_selection_items(
                            &p_clone,
                            &base_dir_clone,
                            &ignore_config_clone,
                            limit_clone
                        );
                        match count_result {
                            Ok(num_items) => {
                                event_tx_clone.send(AppEvent::SelectionCountComplete {
                                    operation_id,
                                    path: p_clone,
                                    count: num_items,
                                    optimistic_folder: optimistic_folder_for_event, // Pass the captured optimistic data
                                    optimistic_children: optimistic_children_for_event,
                                }).unwrap_or_else(|e| eprintln!("Failed to send SelectionCountComplete event: {}", e));
                            }
                            Err(e) => {
                                eprintln!("Error counting items for {}: {}", p_clone.display(), e);
                                // Optionally send an error event if desired, or handle silently
                                // For now, errors are logged by the thread.
                                // We might want to send a specific AppEvent::Error back to the main thread.
                            }
                        }
                    });
                } else { // File selection
                    // Pre-check: Ensure we can add the file
                    if app_state.selected_items.len() >= limit {
                        app_state.set_message(
                            format!("Cannot select file: selection limit ({}) reached.", limit),
                            MessageType::Warning,
                        );
                        return Ok(());
                    }
                    app_state.selected_items.insert(path.clone());
                }
            }
        }
        Ok(())
    }

    /// Recursively selects or deselects a folder and its contents, respecting selection limits.
    /// Renamed to avoid conflict if an old `update_folder_selection` without limit exists.
    pub fn update_folder_selection_recursive(
        app_state: &mut AppState,
        path: &Path,
        selected: bool,
        limit: usize, // Pass limit explicitly
    ) -> io::Result<()> {
        if path.is_dir() {
            if selected {
                if app_state.selected_items.len() < limit || app_state.selected_items.contains(path) {
                    app_state.selected_items.insert(path.to_path_buf());
                } else {
                    return Ok(());
                }
            } else {
                app_state.selected_items.remove(path);
            }

            if selected && app_state.selected_items.len() >= limit && !app_state.selected_items.contains(path) {
                 // If we just added the folder itself and hit the limit, don't process children.
                 // Unless the folder was already selected (e.g. part of optimistic set).
                 // This condition is tricky; the main check is before adding each child.
                 if !app_state.expanded_folders.contains(path) { // Don't stop if it was already expanded and children might be selected
                    return Ok(());
                 }
            }

            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let child_path = entry.path();
                    if selected && app_state.selected_items.len() >= limit && !app_state.selected_items.contains(&child_path) {
                        break;
                    }
                    Self::update_folder_selection_recursive(app_state, &child_path, selected, limit)?;
                }
            }
        } else if selected {
            if app_state.selected_items.len() < limit || app_state.selected_items.contains(path) {
                app_state.selected_items.insert(path.to_path_buf());
            }
        } else {
            app_state.selected_items.remove(path);
        }
        Ok(())
    }

    /// Toggles selection of all items, respecting selection limits.
    pub fn toggle_select_all(&mut self, app_state: &mut AppState) -> io::Result<()> {
        let display_items = app_state.get_display_items().to_vec();

        let all_currently_visible_selected = display_items
            .iter()
            .filter(|path| path.file_name().is_none_or(|n| n != ".."))
            .all(|path| app_state.selected_items.contains(path));

        if all_currently_visible_selected {
            app_state.selected_items.clear();
            app_state.optimistically_added_folder = None;
            app_state.optimistically_added_children.clear();
            app_state.selection_is_over_limit = false; // Also reset this flag
            if app_state.is_counting { // If any count was active (e.g. single item or old select all)
                 if let Some(sender) = app_state.count_abort_sender.take() {
                    let _ = sender.send(());
                }
                app_state.is_counting = false;
                app_state.counting_path = None;
            }
        } else {
            // ---- "SELECT ALL" LOGIC ----
            
            // Cancel any prior single item count that might be running
            if app_state.is_counting {
                 if let Some(sender) = app_state.count_abort_sender.take() {
                    let _ = sender.send(());
                }
                app_state.is_counting = false; // Reset general counting flag
                app_state.counting_path = None;
            }

            let display_items_clone = app_state.get_display_items().to_vec(); // Clone for iteration
            let mut current_optimistic_selection: HashSet<PathBuf> = HashSet::new();
            let mut folders_to_scan_deeply: Vec<PathBuf> = Vec::new();

            // 1. Pre-check: Count visible items first to avoid violating limits
            let mut visible_item_count = 0;
            for path in &display_items_clone {
                if path.file_name().is_some_and(|n| n == "..") {
                    continue;
                }
                visible_item_count += 1;
                
                if path.is_dir() && !app_state.expanded_folders.contains(path) {
                    folders_to_scan_deeply.push(path.clone());
                }
            }

            // Pre-check: If even visible items exceed limit, show warning and don't proceed
            if visible_item_count > app_state.selection_limit {
                app_state.set_message(
                    format!(
                        "Cannot select all: {} visible items exceed selection limit ({})",
                        visible_item_count, app_state.selection_limit
                    ),
                    MessageType::Warning,
                );
                return Ok(());
            }

            // Clear current selection after pre-checks pass
            app_state.selected_items.clear();
            app_state.optimistically_added_folder = None;
            app_state.optimistically_added_children.clear();
            app_state.selection_is_over_limit = false; // Reset flag

            // 2. Optimistic Phase & Identify Folders for Deep Scan (within limits)
            for path in &display_items_clone {
                if path.file_name().is_some_and(|n| n == "..") {
                    continue;
                }

                current_optimistic_selection.insert(path.clone());
            }

            // Update UI immediately with optimistic selection (already pre-checked)
            app_state.selected_items = current_optimistic_selection.clone();
            // Mark related UI elements dirty (FileList, StatusBar)
            // This would ideally be done by the App/Tui main loop after an event or state change signal

            // 2. Asynchronous Deep Scan (if needed)
            if !folders_to_scan_deeply.is_empty() {
                let operation_id = OperationId::new();
                app_state.is_counting = true;
                app_state.counting_path = None;
                app_state.current_operation_id = Some(operation_id);

                let event_tx_clone = app_state.tx.clone();
                let ignore_config_clone = app_state.ignore_config.clone();
                let gitignore_base_dir_clone = app_state.current_dir.clone();
                let initial_optimistic_set_for_thread = current_optimistic_selection.clone();

                thread::spawn(move || {
                    let mut complete_set_for_select_all = initial_optimistic_set_for_thread.clone();

                    for folder_to_scan in &folders_to_scan_deeply {
                        // Use the alias crate_fs
                        if let Err(e) = crate_fs::collect_folder_descendants(
                            folder_to_scan,
                            &gitignore_base_dir_clone,
                            &ignore_config_clone,
                            &mut complete_set_for_select_all
                        ) {
                            eprintln!("Error collecting descendants for {}: {}", folder_to_scan.display(), e);
                        }
                    }

                    let total_count = complete_set_for_select_all.len();

                    event_tx_clone.send(AppEvent::SelectAllScanComplete {
                        operation_id,
                        total_potential_item_count: total_count,
                        final_selection_set: complete_set_for_select_all,
                        initial_optimistic_set: initial_optimistic_set_for_thread,
                    }).unwrap_or_else(|e| eprintln!("Failed to send SelectAllScanComplete event: {}", e));
                });
            } else {
                // No unexpanded folders to scan, the optimistic selection is the final selection.
                app_state.is_counting = false; // Not counting anything async.
                let total_count = app_state.selected_items.len();
                if total_count > app_state.selection_limit {
                    app_state.selection_is_over_limit = true;
                    app_state.set_message(
                        format!(
                            "Selection limit ({}) exceeded. {} items selected (all visible).",
                            app_state.selection_limit, total_count
                        ),
                        MessageType::Warning,
                    );
                    // Potentially revert to empty or show warning more prominently.
                    // For now, selected_items remains, but flagged as over_limit.
                } else {
                    app_state.selection_is_over_limit = false;
                }
            }
        }
        Ok(())
    }
}

// TODO: Add visual feedback for partially selected folders.
// TODO: Add undo/redo for selection changes.
