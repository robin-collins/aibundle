// src/output/markdown.rs
//!
//! # Markdown Output Module
//!
//! This module provides functions for formatting selected files and directories as Markdown output.
//! It is used to generate Markdown-formatted code/documentation bundles for export or clipboard.
//!
//! ## Usage
//! Use `format_markdown_output` to convert a set of selected files and folders into a Markdown string.
//!
//! ## Examples
//! ```rust
//! use crate::output::markdown::format_markdown_output;
//! let (md, stats) = format_markdown_output(&selected, &current_dir, true, &ignore_config).unwrap();
//! assert!(md.contains("```"));
//! ```

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::fs::normalize_path;
use crate::models::CopyStats;
use crate::output::format::{format_file_content, is_binary_file, process_directory};

/// Formats the selected files and directories as Markdown output.
///
/// Each file is wrapped in a code block with its relative path as the language tag.
/// Directories are rendered as Markdown headers, and their contents are recursively included.
///
/// # Arguments
/// * `selected_items` - Set of selected file and directory paths.
/// * `current_dir` - The base directory for relative path calculation.
/// * `show_line_numbers` - Whether to include line numbers in code blocks.
/// * `ignore_config` - Ignore configuration for filtering files.
///
/// # Returns
/// * `io::Result<(String, CopyStats)>` - The Markdown output and copy statistics.
///
/// # Examples
/// ```rust
/// // Used internally by clipboard and file export logic.
/// ```
pub fn format_markdown_output(
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
                    output.push_str(&format!("```{}\n<binary file>\n```\n\n", normalized_path));
                    stats.files += 1;
                }
            } else {
                output.push_str(&format!("```{}\n", normalized_path));
                if let Ok(content) = fs::read_to_string(path) {
                    output.push_str(&format_file_content(&content, show_line_numbers));
                }
                output.push_str("```\n\n");
                stats.files += 1;
            }
        } else if path.is_dir() {
            output.push_str(&format!("## {}/\n\n", normalized_path));
            let mut dir_contents = String::new();
            if let Ok(dir_stats) = process_directory(
                path,
                &mut dir_contents,
                current_dir,
                selected_items,
                &crate::models::OutputFormat::Markdown,
                show_line_numbers,
                ignore_config,
            ) {
                stats.files += dir_stats.files;
                stats.folders += dir_stats.folders;
            }
            output.push_str(&dir_contents);
            stats.folders += 1;
        }
    }

    Ok((output, stats))
}

