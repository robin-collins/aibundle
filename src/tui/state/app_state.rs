//!
//! # Application State Module
//!
//! This module defines the main application state (`AppState`) and related types for the TUI system.
//! It manages file system state, configuration, UI state, selection, search, and modal/message handling.
//!
//! ## Usage
//! Use `AppState` to track and mutate the state of the TUI application, including file navigation, selection, and clipboard operations.
//!
//! ## Examples
//! ```rust
//! use crate::tui::state::AppState;
//! let mut state = AppState::default();
//! state.load_items().unwrap();
//! ```

use crate::clipboard::copy_to_clipboard as copy_text_to_clipboard;
use crate::fs::add_items_recursively;
use crate::models::{
    app_config::Node, constants, AppConfig, CopyStats, IgnoreConfig, OutputFormat,
};
use crate::output::format_selected_items;
use crate::tui::components::modal::Modal;
use ignore::{gitignore::GitignoreBuilder, Match};
use ratatui::widgets::ListState;
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Instant;

/// Message type for user feedback and modal dialogs.
#[derive(Clone, Debug)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

/// Represents a message to be shown to the user, with type and timestamp.
#[derive(Clone, Debug)]
pub struct AppMessage {
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: Instant,
}

impl AppMessage {
    /// Creates a new `AppMessage` with the current timestamp.
    pub fn new(content: String, message_type: MessageType) -> Self {
        Self {
            content,
            message_type,
            timestamp: Instant::now(),
        }
    }
}

/// Main application state for the TUI system.
///
/// Tracks file system, configuration, UI, selection, search, and modal/message state.
///
/// # Fields
/// * `current_dir` - The current working directory.
/// * `items` - List of all items in the current directory.
/// * `list_state` - List selection state for the file list.
/// * `selected_items` - Set of selected file/folder paths.
/// * `expanded_folders` - Set of expanded folder paths.
/// * `config` - Application configuration.
/// * `ignore_config` - Ignore configuration for file filtering.
/// * `output_format` - Current output format.
/// * `show_line_numbers` - Whether to show line numbers in output.
/// * `selection_limit` - Maximum number of items that can be selected.
/// * `recursive` - Whether recursive traversal is enabled.
/// * `quit` - Whether the application should quit.
/// * `is_counting` - Whether a selection count operation is in progress.
/// * `modal` - Current modal dialog, if any.
/// * `show_help` - Whether the help view is active.
/// * `show_message` - Whether the message view is active.
/// * `counting_path` - Path being counted for selection.
/// * `pending_count` - Channel for pending selection count results.
/// * `last_copy_stats` - Statistics from the last copy operation.
/// * `search_query` - Current search query string.
/// * `filtered_items` - List of items matching the search query.
/// * `is_searching` - Whether search mode is active.
/// * `message` - Current message to display.
/// * `file_tree` - File tree structure for LLM output.
pub struct AppState {
    // File system state
    pub current_dir: PathBuf,
    pub items: Vec<PathBuf>,
    pub list_state: ListState,
    pub selected_items: HashSet<PathBuf>,
    pub expanded_folders: HashSet<PathBuf>,

    // Configuration
    pub config: AppConfig,
    pub ignore_config: IgnoreConfig,
    pub output_format: OutputFormat,
    pub show_line_numbers: bool,
    pub selection_limit: usize,
    pub recursive: bool,

    // UI state
    pub quit: bool,
    pub is_counting: bool,

    // Modal state
    pub modal: Option<Modal>,
    pub show_help: bool,
    pub show_message: bool,

    // Selection state
    pub counting_path: Option<PathBuf>,
    pub pending_count: Option<mpsc::Receiver<io::Result<usize>>>,

    // Operation results
    pub last_copy_stats: Option<CopyStats>,

    // Search state
    pub search_query: String,
    pub filtered_items: Vec<PathBuf>,
    pub is_searching: bool,

    // New fields
    pub message: Option<AppMessage>,
    pub file_tree: Option<Node>,
}

impl Default for AppState {
    /// Returns a default-initialized `AppState` using the current directory and default config.
    fn default() -> Self {
        let config = AppConfig::default();
        let ignore_config = IgnoreConfig::default();
        let initial_dir = std::env::current_dir().unwrap_or_default();
        Self::new(config, initial_dir, ignore_config).expect("Failed to create default AppState")
    }
}

