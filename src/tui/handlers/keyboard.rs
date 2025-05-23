// src/tui/handlers/keyboard.rs
//!
//! # Keyboard Handler
//!
//! This module defines the `KeyboardHandler` for managing keyboard input and keybindings in the TUI.
//! It handles navigation, selection, search, clipboard, and command execution.
//!
//! ## Usage
//! Use `KeyboardHandler` to process key events and update application state accordingly.
//!
//! ## Examples
//! ```rust
//! use crate::tui::handlers::KeyboardHandler;
//! KeyboardHandler::handle_key(key_event, &mut app_state, &mut selection_state, &mut search_state).unwrap();
//! ```

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;

use crate::tui::handlers::{ClipboardHandler, FileOpsHandler, SearchHandler};
use crate::tui::state::{AppState, SearchState, SelectionState};

/// Handler for keyboard input and keybindings in the TUI.
pub struct KeyboardHandler;

impl KeyboardHandler {
    /// Creates a new `KeyboardHandler` instance.
    pub fn new() -> Self {
        Self
    }

    /// Handles a key event, updating application state and triggering actions.
    pub fn handle_key(
        key_event: KeyEvent,
        app_state: &mut AppState,
        selection_state: &mut SelectionState,
        search_state: &mut SearchState,
    ) -> io::Result<()> {
        // If counting items (for selection limit), allow cancel with Esc
        if app_state.is_counting {
            if key_event.code == KeyCode::Esc {
                app_state.is_counting = false;
                app_state.pending_count = None;
                app_state.counting_path = None;
                app_state.modal = None;
            }
            return Ok(());
        }

        // If a modal is open, handle modal navigation/close
        if let Some(modal) = &mut app_state.modal {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => app_state.modal = None,
                KeyCode::PageDown | KeyCode::Down | KeyCode::Char('j') => modal.next_page(10),
                KeyCode::PageUp | KeyCode::Up | KeyCode::Char('k') => modal.prev_page(10),
                _ => {}
            }
            return Ok(());
        }

        // If searching, handle search input and navigation
        if app_state.is_searching {
            match key_event.code {
                KeyCode::Esc => {
                    SearchHandler::clear_search(app_state, search_state, selection_state)
                }
                KeyCode::Enter => {
                    SearchHandler::toggle_search(app_state, search_state, selection_state)
                }
                KeyCode::Backspace => SearchHandler::handle_search_input(
                    app_state,
                    search_state,
                    selection_state,
                    None,
                ),
                KeyCode::Char(c) => {
                    if c == '/' {
                        SearchHandler::toggle_search(app_state, search_state, selection_state)
                    } else {
                        SearchHandler::handle_search_input(
                            app_state,
                            search_state,
                            selection_state,
                            Some(c),
                        )
                    }
                }
                _ => Ok(()),
            }
        } else {
            match key_event.code {
                KeyCode::Char('q') => {
                    ClipboardHandler::copy_selected_to_clipboard(app_state)?;
                    app_state.quit = true;
                    Ok(())
                }
                KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
                    ClipboardHandler::copy_selected_to_clipboard(app_state)?;
                    app_state.quit = true;
                    Ok(())
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    let total = app_state.get_display_items().len();
                    SelectionState::move_selection(selection_state, 1_i32, total);
                    Ok(())
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let total = app_state.get_display_items().len();
                    SelectionState::move_selection(selection_state, -1_i32, total);
                    Ok(())
                }
                KeyCode::PageDown => {
                    let total = app_state.get_display_items().len();
                    SelectionState::move_selection(selection_state, 10_i32, total);
                    Ok(())
                }
                KeyCode::PageUp => {
                    let total = app_state.get_display_items().len();
                    SelectionState::move_selection(selection_state, -10_i32, total);
                    Ok(())
                }
                KeyCode::Home => {
                    selection_state.list_state.select(Some(0));
                    Ok(())
                }
                KeyCode::End => {
                    let total = app_state.get_display_items().len();
                    selection_state
                        .list_state
                        .select(Some(total.saturating_sub(1)));
                    Ok(())
                }
                KeyCode::Enter => FileOpsHandler::handle_enter(app_state, selection_state),
                KeyCode::Char(' ') => SelectionState::toggle_selection(selection_state, app_state),
                KeyCode::Char('a') => {
                    SelectionState::toggle_select_all(selection_state, app_state);
                    Ok(())
                }
                KeyCode::Char('c') => ClipboardHandler::copy_selected_to_clipboard(app_state),
                KeyCode::Char('d') => FileOpsHandler::toggle_default_ignores(app_state),
                KeyCode::Char('g') => FileOpsHandler::toggle_gitignore(app_state),
                KeyCode::Char('b') => FileOpsHandler::toggle_binary_files(app_state),
                KeyCode::Char('f') => FileOpsHandler::toggle_output_format(app_state),
                KeyCode::Char('n') => FileOpsHandler::toggle_line_numbers(app_state),
                KeyCode::Char('r') => {
                    app_state.recursive = !app_state.recursive;
                    if app_state.recursive {
                        FileOpsHandler::load_items(app_state)?;
                    } else {
                        FileOpsHandler::load_items_nonrecursive(app_state)?;
                    }
                    selection_state.list_state.select(Some(0));
                    if !search_state.search_query.is_empty() {
                        FileOpsHandler::update_search(app_state, search_state)?;
                    }
                    Ok(())
                }
                KeyCode::Char('/') => {
                    SearchHandler::toggle_search(app_state, search_state, selection_state)
                }
                // Tab: Toggle expansion of the currently selected folder (single-level)
                KeyCode::Tab => FileOpsHandler::toggle_folder_expansion(app_state, selection_state),
                // Shift+Tab: Toggle expansion recursively (all subfolders)
                KeyCode::BackTab => {
                    FileOpsHandler::toggle_folder_expansion_recursive(app_state, selection_state)
                }
                KeyCode::Char('S') => FileOpsHandler::save_config(app_state),
                KeyCode::F(1) | KeyCode::Char('?') => FileOpsHandler::show_help(app_state),
                _ => Ok(()),
            }
        }
    }
}

// TODO: Add support for customizable keybindings.
// TODO: Add support for multi-key command sequences.
