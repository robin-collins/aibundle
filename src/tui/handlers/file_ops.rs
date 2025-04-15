//!
//! # File Operations Handler
//!
//! This module defines the `FileOpsHandler` for managing file and folder operations in the TUI.
//! It handles loading, searching, selection, toggling options, and folder expansion/collapse.
//!
//! ## Usage
//! Use `FileOpsHandler` to manage file system interactions and UI state updates in the TUI.
//!
//! ## Examples
//! ```rust
//! use crate::tui::handlers::FileOpsHandler;
//! FileOpsHandler::load_items(&mut app_state).unwrap();
//! FileOpsHandler::toggle_folder_expansion(&mut app_state, &selection_state).unwrap();
//! ```

// src/tui/handlers/file_ops.rs
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::config::{config_file_path, save_config};
use crate::fs::{add_items_recursively, is_path_ignored_iterative};
use crate::models::{CopyStats, OutputFormat};
use crate::output::format_selected_items;
use crate::tui::state::AppState;
use crate::tui::state::{SearchState, SelectionState};

/// Handler for file and folder operations in the TUI.
pub struct FileOpsHandler;

impl FileOpsHandler {
    /// Loads items (files/folders) into the application state, including parent navigation.
    pub fn load_items(app_state: &mut AppState) -> io::Result<()> {
        app_state.items.clear();

        // Add ".." entry if not at the root
        if let Some(parent) = app_state.current_dir.parent() {
            // Check if parent is different from current_dir to avoid adding ".." at root
            if parent != app_state.current_dir {
                // Check if parent path is valid before adding ".."
                // This avoids adding ".." for paths like "/" where parent is also "/"
                if parent.parent().is_some() || parent == Path::new("/") {
                    // Simplified root check
                    app_state.items.push(app_state.current_dir.join(".."));
                }
            }
        }

        // Use the add_items_recursively function from fs module to populate app_state.items
        add_items_recursively(
            &mut app_state.items,
            &app_state.current_dir,
            &app_state.expanded_folders,
            &app_state.ignore_config,
            &app_state.current_dir, // base_dir for ignore checks relative to current view
        )?;

        // Always update filtered_items after loading.
        // If a search is active, update_search should be called subsequently
        // to apply the filter correctly to the newly loaded items.
        app_state.filtered_items = app_state.items.clone();

        // Reset selection if items list is not empty, otherwise clear selection
        if !app_state.items.is_empty() {
            // TODO: Preserve selection if possible/desired? For now, reset.
            // selection_state.list_state.select(Some(0)); // Requires mutable selection_state
        } else {
            // selection_state.list_state.select(None); // Requires mutable selection_state
        }

        Ok(())
    }

    /// Loads only the current directory (non-recursive) into the application state.
    pub fn load_items_nonrecursive(app_state: &mut AppState) -> io::Result<()> {
        app_state.items.clear();
        app_state.filtered_items.clear();

        // Add parent directory entry if applicable
        if let Some(parent) = app_state.current_dir.parent() {
            if !parent.as_os_str().is_empty() {
                app_state.items.push(app_state.current_dir.join(".."));
            }
        }

        // Read only the current directory, no recursion
        let entries = fs::read_dir(&app_state.current_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                !is_path_ignored_iterative(p, &app_state.current_dir, &app_state.ignore_config)
            })
            .collect::<Vec<_>>();

