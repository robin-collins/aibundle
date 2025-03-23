use std::io;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::tui::state::{AppState, SelectionState, SearchState};
use crate::tui::handlers::{ClipboardHandler, FileOpsHandler};

pub struct KeyboardHandler;

impl KeyboardHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub fn handle_key(
        &self,
        key: KeyEvent,
        app_state: &mut AppState,
        selection_state: &mut SelectionState,
        search_state: &mut SearchState,
    ) -> io::Result<()> {
        // Only handle key press events
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        
        // If searching, handle search input
        if search_state.is_searching {
            match key.code {
                KeyCode::Esc => search_state.clear_search(),
                KeyCode::Backspace => {
                    search_state.handle_backspace();
                    FileOpsHandler::update_search(app_state, search_state)?;
                }
                KeyCode::Char(c) => {
                    search_state.handle_search_input(c);
                    FileOpsHandler::update_search(app_state, search_state)?;
                }
                _ => {}
            }
            return Ok(());
        }
        
        // Normal mode key handling
        match key.code {
            KeyCode::Char('q') => {
                if !app_state.selected_items.is_empty() {
                    ClipboardHandler::copy_selected_to_clipboard(app_state)?;
                }
                app_state.quit = true;
            }
            KeyCode::Char('*') => selection_state.toggle_select_all(app_state)?,
            KeyCode::Char(' ') => selection_state.toggle_selection(app_state)?,
            KeyCode::Char('c') => ClipboardHandler::copy_selected_to_clipboard(app_state)?,
            KeyCode::Char('i') => FileOpsHandler::toggle_default_ignores(app_state)?,
            KeyCode::Char('g') => FileOpsHandler::toggle_gitignore(app_state)?,
            KeyCode::Char('b') => FileOpsHandler::toggle_binary_files(app_state)?,
            KeyCode::Char('f') => FileOpsHandler::toggle_output_format(app_state),
            KeyCode::Char('n') => FileOpsHandler::toggle_line_numbers(app_state),
            KeyCode::Char('s') => FileOpsHandler::save_config(app_state)?,
            KeyCode::Char('/') => {
                search_state.toggle_search();
                if !search_state.is_searching {
                    FileOpsHandler::update_search(app_state, search_state)?;
                }
            }
            KeyCode::Tab => FileOpsHandler::toggle_folder_expansion(app_state, selection_state)?,
            KeyCode::Up => selection_state.move_selection(-1, app_state.filtered_items.len()),
            KeyCode::Down => selection_state.move_selection(1, app_state.filtered_items.len()),
            KeyCode::PageUp => selection_state.move_selection(-10, app_state.filtered_items.len()),
            KeyCode::PageDown => selection_state.move_selection(10, app_state.filtered_items.len()),
            KeyCode::Enter => FileOpsHandler::handle_enter(app_state, selection_state)?,
            KeyCode::Char('h') => FileOpsHandler::show_help(app_state),
            _ => {}
        }
        
        Ok(())
    }
}