impl AppState {
    /// Creates a new `AppState` with the given config, directory, and ignore config.
    pub fn new(
        config: AppConfig,
        initial_dir: PathBuf,
        ignore_config: IgnoreConfig,
    ) -> Result<Self, io::Error> {
        let selection_limit = config
            .selection_limit
            .unwrap_or(constants::DEFAULT_SELECTION_LIMIT);
        let recursive = config.default_recursive.unwrap_or(false);
        let output_format = config
            .default_format
            .as_deref()
            .map(|s| s.parse().unwrap_or_default())
            .unwrap_or_default();
        let show_line_numbers = config.default_line_numbers.unwrap_or(false);

        let slf = Self {
            current_dir: initial_dir,
            items: Vec::new(),
            list_state: ListState::default(),
            selected_items: HashSet::new(),
            expanded_folders: HashSet::new(),
            modal: None,
            show_help: false,
            show_message: false,
            config,
            ignore_config,
            output_format,
            show_line_numbers,
            selection_limit,
            recursive,
            quit: false,
            is_counting: false,
            counting_path: None,
            pending_count: None,
            last_copy_stats: None,
            search_query: String::new(),
            filtered_items: Vec::new(),
            is_searching: false,
            message: None,
            file_tree: None,
        };
        Ok(slf)
    }

    /// Returns the number of selected items.
    pub fn selected_count(&self) -> usize {
        self.selected_items.len()
    }

    /// Returns the number of items in the current directory.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the given file is selected.
    pub fn is_file_selected(&self, path: &PathBuf) -> bool {
        self.selected_items.contains(path)
    }

