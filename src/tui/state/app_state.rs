//!
//! # Application State Module
//!
//! Defines the main application state (`AppState`) and related types for the TUI system. Manages file system state, configuration, UI state, selection, search, and modal/message handling. Provides a central data structure for all TUI operations.
//!
//! ## Organization
//! - [`AppState`]: Main application state struct.
//! - [`Trie`]: Trie data structure for efficient path storage.
//! - [`MessageType`], [`AppMessage`]: User feedback and modal dialog types.
//!
//! ## Usage
//! Use [`AppState`] to track and mutate the state of the TUI application, including file navigation, selection, and clipboard operations.
//!
//! # Examples
//! ```rust
//! use crate::tui::state::AppState;
//! let mut state = AppState::default();
//! state.load_items().unwrap();
//! ```

use crate::models::{
    app_config::Node, constants, AppConfig, CopyStats, IgnoreConfig, OutputFormat,
};
use crate::tui::components::modal::Modal;
use ignore::{gitignore::GitignoreBuilder, Match};
use ratatui::widgets::ListState;
use std::collections::{BTreeMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Instant};
use std::sync::mpsc::{Sender, channel};
use crate::tui::app::AppEvent;

// Trie node structure for efficient path storage
#[derive(Default, Debug, Clone)]
struct TrieNode {
    children: BTreeMap<String, TrieNode>,
    is_end: bool,
}

/// Trie data structure for storing file paths efficiently.
///
/// # Fields
/// * `root` - The root node of the trie.
///
/// # Examples
/// ```rust
/// use crate::tui::state::Trie;
/// let mut trie = Trie::new();
/// trie.insert(std::path::Path::new("foo/bar.txt"));
/// assert_eq!(trie.len(), 1);
/// ```
#[doc(alias = "trie")]
#[derive(Default)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    /// Creates a new empty Trie
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a path into the trie
    pub fn insert(&mut self, path: &Path) {
        let mut node = &mut self.root;
        for component in path.components() {
            if let std::path::Component::Normal(os_str) = component {
                if let Some(comp_str) = os_str.to_str() {
                    node = node.children.entry(comp_str.to_string()).or_default();
                }
            }
        }
        node.is_end = true;
    }

    /// Converts the trie into a vector of paths
    pub fn to_vec(&self) -> Vec<PathBuf> {
        let mut results = Vec::new();
        let mut current_path = PathBuf::new();
        Self::traverse(&self.root, &mut current_path, &mut results);
        results
    }

    /// Recursive traversal to collect paths
    fn traverse(node: &TrieNode, current_path: &mut PathBuf, results: &mut Vec<PathBuf>) {
        if node.is_end {
            results.push(current_path.clone());
        }
        for (component, child) in &node.children {
            current_path.push(component);
            Self::traverse(child, current_path, results);
            current_path.pop();
        }
    }

    /// Returns the number of items in the trie
    pub fn len(&self) -> usize {
        Self::count_nodes(&self.root)
    }

    /// Clears all items from the trie
    pub fn clear(&mut self) {
        self.root = TrieNode::default();
    }

    /// Adds multiple paths to the trie
    pub fn extend<I>(&mut self, paths: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        for path in paths {
            self.insert(&path);
        }
    }

    /// Recursive helper to count nodes
    fn count_nodes(node: &TrieNode) -> usize {
        let mut count = if node.is_end { 1 } else { 0 };
        for child in node.children.values() {
            count += Self::count_nodes(child);
        }
        count
    }
}

/// Message type for user feedback and modal dialogs.
#[derive(Clone, Debug)]
pub enum MessageType {
    #[allow(dead_code)]
    Info,
    Success,
    Warning,
    #[allow(dead_code)]
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
/// * `initial_dir` - The initial directory the application started in.
/// * `items` - Trie of all items in the current directory.
/// * `filtered_items` - List of filtered items for display.
/// * `selected_items` - Set of selected file/folder paths.
/// * `expanded_folders` - Set of expanded folder paths.
/// * `list_state` - List selection state for the file list.
/// * `search_query` - Current search query string.
/// * `is_searching` - Whether search mode is active.
/// * `output_format` - Current output format.
/// * `show_line_numbers` - Whether to show line numbers in output.
/// * `recursive` - Whether recursive traversal is enabled.
/// * `ignore_config` - Ignore configuration for file filtering.
/// * `selection_limit` - Maximum number of items that can be selected.
/// * `quit` - Whether the application should quit.
/// * `message` - Current message to display to the user.
/// * `modal` - Optional modal dialog.
/// * `last_copy_stats` - Statistics from the last copy operation.
/// * `is_counting` - Whether a counting operation is in progress.
/// * `counting_path` - Path being counted (for progress display).
/// * `file_tree` - File tree structure for LLM output.
/// * `optimistically_added_folder` - Folder optimistically added during selection.
/// * `optimistically_added_children` - Children optimistically added during folder selection.
/// * `count_abort_sender` - Sender for aborting a count operation.
/// * `tx` - Sender for AppEvents.
/// * `selection_is_over_limit` - Whether the selection is over the limit.
/// * `pending_save_config_path` - Pending save operation config path.
///
/// # Examples
/// ```rust
/// use crate::tui::state::AppState;
/// let mut state = AppState::default();
/// state.load_items().unwrap();
/// ```
#[doc(alias = "app-state")]
pub struct AppState {
    /// Current directory path.
    pub current_dir: PathBuf,
    /// The initial directory the application started in.
    pub initial_dir: PathBuf,

