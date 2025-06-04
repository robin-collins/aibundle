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

use crossterm::event::{KeyCode, KeyEvent};
use std::io;

use crate::tui::handlers::{FileOpsHandler, SearchHandler};
use crate::tui::state::{AppState, SearchState, SelectionState};

/// Handler for keyboard input and keybindings in the TUI.
pub struct KeyboardHandler;

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

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
                app_state.counting_path = None;
                app_state.modal = None;

                // Send an abort signal if a count_abort_sender exists
                // This is crucial to stop any background counting threads (single or select all)
                if let Some(sender) = app_state.count_abort_sender.take() {
                    let _ = sender.send(()); // Result can be ignored, thread might have finished
                }
            }
            return Ok(());
        }

        // If a modal is open, handle modal navigation/close
        if let Some(modal) = &mut app_state.modal {
            match modal.modal_type {
                crate::tui::components::modal::ModalType::Confirmation => {
                    // Handle confirmation modal (Y/N responses)
                    match key_event.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // User confirmed - handle the pending operation
                            FileOpsHandler::handle_save_config_confirmation(app_state, true)?;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            // User cancelled or escaped
                            FileOpsHandler::handle_save_config_confirmation(app_state, false)?;
                        }
                        _ => {} // Ignore other keys for confirmation modals
                    }
                }
                crate::tui::components::modal::ModalType::Info => {
                    // Handle regular info modals (existing behavior)
                    match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => app_state.modal = None,
                        KeyCode::PageDown | KeyCode::Down | KeyCode::Char('j') => modal.next_page(10),
                        KeyCode::PageUp | KeyCode::Up | KeyCode::Char('k') => modal.prev_page(10),
                        _ => {}
                    }
                }
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
                // Note: 'q' and Ctrl+C clipboard operations are now handled in App::run()
                // to support async clipboard operations
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
                KeyCode::Backspace => {
                    if let Some(selected_index) = selection_state.list_state.selected() {
                        let items = app_state.get_display_items();
                        if !items.is_empty() && selected_index < items.len() {
                            let selected_path = &items[selected_index];
                            // Use is_some_and as suggested by clippy
                            if selected_path.file_name().is_some_and(|name| name == "..") {
                                if let Some(parent) = app_state.current_dir.parent() {
                                    if parent != app_state.current_dir { // Ensure we don't go "up" from root to root
                                        app_state.current_dir = parent.to_path_buf();
                                        FileOpsHandler::load_items(app_state)?;
                                        selection_state.list_state.select(Some(0));
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                }
                KeyCode::Char(' ') => {
                    SelectionState::toggle_selection(selection_state, app_state)?;
                    // Force UI refresh for immediate visual feedback
                    Ok(())
                }
                KeyCode::Char('a') => {
                    let _ = SelectionState::toggle_select_all(selection_state, app_state);
                    Ok(())
                }
                // Note: 'c' clipboard operation is now handled in App::run()
                // to support async clipboard operations
                KeyCode::Char('i') => FileOpsHandler::toggle_default_ignores(app_state),
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
                KeyCode::Char('s') | KeyCode::Char('S') => FileOpsHandler::save_config(app_state),
                KeyCode::F(1) | KeyCode::Char('?') | KeyCode::Char('h') => {
                    FileOpsHandler::show_help(app_state)
                }
                _ => Ok(()),
            }
        }
    }
}

