use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use crate::models::{CopyStats, Node, OutputFormat};
use crate::tui::state::AppState;

pub struct ClipboardHandler;

impl ClipboardHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn copy_selected_to_clipboard(app_state: &mut AppState) -> io::Result<()> {
        let output = Self::format_selected_items(app_state)?;

        let result = crate::clipboard::copy_to_clipboard(&output);

        // Get the counts for the status message
        let (file_count, folder_count) = Self::count_selected_items(app_state);
        let line_count = output.lines().count();
        let byte_size = output.len();

        // Set last copy stats for display in the UI
        app_state.last_copy_stats = Some(CopyStats {
            files: file_count,
            folders: folder_count,
        });

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

    pub fn format_selected_items(app_state: &AppState) -> io::Result<String> {
        let mut output = String::new();
        let selected_items: Vec<_> = app_state
            .selected_items
            .iter()
            .filter(|p| !app_state.is_path_ignored(p))
            .cloned()
            .collect();

        if selected_items.is_empty() {
            return Ok("No items selected or all items are ignored.".to_string());
        }

        let base_path = &app_state.current_dir;
        let mut file_contents = Vec::new();

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
                crate::output::format_xml_output(
                    &mut output,
                    &file_contents,
                    app_state.show_line_numbers,
                );
            }
            OutputFormat::Markdown => {
                crate::output::format_markdown_output(
                    &mut output,
                    &file_contents,
                    app_state.show_line_numbers,
                );
            }
            OutputFormat::Json => {
                crate::output::format_json_output(&mut output, &file_contents);
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
                crate::output::format_llm_output(
                    &mut output,
                    &file_contents,
                    base_path,
                    &root_node,
                    &dependencies,
                );
            }
        }

        Ok(output)
    }

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

        // Try to read the directory entries
        match std::fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.filter_map(Result::ok) {
                    let entry_path = entry.path();
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
        if !app_state.ignore_config.include_binary_files && crate::tui::App::is_binary_file(path) {
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

    fn add_to_tree(path_str: &str, root: &mut Node, base_dir: &Path) {
        let path = Path::new(path_str);
        let mut current = root;

        for component in path.components() {
            let name = component.as_os_str().to_string_lossy().to_string();
            if name.is_empty() {
                continue;
            }

            let is_dir = component != path.components().last().unwrap();

            if current.children.is_none() {
                current.children = Some(std::collections::HashMap::new());
            }

            let children = current.children.as_mut().unwrap();

            if !children.contains_key(&name) {
                children.insert(
                    name.clone(),
                    Node {
                        name,
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
