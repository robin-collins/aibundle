// src/tui/handlers/search.rs
//!
//! # Search Handler
//!
//! Provides search input and filtering logic for the TUI, including toggling search mode, updating queries, and clearing search state. This module enables interactive filtering of file lists in the TUI.
//!
//! ## Organization
//! - `SearchHandler`: Main handler struct for search input and filtering.
//!
//! ## Usage
//! Use [`SearchHandler`] to manage search input and update filtered file lists in the TUI.
//!
//! # Examples
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
///
/// # Purpose
/// Provides methods to manage search input, update filtered file lists, and control search mode in the TUI.
///
/// # Examples
/// ```rust
/// use crate::tui::handlers::SearchHandler;
/// SearchHandler::toggle_search(&mut app_state, &mut search_state, &mut selection_state).unwrap();
/// ```
#[doc(alias = "search")]
pub struct SearchHandler;

impl SearchHandler {
    /// Toggles search mode on/off, updating search results and selection.
    ///
    /// # Arguments
    /// * `app_state` - Mutable reference to [`AppState`].
    /// * `search_state` - Mutable reference to [`SearchState`].
    /// * `selection_state` - Mutable reference to [`SelectionState`].
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(io::Error)` if search update fails.
    ///
    /// # Errors
    /// Returns an error if updating search results fails.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::handlers::SearchHandler;
    /// SearchHandler::toggle_search(&mut app_state, &mut search_state, &mut selection_state).unwrap();
    /// ```
    #[doc(alias = "search-toggle")]
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
    ///
    /// # Arguments
    /// * `app_state` - Mutable reference to [`AppState`].
    /// * `search_state` - Mutable reference to [`SearchState`].
    /// * `selection_state` - Mutable reference to [`SelectionState`].
    /// * `input` - Optional character input (None for backspace).
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(io::Error)` if search update fails.
    ///
    /// # Errors
    /// Returns an error if updating search results fails.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::handlers::SearchHandler;
    /// SearchHandler::handle_search_input(&mut app_state, &mut search_state, &mut selection_state, Some('a')).unwrap();
    /// ```
    #[doc(alias = "search-input")]
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
    ///
    /// # Arguments
    /// * `app_state` - Mutable reference to [`AppState`].
    /// * `search_state` - Mutable reference to [`SearchState`].
    /// * `selection_state` - Mutable reference to [`SelectionState`].
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(io::Error)` if search update fails.
    ///
    /// # Errors
    /// Returns an error if updating search results fails.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::handlers::SearchHandler;
    /// SearchHandler::clear_search(&mut app_state, &mut search_state, &mut selection_state).unwrap();
    /// ```
    #[doc(alias = "search-clear")]
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