    /// Trie structure storing all items in the current directory
    pub items: Trie,

    /// Filtered items based on search query and ignore rules
    pub filtered_items: Vec<PathBuf>,

    /// Selected items for copy/export operations.
    pub selected_items: HashSet<PathBuf>,

    /// Folders that are expanded (for tree view).
    pub expanded_folders: HashSet<PathBuf>,

    /// List state for the file list widget.
    #[allow(dead_code)]
    pub list_state: ListState,

    /// Search query for filtering items.
    pub search_query: String,

    /// Whether the app is in search mode.
    pub is_searching: bool,

    /// Current output format (XML, Markdown, JSON, LLM).
    pub output_format: OutputFormat,

    /// Whether to show line numbers in output.
    pub show_line_numbers: bool,

    /// Whether to enable recursive directory traversal.
    pub recursive: bool,

    /// Ignore configuration for file filtering.
    pub ignore_config: IgnoreConfig,

    /// Selection limit for preventing performance issues.
    pub selection_limit: usize,

    /// Whether the app should quit.
    pub quit: bool,

    /// Current message to display to the user.
    pub message: Option<AppMessage>,

    /// Optional modal dialog.
    pub modal: Option<Modal>,

    /// Statistics from the last copy operation.
    pub last_copy_stats: Option<CopyStats>,

    /// Whether a counting operation is in progress.
    pub is_counting: bool,

    /// Path being counted (for progress display).
    pub counting_path: Option<PathBuf>,

    /// File tree structure for LLM output.
    #[allow(dead_code)]
    pub file_tree: Option<Node>,

    /// Stores the path of a folder optimistically added during selection pending count.
    pub optimistically_added_folder: Option<PathBuf>,

    /// Stores paths of children optimistically added during folder selection.
    pub optimistically_added_children: HashSet<PathBuf>,

    /// Sender for aborting a count operation.
    pub count_abort_sender: Option<Sender<()>>,

    /// Sender for AppEvents.
    pub tx: Sender<AppEvent>,

    /// Whether the selection is over the limit.
    pub selection_is_over_limit: bool,

    /// Pending save operation - stores config path when waiting for overwrite confirmation
    pub pending_save_config_path: Option<std::path::PathBuf>,
}

impl Default for AppState {
    /// Returns a default-initialized `AppState` using the current directory and default config.
    fn default() -> Self {
        let config = AppConfig::default();
        let ignore_config = IgnoreConfig::default();
        let initial_dir = std::env::current_dir().unwrap_or_default();
        let (tx, _rx) = channel::<crate::tui::app::AppEvent>();
        Self::new(config, initial_dir, ignore_config, tx)
            .expect("Failed to create default AppState")
    }
}

impl AppState {
    /// Creates a new `AppState` with the given config, directory, and ignore config.
    pub fn new(
        config: AppConfig,
        initial_dir_param: PathBuf,
        ignore_config_param: IgnoreConfig,
        event_sender: Sender<AppEvent>,
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
            current_dir: initial_dir_param.clone(),
            initial_dir: initial_dir_param,
            items: Trie::new(),
            filtered_items: Vec::new(),
            selected_items: HashSet::new(),
            expanded_folders: HashSet::new(),
            list_state: ListState::default(),
            search_query: String::new(),
            is_searching: false,
            output_format,
            show_line_numbers,
            recursive,
            ignore_config: ignore_config_param,
            selection_limit,
            quit: false,
            message: None,
            modal: None,
            last_copy_stats: None,
            is_counting: false,
            counting_path: None,
            file_tree: None,
            optimistically_added_folder: None,
            optimistically_added_children: HashSet::new(),
            count_abort_sender: None,
            tx: event_sender,
            selection_is_over_limit: false,
            pending_save_config_path: None,
        };
        Ok(slf)
    }

    /// Returns the number of selected items.
    #[allow(dead_code)]
    pub fn selected_count(&self) -> usize {
        self.selected_items.len()
    }

    /// Returns the number of items in the current directory.
    #[allow(dead_code)]
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the given file is selected.
    #[allow(dead_code)]
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

