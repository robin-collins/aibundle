// src/output/format.rs
//!
//! # Output Format Utilities
//!
//! Utilities for formatting files and directories as XML, Markdown, or JSON for output and clipboard operations.
//!
//! ## Purpose
//!
//! - Provide helpers for binary detection, line numbering, and recursive directory processing.
//! - Used by output modules to render file/directory selections in various formats.
//!
//! ## Organization
//! - [`is_binary_file`]: Detects binary files by extension or name.
//! - [`format_file_content`]: Formats file content with optional line numbers.
//! - [`process_directory`]: Recursively processes directories for output.
//!
//! ## Example
//! ```rust
//! use crate::output::format::{process_directory, format_file_content};
//! use std::collections::HashSet;
//! use std::path::PathBuf;
//! let mut output = String::new();
//! let stats = process_directory(&PathBuf::from("src"), &mut output, &PathBuf::from("."), &HashSet::new(), &crate::models::OutputFormat::Xml, true, &crate::models::IgnoreConfig::default()).unwrap();
//! println!("{}", output);
//! ```

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::fs::normalize_path;
use crate::models::{CopyStats, IgnoreConfig, OutputFormat};

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

/// Properly escapes a string for JSON content using serde_json.
///
/// # Arguments
/// * `s` - The string to escape.
///
/// # Returns
/// * `String` - The escaped string safe for JSON content (without surrounding quotes).
fn json_escape(s: &str) -> String {
    // Use serde_json to properly escape all control characters
    let escaped = serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string());
    // Remove the surrounding quotes that serde_json adds
    if escaped.len() >= 2 {
        escaped[1..escaped.len()-1].to_string()
    } else {
        String::new()
    }
}

/// Returns true if the given path is a binary file, based on extension or name.
///
/// # Arguments
///
/// * `path` - The path to check.
///
/// # Returns
///
/// * `bool` - True if the file is binary, false otherwise.
///
/// # Examples
/// ```rust
/// use std::path::Path;
/// assert!(!crate::output::format::is_binary_file(Path::new("main.rs")));
/// ```
pub fn is_binary_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let binary_extensions = [
            "idx", "pack", "rev", "index", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp",
            "ico", "svg", "mp3", "wav", "ogg", "flac", "m4a", "aac", "wma", "mp4", "avi", "mkv",
            "mov", "wmv", "flv", "webm", "zip", "rar", "7z", "tar", "gz", "iso", "exe", "dll",
            "so", "dylib", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "class", "pyc",
            "pyd", "pyo",
        ];
        if binary_extensions.contains(&ext.to_lowercase().as_str()) {
            return true;
        }
    }

    // Removed overly aggressive filename-based binary detection
    // Files like "index.js", "index.html" should not be treated as binary
    false
}

/// Formats file content with optional line numbers.
///
/// # Arguments
///
/// * `content` - The file content as a string.
/// * `show_line_numbers` - Whether to include line numbers.
///
/// # Returns
///
/// * `String` - The formatted content.
///
/// # Examples
/// ```rust
/// let formatted = crate::output::format::format_file_content("line1\nline2", true);
/// assert!(formatted.contains("1 | line1"));
/// ```
pub fn format_file_content(content: &str, show_line_numbers: bool) -> String {
    let mut output = String::new();

    if show_line_numbers {
        for (i, line) in content.lines().enumerate() {
            output.push_str(&format!("{:>6} | {}\n", i + 1, line));
        }
    } else {
        output.push_str(content);
        if !content.ends_with('\n') {
            output.push('\n');
        }
    }

    output
}

