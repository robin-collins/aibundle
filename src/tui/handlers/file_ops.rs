//!
//! # File Operations Handler
//!
//! Provides file and folder management for the TUI, including loading, searching, selection, toggling options, and folder expansion/collapse. This module is central to file system navigation and state management in the TUI.
//!
//! ## Organization
//! - `FileOpsHandler`: Main handler struct for file/folder operations.
//! - Helper functions for expansion/collapse and config management.
//!
//! ## Usage
//! Use [`FileOpsHandler`] to manage file system interactions and UI state updates in the TUI.
//!
//! # Examples
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

use crate::config::config_file_path;
use crate::models::{OutputFormat};
use crate::tui::state::AppState;
use crate::tui::state::{SearchState, SelectionState};
use crate::utils::log_event;
use crate::utils::write_selection_limit_debug_log;

/// Handler for file and folder operations in the TUI.
///
/// # Purpose
/// Provides methods for loading, searching, selecting, and managing files and folders in the TUI, including config save/restore and folder expansion logic.
///
/// # Examples
/// ```rust
/// use crate::tui::handlers::FileOpsHandler;
/// FileOpsHandler::load_items(&mut app_state).unwrap();
/// ```
#[doc(alias = "file-ops")]
pub struct FileOpsHandler;

impl FileOpsHandler {
    /// Loads items (files/folders) into the application state.
    ///
    /// # Arguments
    /// * `app_state` - Mutable reference to [`AppState`].
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(io::Error)` if loading fails.
    ///
    /// # Errors
    /// Returns an error if directory traversal fails.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::handlers::FileOpsHandler;
    /// FileOpsHandler::load_items(&mut app_state).unwrap();
    /// ```
    #[doc(alias = "file-ops-load")]
    pub fn load_items(app_state: &mut AppState) -> io::Result<()> {
        log_event(&format!(
            "FileOpsHandler::load_items: current_dir={}",
            app_state.current_dir.display()
        ));

        // Delegate all loading, sorting, and ".." handling to AppState.
        // AppState::load_items() will internally call update_filtered_items().
        app_state.load_items()
        // No direct manipulation of app_state.filtered_items or app_state.items here.
        // No custom sorting here.
        // No adding ".." here.
    }

    /// Loads only the current directory (non-recursive) into the application state.
    /// This function now delegates to `app_state.load_items()` after ensuring
    /// the `app_state.recursive` flag is false (though `app_state.load_items`
    /// should already respect this flag if it was set prior to this call).
    /// For clarity, if this function is called, it implies a non-recursive view is desired.
    pub fn load_items_nonrecursive(app_state: &mut AppState) -> io::Result<()> {
        log_event(&format!(
            "FileOpsHandler::load_items_nonrecursive: current_dir={}",
            app_state.current_dir.display()
        ));

        // Ensure the state reflects non-recursive mode if this specific function is called.
        // However, AppState::load_items already checks app_state.recursive.
        // If this function is meant to *force* non-recursive, it should set app_state.recursive = false.
        // Assuming it's called when app_state.recursive is already appropriately set.

        // Delegate all loading, sorting, and ".." handling to AppState.
        app_state.load_items()
        // No direct manipulation of app_state.filtered_items or app_state.items here.
        // No custom sorting here.
        // No adding ".." here.
    }

