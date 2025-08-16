// src/output/xml.rs
//!
//! # XML Output Module
//!
//! Provides functions for formatting selected files and directories as XML output for code/documentation bundles.
//!
//! ## Purpose
//! - Generate XML-formatted output for export or clipboard.
//! - Used by output and clipboard logic.
//!
//! ## Organization
//! - [`format_xml_output`]: Formats selected items as XML.
//!
//! ## Example
//! ```rust
//! use crate::output::xml::format_xml_output;
//! use std::collections::HashSet;
//! use std::path::PathBuf;
//! let (xml, stats) = format_xml_output(&HashSet::new(), &PathBuf::from("."), true, &crate::models::IgnoreConfig::default()).unwrap();
//! assert!(xml.contains("<file name="));
//! ```

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::fs::normalize_path;
use crate::models::CopyStats;
use crate::output::format::{format_file_content, is_binary_file, process_directory};

/// Escapes XML special characters in a string.
///
/// # Arguments
/// * `s` - The string to escape.
///
/// # Returns
/// * `String` - The escaped string safe for XML content.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

/// Formats the selected files and directories as XML output.
///
/// Each file is represented as a <file> element with its path as an attribute. Directories are represented as <folder> elements.
///
/// # Arguments
///
/// * `selected_items` - Set of selected file and directory paths.
/// * `current_dir` - The base directory for relative path calculation.
/// * `show_line_numbers` - Whether to include line numbers in file contents.
/// * `ignore_config` - Ignore configuration for filtering files.
///
/// # Returns
///
/// * `io::Result<(String, CopyStats)>` - The XML output and copy statistics.
///
/// # Errors
///
/// Returns an error if a file or directory cannot be read.
///
/// # Examples
/// ```rust
/// use crate::output::xml::format_xml_output;
/// use std::collections::HashSet;
/// use std::path::PathBuf;
/// let (xml, stats) = format_xml_output(&HashSet::new(), &PathBuf::from("."), true, &crate::models::IgnoreConfig::default()).unwrap();
/// assert!(xml.contains("<file name="));
/// ```
pub fn format_xml_output(
    selected_items: &HashSet<PathBuf>,
    current_dir: &PathBuf,
    show_line_numbers: bool,
    ignore_config: &crate::models::IgnoreConfig,
) -> io::Result<(String, CopyStats)> {
    let mut output = String::new();
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

    for path in to_process {

        // Handle the case where paths might be relative and current_dir is "."
        let rel_path = if current_dir == Path::new(".") && path.is_relative() {
            // If current_dir is "." and path is relative, use the path as-is
            path.as_path()
        } else if let Ok(stripped) = path.strip_prefix(current_dir) {
            stripped
        } else {
            // If stripping fails, check if the path is just a filename
            // This handles the case where selected_items contains just filenames
            path.as_path()
        };

        let normalized_path = normalize_path(&rel_path.to_string_lossy());

        if path.is_file() {
            if is_binary_file(path) {
                if ignore_config.include_binary_files {
                    output.push_str(&format!("<file name=\"{}\">\n</file>\n", normalized_path));
                    stats.files += 1;
                }
            } else {
                output.push_str(&format!("<file name=\"{}\">\n", normalized_path));
                if let Ok(content) = fs::read_to_string(path) {
                    let formatted_content = format_file_content(&content, show_line_numbers);
                    output.push_str(&xml_escape(&formatted_content));
                }
                output.push_str("</file>\n");
                stats.files += 1;
            }
        } else if path.is_dir() {
            output.push_str(&format!("<folder name=\"{}\">\n", normalized_path));
            let mut dir_contents = String::new();
            if let Ok(dir_stats) = process_directory(
                path,
                &mut dir_contents,
                current_dir,
                selected_items,
                &crate::models::OutputFormat::Xml,
                show_line_numbers,
                ignore_config,
            ) {
                stats.files += dir_stats.files;
                stats.folders += dir_stats.folders;
            }
            output.push_str(&dir_contents);
            output.push_str("</folder>\n");
        }
    }

    Ok((output, stats))
}

