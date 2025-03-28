use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::fs;

use crate::models::{app_config::Node, CopyStats, OutputFormat};
use crate::tui::state::AppState;
use crate::tui::state::{SearchState, SelectionState};
use crate::fs::is_path_ignored_for_iterative;

pub struct FileOpsHandler;

impl FileOpsHandler {
    pub fn load_items(app_state: &mut AppState) -> io::Result<()> {
        app_state.items.clear();

        if let Some(parent) = app_state.current_dir.parent() {
            if !parent.as_os_str().is_empty() {
                app_state.items.push(app_state.current_dir.join(".."));
            }
        }

        // Use the add_items_iterative function from fs module
        crate::fs::add_items_iterative(
            &mut app_state.items,
            &app_state.current_dir,
            &app_state.expanded_folders,
            &app_state.ignore_config,
            &app_state.current_dir,
        )?;

        // Update filtered items based on search
        app_state.filtered_items = app_state.items.clone();
        Ok(())
    }

    pub fn load_items_nonrecursive(app_state: &mut AppState) -> io::Result<()> {
        app_state.items.clear();
        app_state.filtered_items.clear();

        // Add parent directory entry if applicable
        if let Some(parent) = app_state.current_dir.parent() {
            if !parent.as_os_str().is_empty() {
                app_state.items.push(app_state.current_dir.join(".."));
            }
        }

        // Read only the current directory, no recursion
        let entries = fs::read_dir(&app_state.current_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| !is_path_ignored_for_iterative(p, &app_state.current_dir, &app_state.ignore_config))
            .collect::<Vec<_>>();

        // Sort entries (directories first, then files)
        let mut sorted_entries = entries;
        sorted_entries.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        app_state.items.extend(sorted_entries);
        app_state.filtered_items = app_state.items.clone();
        Ok(())
    }

    pub fn update_search(
        app_state: &mut AppState,
        search_state: &mut SearchState,
    ) -> io::Result<()> {
        if search_state.search_query.is_empty() {
            app_state.filtered_items = app_state.items.clone();
            return Ok(());
        }

        // Create a matcher function based on the search query
        let matcher = search_state.create_matcher();

        // If not in recursive mode, filter only the current items (non-recursive filtering)
        if !app_state.recursive {
            app_state.filtered_items = app_state
                .items
                .iter()
                .filter(|&p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map_or(false, |name| matcher(name))
                })
                .cloned()
                .collect();
            return Ok(());
        }

        // Otherwise, perform recursive search
        let max_depth = 4;
        let mut results = HashSet::new();

        // Recursively search each immediate child of the current directory
        if let Ok(entries) = fs::read_dir(&app_state.current_dir) {
            for entry in entries.filter_map(|e| e.ok()).map(|e| e.path()) {
                crate::fs::recursive_search_helper_generic(
                    app_state,
                    &entry,
                    1,
                    max_depth,
                    &matcher,
                    &mut results,
                );
            }
        }

        let mut matched: Vec<PathBuf> = results.into_iter().collect();
        matched.sort_by_key(|p| {
            p.strip_prefix(&app_state.current_dir)
                .map(|r| r.to_string_lossy().into_owned())
                .unwrap_or_default()
        });

        app_state.filtered_items = matched;

        // Ensure that the full hierarchy is visible by expanding parent folders
        let mut parents_to_expand = HashSet::new();
        for item in &app_state.filtered_items {
            let mut current = item.as_path();
            while let Some(parent) = current.parent() {
                if parent == app_state.current_dir {
                    break;
                }
                parents_to_expand.insert(parent.to_path_buf());
                current = parent;
            }
        }

        app_state.expanded_folders.extend(parents_to_expand);
        Ok(())
    }

    pub fn format_selected_items(app_state: &mut AppState) -> io::Result<String> {
        let result = crate::fs::format_selected_items(
            &app_state.selected_items,
            &app_state.current_dir,
            &app_state.ignore_config,
            app_state.output_format,
            app_state.show_line_numbers
        )?;

        // Update last_copy_stats with the result statistics
        app_state.last_copy_stats = Some(CopyStats {
            files: result.1,
            folders: result.2,
        });

        Ok(result.0)
    }

    pub fn handle_enter(
        app_state: &mut AppState,
        selection_state: &mut SelectionState,
    ) -> io::Result<()> {
        if let Some(selected) = selection_state.list_state.selected() {
            if selected >= app_state.filtered_items.len() {
                return Ok(());
            }

            let path = &app_state.filtered_items[selected];
            if path.is_dir() {
                if path.ends_with("..") {
                    if let Some(parent) = app_state.current_dir.parent() {
                        app_state.current_dir = parent.to_path_buf();
                    }
                } else {
                    app_state.current_dir = path.clone();
                }

                Self::load_items(app_state)?;
                selection_state.list_state.select(Some(0));
            }
        }
        Ok(())
    }

    pub fn toggle_default_ignores(app_state: &mut AppState) -> io::Result<()> {
        app_state.ignore_config.use_default_ignores = !app_state.ignore_config.use_default_ignores;
        Self::load_items(app_state)
    }

    pub fn toggle_gitignore(app_state: &mut AppState) -> io::Result<()> {
        app_state.ignore_config.use_gitignore = !app_state.ignore_config.use_gitignore;
        Self::load_items(app_state)
    }

    pub fn toggle_binary_files(app_state: &mut AppState) -> io::Result<()> {
        app_state.ignore_config.include_binary_files = !app_state.ignore_config.include_binary_files;
        Self::load_items(app_state)
    }

    pub fn toggle_output_format(app_state: &mut AppState) -> io::Result<()> {
        app_state.output_format = app_state.output_format.toggle();
        Ok(())
    }

    pub fn toggle_line_numbers(app_state: &mut AppState) -> io::Result<()> {
        // Don't toggle line numbers if we're in JSON mode
        if app_state.output_format != OutputFormat::Json {
            app_state.show_line_numbers = !app_state.show_line_numbers;
        }
        Ok(())
    }

    pub fn save_config(app_state: &mut AppState) -> io::Result<()> {
        crate::fs::save_config(app_state)
    }

    pub fn check_pending_selection(
        app_state: &mut AppState,
        selection_state: &mut SelectionState,
    ) -> io::Result<()> {
        // Check for pending selection count results
        if app_state.is_counting {
            if let Some(rx) = &app_state.pending_count {
                if let Ok(Ok(count)) = rx.try_recv() {
                    if count <= app_state.selection_limit {
                        if let Some(path) = app_state.counting_path.take() {
                            if path.is_dir() {
                                SelectionState::update_folder_selection(app_state, &path, true)?;
                            } else {
                                app_state.selected_items.insert(path);
                            }
                        }
                    } else {
                        app_state.modal = Some(crate::tui::components::Modal::new(
                            format!(
                                "Cannot select: would exceed limit of {} items\nTried to add {} items",
                                app_state.selection_limit, count
                            ),
                            50,
                            4,
                        ));
                    }
                    app_state.is_counting = false;
                    app_state.pending_count = None;
                }
            }
        }
        Ok(())
    }

    pub fn show_help(app_state: &mut AppState) -> io::Result<()> {
        app_state.modal = Some(crate::tui::components::Modal::help());
        Ok(())
    }
}
