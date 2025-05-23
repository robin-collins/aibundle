// src/tui/handlers/clipboard.rs
//!
//! # Clipboard Handler
//!
//! This module defines the `ClipboardHandler` for managing copy-to-clipboard operations in the TUI.
//! It handles formatting, stats, and tree-building for selected items, and supports multiple output formats.
//!
//! ## Usage
//! Use `ClipboardHandler` to copy selected files/folders to the clipboard in the desired format.
//!
//! ## Examples
//! ```rust
//! use crate::tui::handlers::ClipboardHandler;
//! ClipboardHandler::copy_selected_to_clipboard(&mut app_state).unwrap();
//! ```

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use crate::models::{app_config::Node, CopyStats, OutputFormat};
use crate::tui::state::AppState;

/// Handler for clipboard operations (copying selected items, formatting, stats).
pub struct ClipboardHandler;

impl ClipboardHandler {
    /// Creates a new `ClipboardHandler` instance.
    pub fn new() -> Self {
        Self
    }

    /// Copies selected items to the clipboard, displaying a modal with stats.
    pub fn copy_selected_to_clipboard(app_state: &mut AppState) -> io::Result<()> {
        // Get formatted output and computed stats
        let (output, stats) = Self::format_selected_items(app_state)?;

        let result = crate::clipboard::copy_to_clipboard(&output);

        // Use the computed stats from format_selected_items
        let file_count = stats.files;
        let folder_count = stats.folders;
        let line_count = output.lines().count();
        let byte_size = output.len();

        // Set last copy stats for display in the UI
        app_state.last_copy_stats = Some(stats);

        // Display a modal with the copy stats
        app_state.modal = Some(crate::tui::components::Modal::copy_stats(
            file_count,
            folder_count,
            line_count,
            byte_size,
            &app_state.output_format,
        ));

        result
    }

    /// Counts the number of selected files and folders.
    fn count_selected_items(app_state: &AppState) -> (usize, usize) {
        let mut files = 0;
        let mut folders = 0;

        for path in &app_state.selected_items {
            if path.is_dir() {
                folders += 1;
            } else {
                files += 1;
            }
        }

        (files, folders)
    }

    /// Formats the selected items for clipboard output and returns stats.
    pub fn format_selected_items(app_state: &AppState) -> io::Result<(String, CopyStats)> {
        let mut output = String::new();
        let selected_items: Vec<_> = app_state
            .selected_items
            .iter()
            .filter(|p| !app_state.is_path_ignored(p))
            .cloned()
            .collect();

        if selected_items.is_empty() {
            return Ok((
                "No items selected or all items are ignored.".to_string(),
                CopyStats {
                    files: 0,
                    folders: 0,
                },
            ));
        }

        let base_path = &app_state.current_dir;
        let mut file_contents = Vec::new();
        // Initialize stats for tracking processed files and folders
        let mut total_stats = CopyStats {
            files: 0,
            folders: 0,
        };

        // Process all selected items
        for path in &selected_items {
            if path.is_dir() {
                Self::process_directory(app_state, path, &mut file_contents, base_path)?;
            } else {
                Self::process_file(app_state, path, &mut file_contents, base_path)?;
            }
        }

        // Sort file contents by path for consistent output
        file_contents.sort_by(|(a, _), (b, _)| a.cmp(b));

        // Format the output based on the selected format
        match app_state.output_format {
            OutputFormat::Xml => {
                let (xml_output, xml_stats) = crate::output::format_xml_output(
                    &app_state.selected_items,
                    &app_state.current_dir,
                    app_state.show_line_numbers,
                    &app_state.ignore_config,
                )?;
                output.push_str(&xml_output);
                // Update total stats with the format-specific stats
                total_stats.files += xml_stats.files;
                total_stats.folders += xml_stats.folders;
            }
            OutputFormat::Markdown => {
                let (md_output, md_stats) = crate::output::format_markdown_output(
                    &app_state.selected_items,
                    &app_state.current_dir,
                    app_state.show_line_numbers,
                    &app_state.ignore_config,
                )?;
                output.push_str(&md_output);
                // Update total stats with the format-specific stats
                total_stats.files += md_stats.files;
                total_stats.folders += md_stats.folders;
            }
            OutputFormat::Json => {
                // Call the function with the correct parameters and handle the returned tuple
                let (json_output, json_stats) = crate::output::format_json_output(
                    &app_state.selected_items,
                    &app_state.current_dir,
                    &app_state.ignore_config,
                )?;

                // Append the generated JSON string to our output
                output.push_str(&json_output);

                // Update total stats with the format-specific stats
                total_stats.files += json_stats.files;
                total_stats.folders += json_stats.folders;
            }
            OutputFormat::Llm => {
                // For LLM format, we need to build a tree structure
                let mut root_node = Node {
                    name: base_path.display().to_string(),
                    is_dir: true,
                    children: Some(std::collections::HashMap::new()),
                    parent: None,
                };

                // Build the tree structure
                for (path, _) in &file_contents {
                    Self::add_to_tree(path, &mut root_node, base_path);
                }

                // Analyze dependencies if we're doing LLM format
                let dependencies = crate::output::analyze_dependencies(&file_contents, base_path);

                // Format the output
                crate::output::format_llm_output_internal(
                    &mut output,
                    &file_contents,
                    base_path,
                    &root_node,
                    &dependencies,
                );

                // For LLM format, directly calculate stats from file_contents
                total_stats.files = file_contents.len();

                // Count unique folders (using HashSet to avoid duplicates)
                let mut folders = HashSet::new();
                for (path, _) in &file_contents {
                    let path_obj = Path::new(path);
                    let mut parent = path_obj.parent();
                    while let Some(dir) = parent {
                        if !dir.as_os_str().is_empty() {
                            folders.insert(dir.to_path_buf());
                        }
                        parent = dir.parent();
                    }
                }
                total_stats.folders = folders.len();
            }
        }

        // Return both the output string and the stats
        Ok((output, total_stats))
    }

