// src/tui/state/search.rs
//!
//! # Search State Module
//!
//! Defines the search state and logic for filtering and selecting items in the TUI. Provides utilities for search query management, selection toggling, and matcher creation. Enables interactive search and selection workflows.
//!
//! ## Organization
//! - [`SearchState`]: State struct for search queries and selection.
//!
//! ## Usage
//! Use [`SearchState`] to manage search queries and selection state during interactive search.
//!
//! # Examples
//! ```rust
//! use crate::tui::state::search::SearchState;
//! let mut state = SearchState::new();
//! state.search_query = "main".to_string();
//! ```

use glob::Pattern;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Represents the state of the search UI, including query and selection.
///
/// # Fields
/// * `search_query` - The current search query string.
/// * `is_searching` - Whether search mode is active.
/// * `selected_items` - Set of selected items in the search results.
///
/// # Examples
/// ```rust
/// use crate::tui::state::search::SearchState;
/// let mut state = SearchState::new();
/// state.search_query = "foo".to_string();
/// ```
///
/// # Fields
/// * `search_query` - The current search query string.
/// * `is_searching` - Whether search mode is active.
/// * `selected_items` - Set of selected items in the search results.
///
/// # Examples
/// ```rust
/// use crate::tui::state::search::SearchState;
/// let mut state = SearchState::new();
/// state.search_query = "foo".to_string();
/// ```
#[derive(Default)]
pub struct SearchState {
    pub search_query: String,
    #[allow(dead_code)]
    pub is_searching: bool,
    #[allow(dead_code)]
    pub selected_items: HashSet<PathBuf>,
}

impl SearchState {
    /// Creates a new, empty `SearchState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggles search mode on or off.
    #[allow(dead_code)]
    pub fn toggle_search(&mut self) {
        self.is_searching = !self.is_searching;
    }

    /// Clears the search query and disables search mode.
    #[allow(dead_code)]
    pub fn clear_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
    }

    /// Handles a character input for search, updating the query.
    #[allow(dead_code)]
    pub fn handle_search_input(&mut self, c: char) {
        if !self.is_searching {
            return;
        }

        match c {
            '/' => {
                self.is_searching = false;
            }
            _ if !c.is_control() => {
                self.search_query.push(c);
            }
            _ => {}
        }
    }

    /// Handles backspace in the search query.
    #[allow(dead_code)]
    pub fn handle_backspace(&mut self) {
        if self.is_searching {
            self.search_query.pop();
        }
    }

    /// Creates a matcher function for the current search query.
    ///
    /// If the query contains wildcards, uses glob matching; otherwise, substring matching.
    pub fn create_matcher(&self) -> Box<dyn Fn(&str) -> bool> {
        if self.search_query.is_empty() {
            return Box::new(|_| true);
        }

        let query = self.search_query.to_lowercase();

        // If query contains wildcards, use glob pattern; otherwise, plain substring
        if query.contains('*') || query.contains('?') {
            match Pattern::new(&query) {
                Ok(pattern) => Box::new(move |name: &str| pattern.matches(&name.to_lowercase())),
                Err(_) => Box::new(move |name: &str| name.to_lowercase().contains(&query)),
            }
        } else {
            Box::new(move |name: &str| name.to_lowercase().contains(&query))
        }
    }

    /// Toggles selection of a single path in the search results.
    #[allow(dead_code)]
    pub fn toggle_selection(&mut self, path: PathBuf) {
        if self.selected_items.contains(&path) {
            self.selected_items.remove(&path);
        } else {
            self.selected_items.insert(path);
        }
    }

    /// Toggles selection of all visible items in the search results.
    #[allow(dead_code)]
    pub fn toggle_select_all(&mut self, visible_items: &[PathBuf]) {
        // If all visible items are selected, deselect them all
        // Otherwise, select all visible items
        let all_selected = visible_items
            .iter()
            .all(|item| self.selected_items.contains(item));

        if all_selected {
            // Deselect all visible items
            for item in visible_items {
                self.selected_items.remove(item);
            }
        } else {
            // Select all visible items
            for item in visible_items {
                // Memory optimization: Use to_path_buf() to avoid unnecessary clone
                self.selected_items.insert(item.to_path_buf());
            }
        }
    }

    /// Returns true if the given path is selected in the search results.
    #[allow(dead_code)]
    pub fn is_selected(&self, path: &Path) -> bool {
        self.selected_items.contains(path)
    }

    /// Returns the number of selected items in the search results.
    #[allow(dead_code)]
    pub fn selected_count(&self) -> usize {
        self.selected_items.len()
    }

    /// Clears all selections in the search results.
    #[allow(dead_code)]
    pub fn clear_selections(&mut self) {
        self.selected_items.clear();
    }

    /// Returns a reference to the set of selected items.
    #[allow(dead_code)]
    pub fn get_selected_items(&self) -> &HashSet<PathBuf> {
        &self.selected_items
    }
}

// TODO: Add support for regex-based search queries.
// TODO: Add fuzzy matching for improved search experience.
// TODO: Add search history or recent queries tracking.
