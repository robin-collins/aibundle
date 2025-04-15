// src/output/json.rs
//!
//! # JSON Output Module
//!
//! This module provides functions for formatting selected files and directories as JSON output.
//! It is used to generate JSON-formatted code/documentation bundles for export or clipboard.
//!
//! ## Usage
//! Use `format_json_output` to convert a set of selected files and folders into a JSON string.
//!
//! ## Examples
//! ```rust
//! use crate::output::json::format_json_output;
//! let (json, stats) = format_json_output(&selected, &current_dir, &ignore_config).unwrap();
//! assert!(json.starts_with("["));
//! ```

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::fs::normalize_path;
use crate::models::CopyStats;
use crate::output::format::{is_binary_file, process_directory};

/// Formats the selected files and directories as JSON output.
///
/// Each file is represented as a JSON object with its path and content. Directories are represented as nested objects.
///
/// # Arguments
/// * `selected_items` - Set of selected file and directory paths.
/// * `current_dir` - The base directory for relative path calculation.
/// * `ignore_config` - Ignore configuration for filtering files.
///
/// # Returns
/// * `io::Result<(String, CopyStats)>` - The JSON output and copy statistics.
///
/// # Examples
/// ```rust
/// // Used internally by clipboard and file export logic.
/// ```
pub fn format_json_output(
    selected_items: &HashSet<PathBuf>,
    current_dir: &PathBuf,
    ignore_config: &crate::models::IgnoreConfig,
) -> io::Result<(String, CopyStats)> {
    let mut output = String::new();
    let mut stats = CopyStats {
        files: 0,
        folders: 0,
    };

    output.push('[');

    // Process only items whose parent is not also selected (avoid duplication)
    let mut to_process: Vec<_> = selected_items
        .iter()
        .filter(|path| {
            if let Some(parent) = path.parent() {
                !selected_items.contains(parent)
            } else {
                true
            }
        })
        .collect();
    to_process.sort();

    let mut first_item = true;
    for path in to_process {
        if let Ok(rel_path) = path.strip_prefix(current_dir) {
            let normalized_path = normalize_path(&rel_path.to_string_lossy());

            if !first_item {
                output.push(',');
            }
            first_item = false;

            if path.is_file() {
                if is_binary_file(path) {
                    if ignore_config.include_binary_files {
                        output.push_str(&format!(
                            "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":true}}",
                            normalized_path
                        ));
                        stats.files += 1;
                    }
                } else if let Ok(content) = fs::read_to_string(path) {
                    let escaped_content = content
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r");
                    output.push_str(&format!(
                        "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}",
                        normalized_path, escaped_content
                    ));
                    stats.files += 1;
                }
            } else if path.is_dir() {
                output.push_str(&format!(
                    "{{\"type\":\"directory\",\"path\":\"{}\",\"contents\":[",
                    normalized_path
                ));
                let mut dir_contents = String::new();
                if let Ok(dir_stats) = process_directory(
                    path,
                    &mut dir_contents,
                    current_dir,
                    selected_items,
                    &crate::models::OutputFormat::Json,
                    false, // JSON format doesn't use line numbers
                    ignore_config,
                ) {
                    output.push_str(&dir_contents);
                    stats.files += dir_stats.files;
                    stats.folders += dir_stats.folders;
                }
                output.push_str("]}");
                stats.folders += 1;
            }
        }
    }

    output.push(']');

    Ok((output, stats))
}

// TODO: Add option to pretty-print JSON output for readability.
// TODO: Add support for additional metadata fields if needed.
