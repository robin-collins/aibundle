// src/output/json.rs
//!
//! # JSON Output Module
//!
//! Provides functions for formatting selected files and directories as JSON output for code/documentation bundles.
//!
//! ## Purpose
//! - Generate JSON-formatted output for export or clipboard.
//! - Used by output and clipboard logic.
//!
//! ## Organization
//! - [`format_json_output`]: Formats selected items as JSON.
//!
//! ## Example
//! ```rust
//! use crate::output::json::format_json_output;
//! use std::collections::HashSet;
//! use std::path::PathBuf;
//! let (json, stats) = format_json_output(&HashSet::new(), &PathBuf::from("."), &crate::models::IgnoreConfig::default()).unwrap();
//! assert!(json.starts_with("["));
//! ```

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::fs::normalize_path;
use crate::models::CopyStats;
use crate::output::format::{is_binary_file, process_directory};

// Note: json_escape function removed since we now use serde_json::json! macro
// which handles all escaping automatically

/// Formats the selected files and directories as JSON output.
///
/// Each file is represented as a JSON object with its path and content. Directories are represented as nested objects.
///
/// # Arguments
///
/// * `selected_items` - Set of selected file and directory paths.
/// * `current_dir` - The base directory for relative path calculation.
/// * `ignore_config` - Ignore configuration for filtering files.
///
/// # Returns
///
/// * `io::Result<(String, CopyStats)>` - The JSON output and copy statistics.
///
/// # Errors
///
/// Returns an error if a file or directory cannot be read.
///
/// # Examples
/// ```rust
/// use crate::output::json::format_json_output;
/// use std::collections::HashSet;
/// use std::path::PathBuf;
/// let (json, stats) = format_json_output(&HashSet::new(), &PathBuf::from("."), &crate::models::IgnoreConfig::default()).unwrap();
/// assert!(json.starts_with("["));
/// ```
pub fn format_json_output(
    selected_items: &HashSet<PathBuf>,
    current_dir: &PathBuf,
    ignore_config: &crate::models::IgnoreConfig,
) -> io::Result<(String, CopyStats)> {
    let mut stats = CopyStats {
        files: 0,
        folders: 0,
    };

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

    let mut json_items = Vec::new();
    
    for path in to_process {
        // Handle the case where paths might be relative and current_dir is "."
        let rel_path = if current_dir == Path::new(".") && path.is_relative() {
            path.as_path()
        } else if let Ok(stripped) = path.strip_prefix(current_dir) {
            stripped
        } else {
            path.as_path()
        };

        let normalized_path = normalize_path(&rel_path.to_string_lossy());

        if path.is_file() {
            if is_binary_file(path) {
                if ignore_config.include_binary_files {
                    json_items.push(serde_json::json!({
                        "type": "file",
                        "path": normalized_path,
                        "binary": true
                    }));
                    stats.files += 1;
                }
            } else if let Ok(content) = fs::read_to_string(path) {
                json_items.push(serde_json::json!({
                    "type": "file",
                    "path": normalized_path,
                    "binary": false,
                    "content": content
                }));
                stats.files += 1;
            }
        } else if path.is_dir() {
            // For directories, we still need to use the existing process_directory approach
            // since it handles the recursive structure. We'll build the directory JSON manually
            // but use serde_json for the outer structure.
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
                // Parse the directory contents as JSON and embed it
                let contents_json: serde_json::Value = if dir_contents.is_empty() {
                    serde_json::json!([])
                } else {
                    serde_json::from_str(&format!("[{}]", dir_contents))
                        .unwrap_or_else(|_| serde_json::json!([]))
                };
                
                json_items.push(serde_json::json!({
                    "type": "directory",
                    "path": normalized_path,
                    "contents": contents_json
                }));
                stats.files += dir_stats.files;
                stats.folders += dir_stats.folders;
            }
            stats.folders += 1;
        }
    }

    let output = serde_json::to_string_pretty(&json_items)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("JSON serialization error: {}", e)))?;

    Ok((output, stats))
}

