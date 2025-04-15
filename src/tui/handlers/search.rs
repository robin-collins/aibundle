// src/tui/handlers/search.rs
//!
//! # Search Handler
//!
//! This module defines the `SearchHandler` for managing search input and filtering in the TUI.
//! It handles toggling search mode, updating search queries, and clearing search state.
//!
//! ## Usage
//! Use `SearchHandler` to manage search input and update filtered file lists in the TUI.
//!
//! ## Examples
//! ```rust
//! use crate::tui::handlers::SearchHandler;
//! SearchHandler::toggle_search(&mut app_state, &mut search_state, &mut selection_state).unwrap();
//! SearchHandler::handle_search_input(&mut app_state, &mut search_state, &mut selection_state, Some('a')).unwrap();
//! SearchHandler::clear_search(&mut app_state, &mut search_state, &mut selection_state).unwrap();
//! ```

use crate::tui::handlers::FileOpsHandler;
use crate::tui::state::{AppState, SearchState, SelectionState};
use std::io;

/// Handler for search input and filtering in the TUI.
pub struct SearchHandler;

impl SearchHandler {
    /// Toggles search mode on/off, updating search results and selection.
    pub fn toggle_search(
        app_state: &mut AppState,
        search_state: &mut SearchState,
        selection_state: &mut SelectionState,
    ) -> io::Result<()> {
        if app_state.is_searching {
            // We are currently searching, so exit
            // Update search results based on the final query *before* changing mode
            FileOpsHandler::update_search(app_state, search_state)?;
            // Reset selection to the top of the filtered list
            selection_state.list_state.select(Some(0));
            app_state.is_searching = false; // Change mode *after* updates
        } else {
            // We are not searching, so enter
            app_state.is_searching = true; // Change mode
                                           // No update needed here, input will trigger updates
        }
        Ok(())
    }

    /// Handles search input (character or backspace), updating the filtered list and selection.
    pub fn handle_search_input(
        app_state: &mut AppState,
        search_state: &mut SearchState,
        selection_state: &mut SelectionState,
        input: Option<char>,
    ) -> io::Result<()> {
        match input {
            Some(c) => {
                search_state.search_query.push(c);
            }
            None => {
                // Represents Backspace
                search_state.search_query.pop();
            }
        }
        // Update the filtered list based on the new query
        FileOpsHandler::update_search(app_state, search_state)?;
        // Reset selection to top after each input change during search
        selection_state.list_state.select(Some(0));
        Ok(())
    }

    /// Clears the search query and exits search mode, restoring the full list.
    pub fn clear_search(
        app_state: &mut AppState,
        search_state: &mut SearchState,
        selection_state: &mut SelectionState,
    ) -> io::Result<()> {
        search_state.search_query.clear();
        app_state.is_searching = false; // Ensure search mode is off
                                        // Update search results (will show full list as query is empty)
        FileOpsHandler::update_search(app_state, search_state)?;
        // Reset selection to the top
        selection_state.list_state.select(Some(0));
        Ok(())
    }
}

// TODO: Add support for regex or fuzzy search modes.
// TODO: Add search history or suggestions.
