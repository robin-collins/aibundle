use crate::fs::add_items_iterative;
use crate::models::{app_config::Node, constants, AppConfig, CopyStats, IgnoreConfig, OutputFormat,};
use crate::output::format_selected_items;
use crate::tui::components::modal::Modal;
use crate::clipboard::copy_to_clipboard as copy_text_to_clipboard;
use ignore::{gitignore::GitignoreBuilder, Match};
use ratatui::widgets::ListState;
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Instant;

/// Enum to represent the type of message to display
#[derive(Clone, Debug)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

/// Struct to hold message details
#[derive(Clone, Debug)]
pub struct AppMessage {
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: Instant,
}

impl AppMessage {
    pub fn new(content: String, message_type: MessageType) -> Self {
        Self {
            content,
            message_type,
            timestamp: Instant::now(),
        }
    }
}

/// Core application state, separated from behavior
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
    fn default() -> Self {
        let config = AppConfig::default();
        let ignore_config = IgnoreConfig::default();
        let initial_dir = std::env::current_dir().unwrap_or_default();
        Self::new(config, initial_dir, ignore_config).expect("Failed to create default AppState")
    }
}

impl AppState {
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

    pub fn selected_count(&self) -> usize {
        self.selected_items.len()
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn is_file_selected(&self, path: &PathBuf) -> bool {
        self.selected_items.contains(path)
    }

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
            if self.ignore_config.extra_ignore_patterns.contains(&name.to_string()) {
                return true;
            }
        }
        if self.ignore_config.use_gitignore {
            let mut builder = GitignoreBuilder::new(&self.current_dir);
            let mut dir = self.current_dir.clone();
            while let Some(parent) = dir.parent() {
                let gitignore = dir.join(".gitignore");
                if gitignore.exists() {
                    if builder.add(gitignore).is_some() {
                        break;
                    }
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

    pub fn update_search(&mut self) {
        if self.is_searching && !self.search_query.is_empty() {
            let filtered_indices = crate::tui::state::search::perform_search(
                &self.items,
                &self.search_query,
            );
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
        } else {
            if let Some(selected) = self.list_state.selected() {
                if selected >= filtered_len {
                    self.list_state.select(Some(0));
                }
            } else {
                self.list_state.select(Some(0));
            }
        }
    }

    pub fn get_display_items(&self) -> &Vec<PathBuf> {
        &self.filtered_items
    }

    pub fn load_items(&mut self) -> io::Result<()> {
        let mut loaded_items = Vec::new();
        add_items_iterative(
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
                self.list_state.select(if self.items.is_empty() { None } else { Some(0) });
            }
        } else if !self.items.is_empty() {
            self.list_state.select(Some(0));
        }
        self.update_search();
        Ok(())
    }

    pub fn copy_selected_to_clipboard(&mut self) -> io::Result<()> {
        let (formatted_string, stats) = format_selected_items(
            &self.selected_items,
            &self.current_dir,
            &self.output_format,
            self.show_line_numbers,
            &self.ignore_config,
        )?;

        copy_text_to_clipboard(&formatted_string)?;

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

        Ok(())
    }

    pub fn move_selection(&mut self, delta: i32) {
        let items_len = self.filtered_items.len();
        if items_len == 0 {
            self.list_state.select(None);
            return;
        }
        let current_selection = self.list_state.selected().unwrap_or(0);
        let new_selected = (current_selection as i32 + delta).clamp(0, items_len as i32 - 1) as usize;
        self.list_state.select(Some(new_selected));
    }

    pub fn select_first(&mut self) {
        self.list_state.select(if self.filtered_items.is_empty() { None } else { Some(0) });
    }

    pub fn select_last(&mut self) {
        let len = self.filtered_items.len();
        self.list_state.select(if len == 0 { None } else { Some(len - 1) });
    }

    /// Handles character input when in search mode.
    pub fn handle_search_input(&mut self, c: char) {
        if self.is_searching {
            self.search_query.push(c);
            self.update_search();
        }
    }

    /// Handles backspace key when in search mode.
    pub fn handle_search_backspace(&mut self) {
        if self.is_searching {
            self.search_query.pop();
            self.update_search();
        }
    }

    /// Toggles the search mode on/off.
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

    /// Toggles the selection state of the currently highlighted item.
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
                        // Optionally show a modal indicating the limit is reached
                        let message = format!("Selection Limit Reached\n\nSelection limit ({}) reached.", self.selection_limit);
                        self.modal = Some(Modal::new(
                            message,
                            50, // Added default width
                            5 // Added default height
                        ));
                    }
                }
            }
        }
    }

    /// Toggles the selection state for all currently visible items (filtered or full list).
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
                    let message = format!(
                        "Selection Limit Reached\n\nSelection limit ({}) reached during select all.",
                        self.selection_limit
                    );
                    self.modal = Some(Modal::new(
                        message,
                        50, // Added default width
                        5 // Added default height
                    ));
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

    /// Sets or updates the message to be displayed.
    pub fn set_message(&mut self, content: String, message_type: MessageType) {
        self.message = Some(AppMessage::new(content, message_type));
    }

    /// Clears the current message.
    pub fn clear_message(&mut self) {
        self.message = None;
    }

    // TODO: Implement `update_folder_selection` based on monolithic version
    // fn update_folder_selection(&mut self, path: &PathBuf, selected: bool) { ... }
}