/// Recursively processes a directory and adds its contents to the output in the specified format.
///
/// # Arguments
///
/// * `path` - The directory to process.
/// * `output` - Mutable string to write output to.
/// * `base_path` - The base directory for relative paths.
/// * `selected_items` - Set of selected file and directory paths.
/// * `output_format` - The output format (XML, Markdown, JSON).
/// * `show_line_numbers` - Whether to include line numbers in file content.
/// * `ignore_config` - Ignore configuration.
///
/// # Returns
///
/// * `io::Result<CopyStats>` - The number of files and folders processed.
///
/// # Panics
///
/// This function will panic if the file system is in an inconsistent state (e.g., permissions change during traversal).
///
/// # Errors
///
/// Returns an error if a directory entry cannot be read.
///
/// # Examples
/// ```rust
/// use std::collections::HashSet;
/// use std::path::PathBuf;
/// let mut output = String::new();
/// let stats = crate::output::format::process_directory(&PathBuf::from("src"), &mut output, &PathBuf::from("."), &HashSet::new(), &crate::models::OutputFormat::Xml, true, &crate::models::IgnoreConfig::default()).unwrap();
/// assert!(stats.files >= 0);
/// ```
pub fn process_directory(
    path: &PathBuf,
    output: &mut String,
    base_path: &PathBuf,
    selected_items: &HashSet<PathBuf>,
    output_format: &OutputFormat,
    show_line_numbers: bool,
    ignore_config: &IgnoreConfig,
) -> io::Result<CopyStats> {
    let mut files = 0;
    let mut folders = 0;

    // Create a list of selected entries within this directory
    let entries = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| selected_items.contains(p))
        .collect::<Vec<_>>();

    let mut first_item = true;
    if *output_format == OutputFormat::Json {
        for entry in entries {
            if !first_item {
                output.push(',');
            }
            first_item = false;

            if let Ok(rel_path) = entry.strip_prefix(base_path) {
                let normalized_path = normalize_path(&rel_path.to_string_lossy());

                if entry.is_file() {
                    if is_binary_file(&entry) {
                        if ignore_config.include_binary_files {
                            output.push_str(&format!(
                                "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":true}}",
                                normalized_path
                            ));
                            files += 1;
                        }
                    } else if let Ok(content) = fs::read_to_string(&entry) {
                        let escaped_content = json_escape(&content);
                        output.push_str(&format!(
                            "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}",
                            normalized_path,
                            escaped_content
                        ));
                        files += 1;
                    }
                } else if entry.is_dir() {
                    output.push_str(&format!(
                        "{{\"type\":\"directory\",\"path\":\"{}\",\"contents\":[",
                        normalized_path
                    ));
                    let mut dir_contents = String::new();
                    let stats = process_directory(
                        &entry,
                        &mut dir_contents,
                        base_path,
                        selected_items,
                        output_format,
                        show_line_numbers,
                        ignore_config,
                    )?;
                    files += stats.files;
                    folders += stats.folders;
                    output.push_str(&dir_contents);
                    output.push_str("]}");
                }
            }
        }
    } else {
        // XML or Markdown format (LLM has its own handling)
        for entry in entries {
            if let Ok(rel_path) = entry.strip_prefix(base_path) {
                let normalized_path = normalize_path(&rel_path.to_string_lossy());

                if entry.is_file() {
                    if is_binary_file(&entry) {
                        if ignore_config.include_binary_files {
                            match output_format {
                                OutputFormat::Xml => {
                                    output.push_str(&format!(
                                        "<file name=\"{}\">\n</file>\n",
                                        normalized_path
                                    ));
                                }
                                OutputFormat::Markdown => {
                                    output.push_str(&format!(
                                        "```{}\n<binary file>\n```\n\n",
                                        normalized_path
                                    ));
                                }
                                _ => {} // Other formats handled elsewhere
                            }
                            files += 1;
                        }
                    } else {
                        match output_format {
                            OutputFormat::Xml => {
                                output.push_str(&format!("<file name=\"{}\">\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(&entry) {
                                    let formatted_content = format_file_content(
                                        &content,
                                        show_line_numbers,
                                    );
                                    output.push_str(&xml_escape(&formatted_content));
                                }
                                output.push_str("</file>\n");
                                files += 1;
                            }
                            OutputFormat::Markdown => {
                                output.push_str(&format!("```{}\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(&entry) {
                                    output.push_str(&format_file_content(
                                        &content,
                                        show_line_numbers,
                                    ));
                                }
                                output.push_str("```\n\n");
                                files += 1;
                            }
                            _ => {} // Other formats handled elsewhere
                        }
                    }
                } else if entry.is_dir() {
                    folders += 1;
                    match output_format {
                        OutputFormat::Xml => {
                            output.push_str(&format!("<folder name=\"{}\">\n", normalized_path));
                            let mut dir_contents = String::new();
                            let stats = process_directory(
                                &entry,
                                &mut dir_contents,
                                base_path,
                                selected_items,
                                output_format,
                                show_line_numbers,
                                ignore_config,
                            )?;
                            files += stats.files;
                            folders += stats.folders;
                            output.push_str(&dir_contents);
                            output.push_str("</folder>\n");
                        }
                        OutputFormat::Markdown => {
                            output.push_str(&format!("### {}/\n\n", normalized_path));
                            let mut dir_contents = String::new();
                            let stats = process_directory(
                                &entry,
                                &mut dir_contents,
                                base_path,
                                selected_items,
                                output_format,
                                show_line_numbers,
                                ignore_config,
                            )?;
                            files += stats.files;
                            folders += stats.folders;
                            output.push_str(&dir_contents);
                        }
                        _ => {} // Other formats handled elsewhere
                    }
                }
            }
        }
    }

    Ok(CopyStats { files, folders })
}

// TODO: Add error handling for missing or unreadable files.