        // Sort entries (directories first, then files)
        let mut sorted_entries = entries;
        sorted_entries.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        app_state.items.extend(sorted_entries);
        app_state.filtered_items = app_state.items.clone();
        Ok(())
    }

    /// Updates the search results in the application state based on the search query.
    pub fn update_search(
        app_state: &mut AppState,
        search_state: &mut SearchState,
    ) -> io::Result<()> {
        app_state.search_query = search_state.search_query.clone();
        app_state.is_searching = !search_state.search_query.is_empty();

        if !app_state.is_searching {
            app_state.filtered_items = app_state.items.clone();
            return Ok(());
        }

        // Create a matcher function based on the search query
        let matcher = search_state.create_matcher();

        // If not in recursive mode, filter only the current items (non-recursive filtering)
        if !app_state.recursive {
            app_state.filtered_items = app_state
                .items
                .iter()
                .filter(|&p| p.file_name().and_then(|n| n.to_str()).is_some_and(&matcher))
                .cloned()
                .collect();
            return Ok(());
        }

        // Otherwise, perform recursive search
        let max_depth = 4;
        let mut results = HashSet::new();

        // Recursively search each immediate child of the current directory
        if let Ok(entries) = fs::read_dir(&app_state.current_dir) {
            for entry in entries.filter_map(|e| e.ok()).map(|e| e.path()) {
                if !is_path_ignored_iterative(
                    &entry,
                    &app_state.current_dir,
                    &app_state.ignore_config,
                ) {
                    crate::fs::recursive_search_helper_generic(
                        app_state,
                        &entry,
                        1,
                        max_depth,
                        &matcher,
                        &mut results,
                    );
                }
            }
        }

        let mut matched: Vec<PathBuf> = results.into_iter().collect();
        matched.sort_by_key(|p| {
            p.strip_prefix(&app_state.current_dir)
                .map(|r| r.to_string_lossy().into_owned())
                .unwrap_or_default()
        });

        app_state.filtered_items = matched;

        // Ensure that the full hierarchy is visible by expanding parent folders
        let mut parents_to_expand = HashSet::new();
        for item in &app_state.filtered_items {
            let mut current = item.as_path();
            while let Some(parent) = current.parent() {
                if parent == app_state.current_dir
                    || parent == Path::new("/")
                    || parent == Path::new("")
                {
                    break;
                }
                if !app_state.expanded_folders.contains(parent)
                    && parent.starts_with(&app_state.current_dir)
                {
                    parents_to_expand.insert(parent.to_path_buf());
                }
                current = parent;
            }
        }

        app_state.expanded_folders.extend(parents_to_expand);

        // Reload items to reflect newly expanded folders during search
        Self::load_items(app_state)?;

        Ok(())
    }

    /// Formats the selected items for output and updates last_copy_stats.
    pub fn format_selected_items(app_state: &mut AppState) -> io::Result<String> {
        let result = format_selected_items(
            &app_state.selected_items,
            &app_state.current_dir,
            &app_state.output_format,
            app_state.show_line_numbers,
            &app_state.ignore_config,
        )?;

        // Update last_copy_stats with the result statistics
        app_state.last_copy_stats = Some(CopyStats {
            files: result.1.files,
            folders: result.1.folders,
        });

        Ok(result.0)
    }

    /// Handles Enter key: navigates into directories or up to parent.
    pub fn handle_enter(
        app_state: &mut AppState,
        selection_state: &mut SelectionState,
    ) -> io::Result<()> {
        if let Some(selected) = selection_state.list_state.selected() {
            if selected >= app_state.filtered_items.len() {
                return Ok(());
            }

            let path = &app_state.filtered_items[selected];
            if path.is_dir() {
                if path.ends_with("..") {
                    if let Some(parent) = app_state.current_dir.parent() {
                        app_state.current_dir = parent.to_path_buf();
                    }
                } else {
                    app_state.current_dir = path.clone();
                }

                app_state.is_searching = false;
                app_state.search_query.clear();

                Self::load_items(app_state)?;
                selection_state.list_state.select(Some(0));
            } else {
                // Handle file selection if needed (e.g., open file, show preview)
                // Currently, Enter on a file does nothing.
            }
        }
        Ok(())
    }

    /// Toggles the use of default ignores and reloads items.
    pub fn toggle_default_ignores(app_state: &mut AppState) -> io::Result<()> {
        app_state.ignore_config.use_default_ignores = !app_state.ignore_config.use_default_ignores;
        Self::load_items(app_state)
    }

    /// Toggles the use of .gitignore and reloads items.
    pub fn toggle_gitignore(app_state: &mut AppState) -> io::Result<()> {
        app_state.ignore_config.use_gitignore = !app_state.ignore_config.use_gitignore;
        Self::load_items(app_state)
    }

    /// Toggles the inclusion of binary files and reloads items.
    pub fn toggle_binary_files(app_state: &mut AppState) -> io::Result<()> {
        app_state.ignore_config.include_binary_files =
            !app_state.ignore_config.include_binary_files;
        Self::load_items(app_state)
    }

    /// Toggles the output format (XML, Markdown, JSON, LLM).
    pub fn toggle_output_format(app_state: &mut AppState) -> io::Result<()> {
        app_state.output_format = app_state.output_format.toggle();
        Ok(())
    }

    /// Toggles line numbers for output (except in JSON mode).
    pub fn toggle_line_numbers(app_state: &mut AppState) -> io::Result<()> {
        // Don't toggle line numbers if we're in JSON mode
        if app_state.output_format != OutputFormat::Json {
            app_state.show_line_numbers = !app_state.show_line_numbers;
        }
        Ok(())
    }

    /// Saves the current configuration to disk, warning if file exists.
    pub fn save_config(app_state: &mut AppState) -> io::Result<()> {
        let config_path = config_file_path()?;
        if config_path.exists() {
            // Prompt for overwrite (not interactive in TUI, so just warn)
            app_state.set_message(
                format!("Configuration file already exists: {}", config_path.display()),
                crate::tui::state::MessageType::Warning,
            );
            // In a real TUI, you might want to prompt the user for confirmation
            // For now, just return an error
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Config file exists"));
        }
        save_config(&app_state.config, config_path.to_str().unwrap_or(""))?;
        // Success message
        app_state.set_message(
            format!("Configuration saved successfully to {}", config_path.display()),
            crate::tui::state::MessageType::Success,
        );
        Ok(())
    }

    /// Checks for pending selection count results and updates selection state.
    pub fn check_pending_selection(
        app_state: &mut AppState,
        _selection_state: &mut SelectionState,
    ) -> io::Result<()> {
        // Check for pending selection count results
        if app_state.is_counting {
            if let Some(rx) = &app_state.pending_count {
                if let Ok(Ok(count)) = rx.try_recv() {
                    if count <= app_state.selection_limit {
                        if let Some(path) = app_state.counting_path.take() {
                            if path.is_dir() {
                                SelectionState::update_folder_selection(app_state, &path, true)?;
                            } else {
                                app_state.selected_items.insert(path);
                            }
                        }
                    } else {
                        app_state.modal = Some(crate::tui::components::Modal::new(
                            format!(
                                "Cannot select: would exceed limit of {} items\nTried to add {} items",
                                app_state.selection_limit, count
                            ),
                            50,
                            4,
                        ));
                    }
                    app_state.is_counting = false;
                    app_state.pending_count = None;
                }
            }
        }
        Ok(())
    }

    /// Shows the help modal in the application.
    pub fn show_help(app_state: &mut AppState) -> io::Result<()> {
        app_state.modal = Some(crate::tui::components::Modal::help());
        Ok(())
    }

    /// Toggles expansion/collapse of a folder in the file list.
    pub fn toggle_folder_expansion(
        app_state: &mut AppState,
        selection_state: &SelectionState,
    ) -> io::Result<()> {
        if let Some(selected_index) = selection_state.list_state.selected() {
            if selected_index < app_state.filtered_items.len() {
                let path = &app_state.filtered_items[selected_index];
                if path.is_dir() && !path.ends_with("..") {
                    let path_buf = path.to_path_buf();
                    if app_state.expanded_folders.contains(&path_buf) {
                        app_state.expanded_folders.remove(&path_buf);
                    } else {
                        app_state.expanded_folders.insert(path_buf);
                    }
                    Self::load_items(app_state)?;
                }
            }
        }
        Ok(())
    }

    // Helper function for recursive expansion
    fn expand_all(app_state: &mut AppState, path: &PathBuf) -> io::Result<()> {
        if !path.is_dir() || app_state.is_path_ignored(path) {
            return Ok(());
        }

        app_state.expanded_folders.insert(path.clone());

        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.filter_map(|e| e.ok()) {
                    let child_path = entry.path();
                    if child_path.is_dir() && !app_state.is_path_ignored(&child_path) {
                        // Check if the child path is already expanded to prevent infinite loops in case of symlinks
                        if !app_state.expanded_folders.contains(&child_path) {
                            Self::expand_all(app_state, &child_path)?;
                        }
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                // Ignore permission denied errors and continue
            }
            Err(e) => return Err(e), // Propagate other errors
        }

        Ok(())
    }

    // Helper function for recursive collapse
    fn collapse_all(app_state: &mut AppState, path: &PathBuf) -> io::Result<()> {
        if !path.is_dir() {
            return Ok(());
        }

        // Use retain to efficiently remove the path and all its descendants
        // Convert path to string to perform prefix check efficiently
        if let Some(path_str) = path.to_str() {
            app_state.expanded_folders.retain(|expanded_path| {
                // Keep paths that are not the target path itself and do not start with the target path's prefix
                expanded_path != path
                    && expanded_path
                        .to_str()
                        .map_or(true, |ep_str| !ep_str.starts_with(path_str))
            });
        } else {
            // Fallback if path is not valid UTF-8 (less likely but good to handle)
            app_state.expanded_folders.remove(path);
            // This fallback won't remove children, but it's better than erroring.
        }

        Ok(())
    }

    /// Toggles recursive expansion/collapse of a folder and its descendants.
    pub fn toggle_folder_expansion_recursive(
        app_state: &mut AppState,
        selection_state: &mut SelectionState,
    ) -> io::Result<()> {
        if let Some(selected_index) = selection_state.list_state.selected() {
            if selected_index < app_state.filtered_items.len() {
                let path = &app_state.filtered_items[selected_index];
                if path.is_dir() && !path.ends_with("..") {
                    let path_buf = path.to_path_buf();
                    if app_state.expanded_folders.contains(&path_buf) {
                        // Currently expanded, so collapse all descendants
                        Self::collapse_all(app_state, &path_buf)?;
                    } else {
                        // Currently collapsed, so expand all descendants
                        Self::expand_all(app_state, &path_buf)?;
                    }
                    // Reload items to reflect changes
                    Self::load_items(app_state)?;
                    // Try to keep the selection, might need adjustment if the item disappears
                    selection_state.list_state.select(Some(
                        selected_index.min(app_state.filtered_items.len().saturating_sub(1)),
                    ));
                }
            }
        }
        Ok(())
    }
}

// TODO: Add support for file previews on selection.
// TODO: Add support for batch operations (move, delete, rename).
// TODO: Add undo/redo for file operations.
