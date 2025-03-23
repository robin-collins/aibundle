use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use std::sync::mpsc;
use std::io;
use std::thread;

use crate::fs;
use crate::models::IgnoreConfig;
use crate::tui::state::AppState;

/// Handles selection state operations
pub struct SelectionState {
    pub list_state: ratatui::widgets::ListState,
}

impl Default for SelectionState {
    fn default() -> Self {
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(0));
        Self { list_state }
    }
}

impl SelectionState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn move_selection(&mut self, delta: i32, item_count: usize) {
        if item_count == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_selected = (current as i32 + delta)
            .clamp(0, item_count as i32 - 1) as usize;
        self.list_state.select(Some(new_selected));
    }
    
    pub fn toggle_selection(&mut self, app_state: &mut AppState) -> io::Result<()> {
        if let Some(selected_index) = self.list_state.selected() {
            if selected_index >= app_state.filtered_items.len() {
                return Ok(());
            }
            
            let path = app_state.filtered_items[selected_index].clone();
            if path.file_name().map_or(false, |n| n == "..") {
                return Ok(());
            }
            
            let is_selected = app_state.selected_items.contains(&path);
            
            // If already selected, unselect immediately (no counting needed)
            if is_selected {
                if path.is_dir() {
                    Self::update_folder_selection(app_state, &path, false)?;
                } else {
                    app_state.selected_items.remove(&path);
                }
                return Ok(());
            }
            
            // If not selected, start an async count
            if !app_state.is_counting {
                let (tx, rx) = mpsc::channel();
                let base_path = app_state.current_dir.clone();
                let ignore_config = app_state.ignore_config.clone();
                let path_clone = path.clone();
                let selection_limit = app_state.selection_limit;
                
                thread::spawn(move || {
                    let result = fs::count_selection_items_async(
                        &path_clone,
                        &base_path,
                        &ignore_config,
                        selection_limit,
                    );
                    let _ = tx.send(result);
                });
                
                app_state.pending_count = Some(rx);
                app_state.counting_path = Some(path);
                app_state.is_counting = true;
            }
        }
        
        Ok(())
    }
    
    pub fn update_folder_selection(
        app_state: &mut AppState,
        path: &PathBuf,
        selected: bool
    ) -> io::Result<()> {
        if path.is_dir() {
            if selected {
                app_state.selected_items.insert(path.clone());
            } else {
                app_state.selected_items.remove(path);
            }
            
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let child_path = entry.path();
                    if child_path.is_dir() {
                        Self::update_folder_selection(app_state, &child_path, selected)?;
                    } else if selected {
                        app_state.selected_items.insert(child_path);
                    } else {
                        app_state.selected_items.remove(&child_path);
                    }
                }
            }
        } else if selected {
            app_state.selected_items.insert(path.clone());
        } else {
            app_state.selected_items.remove(path);
        }
        
        Ok(())
    }
    
    pub fn toggle_select_all(&mut self, app_state: &mut AppState) -> io::Result<()> {
        let all_selected = app_state.filtered_items
            .iter()
            .filter(|path| !path.ends_with(".."))
            .all(|path| app_state.selected_items.contains(path));
            
        if all_selected {
            app_state.selected_items.clear();
        } else {
            // Select all items in filtered_items, except ".."
            for path in &app_state.filtered_items {
                if path.file_name().map_or(false, |n| n == "..") {
                    continue;
                }
                
                app_state.selected_items.insert(path.clone());
                
                // If this is a directory, also select all its children
                // but only if we're in recursive mode or the folder is expanded
                if path.is_dir() && (app_state.recursive || app_state.expanded_folders.contains(path)) {
                    Self::update_folder_selection(app_state, path, true)?;
                }
            }
            
            // Count total selected items for the warning
            if !app_state.is_counting && !app_state.filtered_items.is_empty() {
                let (tx, rx) = mpsc::channel();
                let counting_path = app_state.current_dir.clone();
                let base_dir = app_state.current_dir.clone();
                let ignore_config = app_state.ignore_config.clone();
                let selection_limit = app_state.selection_limit;
                
                // Spawn a background thread to count items
                thread::spawn(move || {
                    let result = fs::count_selection_items_async(
                        &counting_path,
                        &base_dir,
                        &ignore_config,
                        selection_limit,
                    );
                    let _ = tx.send(result);
                });
                
                app_state.pending_count = Some(rx);
                app_state.counting_path = Some(counting_path);
                app_state.is_counting = true;
            }
        }
        
        Ok(())
    }
}