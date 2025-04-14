use glob::Pattern;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct SearchState {
    pub search_query: String,
    pub is_searching: bool,
    pub selected_items: HashSet<PathBuf>,
}

impl SearchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_search(&mut self) {
        self.is_searching = !self.is_searching;
    }

    pub fn clear_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
    }

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

    pub fn handle_backspace(&mut self) {
        if self.is_searching {
            self.search_query.pop();
        }
    }

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

    pub fn toggle_selection(&mut self, path: PathBuf) {
        if self.selected_items.contains(&path) {
            self.selected_items.remove(&path);
        } else {
            self.selected_items.insert(path);
        }
    }

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
                self.selected_items.insert(item.clone());
            }
        }
    }

    pub fn is_selected(&self, path: &Path) -> bool {
        self.selected_items.contains(path)
    }

    pub fn selected_count(&self) -> usize {
        self.selected_items.len()
    }

    pub fn clear_selections(&mut self) {
        self.selected_items.clear();
    }

    pub fn get_selected_items(&self) -> &HashSet<PathBuf> {
        &self.selected_items
    }
}

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

// Potential future functions related to search state management could go here.