    /// Updates the search results in the application state based on the search query.
    ///
    /// # Arguments
    /// * `app_state` - Mutable reference to [`AppState`].
    /// * `search_state` - Mutable reference to [`SearchState`].
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(io::Error)` if search update fails.
    ///
    /// # Errors
    /// Returns an error if directory traversal fails during search.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::handlers::FileOpsHandler;
    /// FileOpsHandler::update_search(&mut app_state, &mut search_state).unwrap();
    /// ```
    #[doc(alias = "file-ops-search")]
    pub fn update_search(
        app_state: &mut AppState,
        search_state: &mut SearchState,
    ) -> io::Result<()> {
        // Memory optimization: Use clone_from for more efficient string copying
        app_state
            .search_query
            .clone_from(&search_state.search_query);
        app_state.is_searching = !search_state.search_query.is_empty();

        if !app_state.is_searching {
            // Recompute proper display list (handles '..', sorting, absolute paths)
            // instead of using items.to_vec() which produces relative paths from Trie
            Self::load_items(app_state)?;
            return Ok(());
        }

        // Create a matcher function based on the search query
        let matcher = search_state.create_matcher();

        // If not in recursive mode, filter only the current items (non-recursive filtering)
        if !app_state.recursive {
            // Load items properly first to get absolute paths, then filter
            Self::load_items(app_state)?;
            app_state.filtered_items = app_state
                .filtered_items
                .clone()
                .into_iter()
                .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(&matcher))
                .collect();
            return Ok(());
        }

        // Otherwise, perform recursive search
        let max_depth = 4;
        let mut results = HashSet::new();