    /// Returns true if the given path should be ignored based on ignore config and .gitignore.
    pub fn is_path_ignored(&self, path: &Path) -> bool {
        if !self.ignore_config.use_default_ignores && !self.ignore_config.use_gitignore {
            return false;
        }
        if self.ignore_config.use_default_ignores {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if constants::DEFAULT_IGNORED_DIRS.contains(&name) {
                    return true;
                }
            }
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if self
                .ignore_config
                .extra_ignore_patterns
                .contains(&name.to_string())
            {
                return true;
            }
        }
        if self.ignore_config.use_gitignore {
            let mut builder = GitignoreBuilder::new(&self.current_dir);
            let mut dir = self.current_dir.clone();
            while let Some(parent) = dir.parent() {
                let gitignore = dir.join(".gitignore");
                if gitignore.exists() && builder.add(gitignore).is_some() {
                    break;
                }
                if dir.parent() == Some(&dir) {
                    break;
                }
                dir = parent.to_path_buf();
            }
            if let Ok(gitignore) = builder.build() {
                let is_dir = path.is_dir();
                if let Match::Ignore(_) = gitignore.matched_path_or_any_parents(path, is_dir) {
                    return true;
                }
            }
        }
        false
    }

    /// Updates the filtered items list based on the current search query.
    pub fn update_search(&mut self) {
        if self.is_searching && !self.search_query.is_empty() {
            let filtered_indices =
                crate::tui::state::search::perform_search(&self.items, &self.search_query);
            self.filtered_items = filtered_indices
                .iter()
                .filter_map(|&idx| self.items.get(idx).cloned())
                .collect();
        } else {
            self.filtered_items = self.items.clone();
        }
        let filtered_len = self.filtered_items.len();
        if filtered_len == 0 {
            self.list_state.select(None);
        } else if let Some(selected) = self.list_state.selected() {
            if selected >= filtered_len {
                self.list_state.select(Some(0));
            }
        } else {
            self.list_state.select(Some(0));
        }
    }

    /// Returns the list of items to display (filtered by search if active).
    pub fn get_display_items(&self) -> &Vec<PathBuf> {
        &self.filtered_items
    }

    /// Loads items from the current directory, applying ignore and expansion rules.
    pub fn load_items(&mut self) -> io::Result<()> {
        let mut loaded_items = Vec::new();
        add_items_recursively(
            &mut loaded_items,
            &self.current_dir,
            &self.expanded_folders,
            &self.ignore_config,
            &self.current_dir,
        )?;
        self.items = loaded_items;
        let current_selection = self.list_state.selected();
        if let Some(selected) = current_selection {
            if selected >= self.items.len() {
                self.list_state
                    .select(if self.items.is_empty() { None } else { Some(0) });
            }
        } else if !self.items.is_empty() {
            self.list_state.select(Some(0));
        }
        self.update_search();
        Ok(())
    }

    /// Copies the selected items to the clipboard, showing a modal with stats.
    pub fn copy_selected_to_clipboard(&mut self) -> io::Result<()> {
        let (formatted_string, stats) = format_selected_items(
            &self.selected_items,
            &self.current_dir,
            &self.output_format,
            self.show_line_numbers,
            &self.ignore_config,
        )?;

        match copy_text_to_clipboard(&formatted_string) {
            Ok(_) => {
                self.last_copy_stats = Some(stats.clone());
                let line_count = formatted_string.lines().count();
                let byte_size = formatted_string.len();
                self.modal = Some(Modal::copy_stats(
                    stats.files,
                    stats.folders,
                    line_count,
                    byte_size,
                    &self.output_format,
                ));
                self.set_message(
                    "Copied selected items to clipboard successfully.".to_string(),
                    MessageType::Info,
                );
                Ok(())
            }
            Err(e) => {
                self.set_message(
                    format!("Failed to copy to clipboard: {}", e),
                    MessageType::Error,
                );
                Err(e)
            }
        }
    }

    /// Moves the selection up or down by the given delta.
    pub fn move_selection(&mut self, delta: i32) {
        let items_len = self.filtered_items.len();
        if items_len == 0 {
            self.list_state.select(None);
            return;
        }
        let current_selection = self.list_state.selected().unwrap_or(0);
        let new_selected =
            (current_selection as i32 + delta).clamp(0, items_len as i32 - 1) as usize;
        self.list_state.select(Some(new_selected));
    }

    /// Selects the first item in the list.
    pub fn select_first(&mut self) {
        self.list_state.select(if self.filtered_items.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    /// Selects the last item in the list.
    pub fn select_last(&mut self) {
        let len = self.filtered_items.len();
        self.list_state
            .select(if len == 0 { None } else { Some(len - 1) });
    }

    /// Handles a character input for search, updating the search query.
    pub fn handle_search_input(&mut self, c: char) {
        if self.is_searching {
            self.search_query.push(c);
            self.update_search();
        }
    }

    /// Handles backspace in the search query.
    pub fn handle_search_backspace(&mut self) {
        if self.is_searching {
            self.search_query.pop();
            self.update_search();
        }
    }

    /// Toggles search mode on or off, clearing the query if turning off.
    pub fn toggle_search(&mut self) {
        self.is_searching = !self.is_searching;
        if !self.is_searching {
            self.search_query.clear();
        }
        self.update_search();
    }

    /// Clears the current search query.
    pub fn clear_search(&mut self) {
        if !self.search_query.is_empty() {
            self.search_query.clear();
            if self.is_searching {
                self.update_search();
            }
        }
    }

    /// Toggles selection of the currently highlighted item.
    pub fn toggle_selection(&mut self) {
        if let Some(selected_display_index) = self.list_state.selected() {
            // Get the path directly from filtered_items using the display index
            if let Some(path_to_toggle) = self.filtered_items.get(selected_display_index).cloned() {
                // Use the HashSet for selection management
                if self.selected_items.contains(&path_to_toggle) {
                    self.selected_items.remove(&path_to_toggle);
                } else {
                    // Check selection limit before adding
                    if self.selected_items.len() < self.selection_limit {
                        self.selected_items.insert(path_to_toggle.clone());
                    } else {
                        // Show a warning message for selection limit
                        self.set_message(
                            format!(
                                "Selection Limit Reached\n\nSelection limit ({}) reached.",
                                self.selection_limit
                            ),
                            MessageType::Error,
                        );
                    }
                }
            }
        }
    }

    /// Toggles selection of all currently visible items, respecting the selection limit.
    pub fn toggle_select_all(&mut self) {
        let display_items_paths: Vec<PathBuf> = self
            .get_display_items()
            .iter()
            .map(|p| (*p).clone()) // Clone paths from the slice of references
            .collect();

        if display_items_paths.is_empty() {
            return;
        }

        // Check if all currently visible items are already selected
        let all_visible_selected = display_items_paths
            .iter()
            .all(|path| self.selected_items.contains(path));

        if all_visible_selected {
            // Deselect all visible items
            for path in display_items_paths {
                self.selected_items.remove(&path);
                // TODO: Implement recursive deselection if it's a folder
            }
        } else {
            // Select all visible items, respecting the limit
            let mut selection_count = self.selected_items.len();
            for path in display_items_paths {
                if selection_count >= self.selection_limit {
                    self.set_message(
                        format!(
                            "Selection Limit Reached\n\nSelection limit ({}) reached during select all.",
                            self.selection_limit
                        ),
                        MessageType::Error,
                    );
                    break; // Stop selecting once limit is hit
                }
                // Use HashSet insert's return value to check if it was newly inserted
                if self.selected_items.insert(path.clone()) {
                    selection_count += 1;
                    // TODO: Implement recursive selection if it's a folder
                }
            }
        }
    }

    /// Sets a typed message and shows the message view.
    pub fn set_message(&mut self, content: String, message_type: MessageType) {
        self.message = Some(AppMessage::new(content, message_type));
        self.show_message = true;
    }

    /// Clears the current message and hides the message view.
    pub fn clear_message(&mut self) {
        self.message = None;
        self.show_message = false;
    }

    // TODO: Implement `update_folder_selection` based on monolithic version
    // fn update_folder_selection(&mut self, path: &PathBuf, selected: bool) { ... }
}
