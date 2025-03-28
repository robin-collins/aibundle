use crate::tui::state::{AppState, SearchState};
use crate::tui::handlers::FileOpsHandler; // Need FileOpsHandler for update_search
use std::io;

pub struct SearchHandler;

impl SearchHandler {
    /// Toggles the search mode on/off.
    pub fn toggle_search(app_state: &mut AppState, search_state: &mut SearchState) {
        app_state.is_searching = !app_state.is_searching;
        if !app_state.is_searching {
            // Exiting search mode
            search_state.search_query.clear();
            // Reset filtered items to the full list when exiting search
            app_state.filtered_items = app_state.items.clone();
            // TODO: Consider if selection needs adjustment when exiting search
        } else {
            // Entering search mode - input will be handled by handle_search_input
            // Ensure filtered_items is initially empty or reflects current items before filtering?
            // update_search will handle the filtering based on the query as it's typed.
        }
    }

    /// Handles character input or backspace during search.
    pub fn handle_search_input(
        app_state: &mut AppState,
        search_state: &mut SearchState,
        input: Option<char>,
    ) -> io::Result<()> { // Return io::Result because update_search returns it
        match input {
            Some(c) => {
                search_state.search_query.push(c);
            }
            None => { // Represents Backspace
                search_state.search_query.pop();
            }
        }
        // Update the filtered list based on the new query
        FileOpsHandler::update_search(app_state, search_state)?;
        Ok(())
    }

     /// Clears the current search query and results.
     pub fn clear_search(app_state: &mut AppState, search_state: &mut SearchState) -> io::Result<()> {
        search_state.search_query.clear();
        app_state.is_searching = false; // Ensure search mode is off
        // Reload items or just reset filtered_items? Resetting is simpler.
        app_state.filtered_items = app_state.items.clone();
        // FileOpsHandler::load_items(app_state)?; // Alternative: reload everything
        Ok(())
     }
}
