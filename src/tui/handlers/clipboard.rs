// src/tui/handlers/clipboard.rs
//!
//! # Clipboard Handler
//!
//! Clipboard management for the TUI, supporting copy-to-clipboard operations, formatting, and statistics for selected files and folders. This module enables exporting selections in multiple formats (XML, Markdown, JSON, LLM) and integrates with the application state for seamless user workflows.
//!
//! ## Organization
//! - `ClipboardHandler`: Main handler struct for clipboard operations.
//! - Internal helpers for directory and file processing.
//!
//! ## Usage
//! Use [`ClipboardHandler`] to copy selected files/folders to the clipboard in the desired format.
//!
//! # Examples
//! ```rust
//! use crate::tui::handlers::ClipboardHandler;
//! let (output, stats) = ClipboardHandler::format_selected_items(&app_state).unwrap();
//! println!("{} ({} files, {} folders)", output, stats.files, stats.folders);
//! ```

use std::collections::HashSet;
use std::io;
use std::path::PathBuf;

use crate::models::{CopyStats, OutputFormat};
use crate::tui::state::AppState;

/// Handler for clipboard operations (copying selected items, formatting, stats).
///
/// # Purpose
/// Provides methods to format and process selected items for clipboard export, supporting multiple output formats and statistics.
///
/// # Examples
/// ```rust
/// use crate::tui::handlers::ClipboardHandler;
/// let (output, stats) = ClipboardHandler::format_selected_items(&app_state).unwrap();
/// assert!(stats.files >= 0);
/// ```
#[doc(alias = "clipboard")]
pub struct ClipboardHandler;

impl Default for ClipboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardHandler {
    /// Creates a new `ClipboardHandler` instance.
    ///
    /// # Returns
    ///
    /// * `ClipboardHandler` - A new handler instance.
    ///
    /// # Examples
    /// ```rust
    /// let handler = crate::tui::handlers::ClipboardHandler::new();
    /// ```
    #[doc(alias = "clipboard-new")]
    pub fn new() -> Self {
        Self
    }

    /// Formats the selected items for clipboard output and returns stats.
    ///
    /// # Arguments
    /// * `app_state` - Reference to the current [`AppState`] containing selection and config.
    ///
    /// # Returns
    /// * `Ok((String, CopyStats))` - The formatted output and statistics.
    /// * `Err(io::Error)` - If file or directory processing fails.
    ///
    /// # Errors
    /// Returns an error if reading files or directories fails.
    ///
    /// # Examples
    /// ```rust
    /// let (output, stats) = crate::tui::handlers::ClipboardHandler::format_selected_items(&app_state).unwrap();
    /// assert!(output.len() > 0);
    /// ```
    #[doc(alias = "clipboard-format")]
    pub fn format_selected_items(app_state: &AppState) -> io::Result<(String, CopyStats)> {
        let mut output = String::new();

        let selected_items: Vec<_> = app_state.selected_items.iter().cloned().collect();

        if selected_items.is_empty() {
            return Ok((
                "No items selected.".to_string(),
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
                // Use the comprehensive LLM format function
                let (llm_output, llm_stats) = crate::output::format_llm_output(
                    &app_state.selected_items,
                    &app_state.current_dir,
                    &app_state.ignore_config,
                )?;
                output.push_str(&llm_output);

                // Update total stats with the format-specific stats
                total_stats.files = llm_stats.files;
                total_stats.folders = llm_stats.folders;
            }
        }

        // Return both the output string and the stats
        Ok((output, total_stats))
    }

    /// Recursively processes a directory, collecting file contents and stats.
    ///
    /// # Arguments
    /// * `app_state` - Reference to the current [`AppState`].
    /// * `path` - Directory path to process.
    /// * `file_contents` - Accumulator for file contents.
    /// * `base_path` - Base directory for relative path calculation.
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(io::Error)` if reading fails.
    ///
    /// # Errors
    /// Returns an error if directory traversal or file reading fails.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // Used internally by ClipboardHandler
    /// ```
    fn process_directory(
        app_state: &AppState,
        path: &PathBuf,
        file_contents: &mut Vec<(String, String)>,
        base_path: &PathBuf,
    ) -> io::Result<()> {
        // Note: Don't skip explicitly selected directories based on ignore rules

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
            Err(_e) => {
                // eprintln!("Error reading directory {}: {}", path.display(), e);
            }
        }

        Ok(())
    }

    /// Processes a file, collecting its contents if not ignored or binary (unless allowed).
    ///
    /// # Arguments
    /// * `app_state` - Reference to the current [`AppState`].
    /// * `path` - File path to process.
    /// * `file_contents` - Accumulator for file contents.
    /// * `base_path` - Base directory for relative path calculation.
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(io::Error)` if reading fails.
    ///
    /// # Errors
    /// Returns an error if file reading fails.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // Used internally by ClipboardHandler
    /// ```
    fn process_file(
        app_state: &AppState,
        path: &PathBuf,
        file_contents: &mut Vec<(String, String)>,
        base_path: &PathBuf,
    ) -> io::Result<()> {
        // Note: Don't skip explicitly selected files based on ignore rules

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
            Err(_e) => {
                // eprintln!("Error reading file {}: {}", path.display(), e);
            }
        }

        Ok(())
    }
}

// TODO: Add support for progress reporting during large copy operations.
