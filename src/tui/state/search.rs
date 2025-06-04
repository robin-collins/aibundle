// src/tui/state/search.rs
//!
//! # Search State Module
//!
//! This module defines the search state and logic for filtering and selecting items in the TUI.
//! It provides utilities for search query management, selection toggling, and matcher creation.
//!
//! ## Usage
//! Use `SearchState` to manage search queries and selection state during interactive search.
//!
//! ## Examples
//! ```rust
//! use crate::tui::state::search::{SearchState, perform_search};
//! let mut state = SearchState::new();
//! state.search_query = "main".to_string();
//! let indices = perform_search(&items, &state.search_query);
//! ```

use glob::Pattern;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Represents the state of the search UI, including query and selection.
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

/// Performs a search on the given items, returning indices of matches.
///
/// # Arguments
/// * `items` - The list of items to search.
/// * `query` - The search query string.
///
/// # Returns
/// * `Vec<usize>` - Indices of items matching the query.
///
/// # Examples
/// ```rust
/// let indices = crate::tui::state::search::perform_search(&items, "main");
/// assert!(indices.len() <= items.len());
/// ```
#[allow(dead_code)]
pub fn perform_search(items: &[PathBuf], query: &str) -> Vec<usize> {
    if query.is_empty() {
        // If query is empty, return all indices
        return (0..items.len()).collect();
    }

    let lower_query = query.to_lowercase();
    let mut filtered_indices = Vec::new();

    for (index, item_path_buf) in items.iter().enumerate() {
        // Explicitly borrow as Path to use its methods and satisfy the linter
        let item_path: &Path = item_path_buf.as_path();

        // Check filename containment (matching monolithic logic)
        if let Some(filename) = item_path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                if filename_str.to_lowercase().contains(&lower_query) {
                    filtered_indices.push(index);
                }
            }
        }
    }

    filtered_indices
}

// TODO: Add support for regex-based search queries.
// TODO: Add fuzzy matching for improved search experience.
// TODO: Add search history or recent queries tracking.
