use glob::Pattern;
use std::collections::HashSet;
use std::path::Path;

/// Manages search state and operations
pub struct SearchState {
    pub search_query: String,
    pub is_searching: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            is_searching: false,
        }
    }
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
}