    /// Helper to gather items for the non-recursive view, respecting expanded_folders,
    /// and sorting items at each directory level.
    fn gather_items_for_view(
        &self,
        dir: &Path,
        collected_items: &mut Vec<PathBuf>,
        current_depth: usize,
        max_depth: usize,
    ) -> io::Result<()> {
        if current_depth > max_depth {
            return Ok(());
        }
        if !dir.is_dir() {
            return Ok(());
        }

        let mut current_level_entries: Vec<PathBuf> = Vec::new();
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry_result in entries {
                    match entry_result {
                        Ok(entry) => {
                            let path = entry.path();
                            if !self.is_path_ignored(&path) {
                                current_level_entries.push(path);
                            }
                        }
                        Err(_) => { /* Skip unreadable entries */ continue; }
                    }
                }
            }
            Err(e) => { return Err(e); }
        }

        // Sort the entries for the current directory level
        current_level_entries.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            let a_name_os = a.file_name().unwrap_or_default();
            let b_name_os = b.file_name().unwrap_or_default();
            let a_name_str = a_name_os.to_string_lossy();
            let b_name_str = b_name_os.to_string_lossy();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    let a_is_dot = a_name_str.starts_with(".");
                    let b_is_dot = b_name_str.starts_with(".");
                    match (a_is_dot, b_is_dot) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => {
                            let ordering = a_name_str.to_lowercase().cmp(&b_name_str.to_lowercase());
                            if ordering != std::cmp::Ordering::Equal {
                                ordering
                            } else {
                                a_name_str.cmp(&b_name_str)
                            }
                        }
                    }
                }
            }
        });

        // Add sorted entries to collected_items and recurse for expanded folders
        for path in current_level_entries {
            collected_items.push(path.clone());
            if path.is_dir() && self.expanded_folders.contains(&path) {
                self.gather_items_for_view(&path, collected_items, current_depth + 1, max_depth)?;
            }
        }
        Ok(())
    }

    /// Processes a raw list of items (search, sort if recursive) and updates filtered_items, including "..".
    fn process_for_display_and_set_filtered_items(&mut self, mut raw_items: Vec<PathBuf>) {
        // Apply search query if active
        if !self.search_query.is_empty() {
            raw_items.retain(|path| {
                path.to_string_lossy()
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
            });
        }

        // If in full recursive mode, apply a global sort.
        // Otherwise, items from gather_items_for_view are already hierarchically sorted.
        if self.recursive {
            raw_items.sort_by(|a, b| {
                let a_is_dir = a.is_dir();
                let b_is_dir = b.is_dir();
                let a_name_os = a.file_name().unwrap_or_default();
                let b_name_os = b.file_name().unwrap_or_default();
                let a_name_str = a_name_os.to_string_lossy();
                let b_name_str = b_name_os.to_string_lossy();
                match (a_is_dir, b_is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => {
                        let a_is_dot = a_name_str.starts_with(".");
                        let b_is_dot = b_name_str.starts_with(".");
                        match (a_is_dot, b_is_dot) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => {
                                let ordering = a_name_str.to_lowercase().cmp(&b_name_str.to_lowercase());
                                if ordering != std::cmp::Ordering::Equal {
                                    ordering
                                } else {
                                    a_name_str.cmp(&b_name_str)
                                }
                            }
                        }
                    }
                }
            });
        }

        self.filtered_items = raw_items;

        // Prepend ".." entry if not in initial_dir and a distinct parent exists
        if self.current_dir != self.initial_dir {
            if let Some(parent_dir_path) = self.current_dir.parent() {
                if parent_dir_path != self.current_dir {
                    let dot_dot_entry = self.current_dir.join("..");
                    self.filtered_items.insert(0, dot_dot_entry);
                }
            }
        }
    }

    /// Loads items from the current directory into the trie structure and updates filtered_items.
    pub fn load_items(&mut self) -> io::Result<()> {
        self.items.clear(); // Clear the Trie
        let mut collected_paths = Vec::new();

        if self.recursive {
            let all_files_recursive = crate::fs::list_files(&self.current_dir);
            for item in all_files_recursive {
                if !self.is_path_ignored(&item) {
                    collected_paths.push(item);
                }
            }
        } else {
            const MAX_EXPAND_DEPTH: usize = 10;
            self.gather_items_for_view(&self.current_dir, &mut collected_paths, 0, MAX_EXPAND_DEPTH)?;
        }

        self.items.extend(collected_paths.iter().cloned()); // Populate Trie for other uses
        self.process_for_display_and_set_filtered_items(collected_paths); // Process and set filtered_items

        Ok(())
    }

    /// Returns the display items (filtered items for UI display)
    pub fn get_display_items(&self) -> &[PathBuf] {
        &self.filtered_items
    }

    /// Sets a message to display to the user
    pub fn set_message(&mut self, content: String, message_type: MessageType) {
        self.message = Some(AppMessage::new(content, message_type));
    }
}