        // Recursively search each immediate child of the current directory
        if let Ok(entries) = fs::read_dir(&app_state.current_dir) {
            for entry in entries.filter_map(|e| e.ok()).map(|e| e.path()) {
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

    /// Handles Enter key: navigates into directories or up to parent.
    ///
    /// # Arguments
    /// * `app_state` - Mutable reference to [`AppState`].
    /// * `selection_state` - Mutable reference to [`SelectionState`].
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(io::Error)` if navigation fails.
    ///
    /// # Errors
    /// Returns an error if directory navigation fails.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::handlers::FileOpsHandler;
    /// FileOpsHandler::handle_enter(&mut app_state, &mut selection_state).unwrap();
    /// ```
    #[doc(alias = "file-ops-enter")]
    pub fn handle_enter(
        app_state: &mut AppState,
        selection_state: &mut SelectionState,
    ) -> io::Result<()> {
        if let Some(selected) = selection_state.list_state.selected() {
            if selected >= app_state.filtered_items.len() {
                return Ok(());
            }

            let path = &app_state.filtered_items[selected];
            log_event(&format!(
                "handle_enter: selected={} current_dir={}",
                path.display(),
                app_state.current_dir.display()
            ));
            if path.is_dir() {
                if path.ends_with("..") {
                    if let Some(parent) = app_state.current_dir.parent() {
                        log_event(&format!(
                            "handle_enter: going up to parent {}",
                            parent.display()
                        ));
                        app_state.current_dir = parent.to_path_buf();
                        // Clear gitignore cache when changing directories
                        crate::fs::clear_gitignore_cache();
                    }
                } else {
                    log_event(&format!("handle_enter: entering dir {}", path.display()));
                    // Memory optimization: Use clone_from for more efficient PathBuf copying
                    app_state.current_dir.clone_from(path);
                    // Clear gitignore cache when changing directories
                    crate::fs::clear_gitignore_cache();
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

    /// Saves the current configuration to disk, prompting for overwrite if file exists.
    pub fn save_config(app_state: &mut AppState) -> io::Result<()> {
        let config_path = config_file_path()?;
        if config_path.exists() {
            // Show confirmation modal for overwrite
            app_state.modal = Some(crate::tui::components::Modal::config_overwrite_confirmation(&config_path));
            app_state.pending_save_config_path = Some(config_path);
            return Ok(());
        }

        // File doesn't exist, proceed with save
        Self::perform_config_save(app_state, &config_path)
    }

    /// Actually performs the config save operation (TUI-specific, bypasses terminal confirmation)
    pub fn perform_config_save(app_state: &mut AppState, config_path: &std::path::Path) -> io::Result<()> {
        // Create config from current app state
        let config = crate::models::AppConfig {
            default_format: Some(format!("{:?}", app_state.output_format).to_lowercase()),
            default_gitignore: Some(app_state.ignore_config.use_gitignore),
            default_ignore: Some(app_state.ignore_config.extra_ignore_patterns.clone()),
            default_line_numbers: Some(app_state.show_line_numbers),
            default_recursive: Some(app_state.recursive),
            selection_limit: Some(app_state.selection_limit),
        };

        // Serialize config to TOML string
        let toml_str = toml::to_string_pretty(&config)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("TOML serialize error: {e}")))?;

        // Perform atomic write operation to prevent data corruption
        crate::config::atomic_write_config(config_path, &toml_str)?;

        // Success message
        app_state.set_message(
            format!(
                "Configuration saved successfully to {}",
                config_path.display()
            ),
            crate::tui::state::MessageType::Success,
        );
        Ok(())
    }

    /// Handles confirmation response for config save
    pub fn handle_save_config_confirmation(app_state: &mut AppState, confirmed: bool) -> io::Result<()> {
        if let Some(config_path) = app_state.pending_save_config_path.take() {
            app_state.modal = None; // Close the confirmation modal

            if confirmed {
                // User confirmed overwrite
                Self::perform_config_save(app_state, &config_path)
            } else {
                // User cancelled
                app_state.set_message(
                    "Configuration save cancelled".to_string(),
                    crate::tui::state::MessageType::Info,
                );
                Ok(())
            }
        } else {
            // No pending save operation
            Ok(())
        }
    }

    /// Finalizes a single item selection after its count has been processed.
    /// This is called when an AppEvent::SelectionCountComplete is received.
    pub fn finalize_single_selection(
        app_state: &mut AppState,
        path_counted: PathBuf,
        num_items_in_op: usize,
        original_optimistic_folder: Option<PathBuf>,
        original_optimistic_children: HashSet<PathBuf> // Passed from the event
    ) {
        // Logic extracted from the old check_pending_selection, adapted for direct parameters
        app_state.is_counting = false; // Now that processing is done for this item
        // counting_path was for the item being processed, clear it or let AppEvent handler do it.
        // AppState.counting_path should be cleared by the AppEvent handler in app.rs if it was set for this specific op.

        if let Some(folder_path) = &original_optimistic_folder { // Ensure this is the correct var name
            app_state.selected_items.remove(folder_path);
        }
        for child_path in &original_optimistic_children {
            app_state.selected_items.remove(child_path);
        }

        // Use clippy suggestion for map_or, with correct dereferencing
        let items_to_truly_deselect: Vec<PathBuf> = app_state.selected_items.iter()
            // p is &&PathBuf, actual_folder is &PathBuf. Compare *p with actual_folder.
            // original_optimistic_children is HashSet<PathBuf>, contains needs &PathBuf.
            .filter(|p| (original_optimistic_folder.as_ref() != Some(p)) && !original_optimistic_children.contains(*p))
            .cloned()
            .collect();

        let count_of_other_selected_items = items_to_truly_deselect.len();

        if count_of_other_selected_items + num_items_in_op <= app_state.selection_limit {
            if path_counted.is_dir() { // Only call update_folder_selection_recursive if it was a directory
                // The path_counted itself should already be in selected_items if it was an optimistic add.
                // update_folder_selection_recursive will add its children.
                if let Err(e) = SelectionState::update_folder_selection_recursive(app_state, &path_counted, true, app_state.selection_limit) {
                    eprintln!("Error updating folder selection: {}", e);
                    // Minimal revert: if the folder was the one optimistically added, remove it on error.
                    if original_optimistic_folder.as_ref() == Some(&path_counted) {
                        app_state.selected_items.remove(&path_counted);
                        for child in &original_optimistic_children {
                            app_state.selected_items.remove(child);
                        }
                    }
                    app_state.set_message(
                        format!("Error fully selecting folder {}: {}", path_counted.display(), e),
                        crate::tui::state::MessageType::Error
                    );
                }
            } // If it's a file, it was already added to selected_items if within limit, no further action here.
        } else {
            // Exceeded limit. Revert optimistic add if it was this path.
            if let Some(opt_path) = &original_optimistic_folder {
                // Check if the path that was counted matches the folder that was optimistically added.
                if path_counted == *opt_path { // Ensure we are reverting the correct optimistic add
                    app_state.selected_items.remove(opt_path);
                    for child_path in &original_optimistic_children {
                        app_state.selected_items.remove(child_path);
                    }
                }
            }

            write_selection_limit_debug_log(
                &app_state.selected_items,
                &Some(path_counted.clone()), // path_counted is the one that triggered this specific finalization
                num_items_in_op,
                count_of_other_selected_items,
                app_state.selection_limit
            );

            app_state.modal = Some(crate::tui::components::Modal::new(
                format!(
                    "Selection limit ({}) exceeded for {}. Tried to select {} items (total would be {}).",
                    app_state.selection_limit, path_counted.file_name().unwrap_or_default().to_string_lossy(), num_items_in_op, count_of_other_selected_items + num_items_in_op
                ),
                75, // Wider modal for more info
                6,
            ));
        }
        // Clear optimistic data related to the single item selection that just completed.
        // Check if the completed path matches the stored optimistically_added_folder before clearing.
        if app_state.optimistically_added_folder.as_ref() == Some(&path_counted) {
            app_state.optimistically_added_folder = None;
            app_state.optimistically_added_children.clear();
        }
    }

    /// Shows the help modal in the application.
    pub fn show_help(app_state: &mut AppState) -> io::Result<()> {
        app_state.modal = Some(crate::tui::components::Modal::help());
        Ok(())
    }

    /// Toggles expansion/collapse of a folder in the file list.
    ///
    /// # Arguments
    /// * `app_state` - Mutable reference to [`AppState`].
    /// * `selection_state` - Reference to [`SelectionState`].
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(io::Error)` if expansion fails.
    ///
    /// # Errors
    /// Returns an error if folder expansion/collapse fails.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::handlers::FileOpsHandler;
    /// FileOpsHandler::toggle_folder_expansion(&mut app_state, &selection_state).unwrap();
    /// ```
    #[doc(alias = "file-ops-expand")]
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
        let mut visited = std::collections::HashSet::new();
        Self::expand_all_inner(app_state, path, &mut visited)
    }

    // Inner helper that tracks visited canonical paths to prevent symlink loops
    fn expand_all_inner(app_state: &mut AppState, path: &PathBuf, visited: &mut std::collections::HashSet<PathBuf>) -> io::Result<()> {
        if !path.is_dir() || app_state.is_path_ignored(path) {
            return Ok(());
        }

        // Get canonical path to detect symlink loops
        let canonical_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
        if !visited.insert(canonical_path) {
            // Already visited this canonical path - avoid infinite recursion
            return Ok(());
        }

        // Memory optimization: Use to_path_buf() to avoid unnecessary clone
        app_state.expanded_folders.insert(path.to_path_buf());

        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.filter_map(|e| e.ok()) {
                    let child_path = entry.path();
                    if child_path.is_dir() && !app_state.is_path_ignored(&child_path) {
                        Self::expand_all_inner(app_state, &child_path, visited)?;
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
                        .is_none_or(|ep_str| !ep_str.starts_with(path_str))
            });
        } else {
            // Fallback if path is not valid UTF-8 (less likely but good to handle)
            app_state.expanded_folders.remove(path);
            // This fallback won't remove children, but it's better than erroring.
        }

        Ok(())
    }

    /// Toggles recursive expansion/collapse of a folder and its descendants.
    ///
    /// # Arguments
    /// * `app_state` - Mutable reference to [`AppState`].
    /// * `selection_state` - Mutable reference to [`SelectionState`].
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(io::Error)` if recursive expansion fails.
    ///
    /// # Errors
    /// Returns an error if recursive folder expansion/collapse fails.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::handlers::FileOpsHandler;
    /// FileOpsHandler::toggle_folder_expansion_recursive(&mut app_state, &mut selection_state).unwrap();
    /// ```
    #[doc(alias = "file-ops-expand-recursive")]
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