    /// Recursively processes a directory, collecting file contents and stats.
    fn process_directory(
        app_state: &AppState,
        path: &PathBuf,
        file_contents: &mut Vec<(String, String)>,
        base_path: &PathBuf,
    ) -> io::Result<()> {
        // Skip if this directory should be ignored
        if app_state.is_path_ignored(path) {
            return Ok(());
        }

        // Use HashSet to track processed paths and avoid duplicates
        let mut processed_paths = HashSet::new();

        // Try to read the directory entries
        match std::fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.filter_map(Result::ok) {
                    let entry_path = entry.path();

                    // Skip if already processed (prevents duplicates)
                    if processed_paths.contains(&entry_path) {
                        continue;
                    }

                    processed_paths.insert(entry_path.clone());

                    if entry_path.is_dir() {
                        Self::process_directory(app_state, &entry_path, file_contents, base_path)?;
                    } else {
                        Self::process_file(app_state, &entry_path, file_contents, base_path)?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading directory {}: {}", path.display(), e);
            }
        }

        Ok(())
    }

    /// Processes a file, collecting its contents if not ignored or binary (unless allowed).
    fn process_file(
        app_state: &AppState,
        path: &PathBuf,
        file_contents: &mut Vec<(String, String)>,
        base_path: &PathBuf,
    ) -> io::Result<()> {
        // Skip if this file should be ignored
        if app_state.is_path_ignored(path) {
            return Ok(());
        }

        // Skip binary files unless explicitly included
        if !app_state.ignore_config.include_binary_files && crate::output::is_binary_file(path) {
            return Ok(());
        }

        // Try to read the file contents
        match std::fs::read_to_string(path) {
            Ok(content) => {
                if let Ok(relative_path) = path.strip_prefix(base_path) {
                    let path_str = relative_path.to_string_lossy().replace('\\', "/");
                    file_contents.push((path_str.to_string(), content));
                }
            }
            Err(e) => {
                eprintln!("Error reading file {}: {}", path.display(), e);
            }
        }

        Ok(())
    }

    /// Adds a file or directory path to the tree structure for LLM output.
    fn add_to_tree(path_str: &str, root: &mut Node, _base_dir: &Path) {
        let path = Path::new(path_str);
        let mut current = root;

        // Use HashSet to track processed components
        let mut processed_components = HashSet::new();

        for component in path.components() {
            let name = component.as_os_str().to_string_lossy().to_string();
            if name.is_empty() {
                continue;
            }

            // Skip if we've seen this component before (prevents duplicates in complex paths)
            if processed_components.contains(&name) {
                continue;
            }

            processed_components.insert(name.clone());

            let is_dir = component != path.components().last().unwrap();

            if current.children.is_none() {
                current.children = Some(std::collections::HashMap::new());
            }

            let children = current.children.as_mut().unwrap();

            if !children.contains_key(&name) {
                children.insert(
                    name.clone(),
                    Node {
                        name: name.clone(),
                        is_dir,
                        children: if is_dir {
                            Some(std::collections::HashMap::new())
                        } else {
                            None
                        },
                        parent: None,
                    },
                );
            }

            current = children.get_mut(&name).unwrap();
        }
    }
}

// TODO: Add support for progress reporting during large copy operations.
// TODO: Add support for filtering or transforming output before copying.
