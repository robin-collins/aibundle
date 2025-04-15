// src/output/mod.rs
//!
//! # Output Module Root
//!
//! This module is the entry point for all output formatting logic, including XML, Markdown, JSON, and LLM formats.
//! It re-exports the main formatting functions and types for use throughout the application.
//!
//! ## Submodules
//! - `format`: Core formatting utilities and helpers.
//! - `json`: JSON output formatting.
//! - `llm`: LLM (language model) output formatting.
//! - `markdown`: Markdown output formatting.
//! - `xml`: XML output formatting.
//!
//! ## Re-exports
//! The most commonly used formatting functions and types are re-exported for ergonomic access.
//!
//! ## Examples
//! ```rust
//! use crate::output::format_selected_items;
//! let (output, stats) = format_selected_items(&selected_items, &base_dir, &output_format, true, &ignore_config).unwrap();
//! println!("{}", output);
//! ```
pub mod format;
pub mod json;
pub mod llm;
pub mod markdown;
pub mod xml;

pub use format::*;
pub use json::*;
pub use llm::*;
pub use markdown::*;
pub use xml::*;

use crate::models::{CopyStats, IgnoreConfig, OutputFormat};
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use crate::models::constants::get_language_name;

/// Formats the selected items in the specified output format (XML, Markdown, JSON, LLM).
/// Returns the formatted output and copy statistics.
///
/// # Arguments
/// * `selected_items` - Set of selected file and directory paths.
/// * `base_dir` - The base directory for relative paths.
/// * `output_format` - The output format to use.
/// * `show_line_numbers` - Whether to include line numbers in file content.
/// * `ignore_config` - Ignore configuration.
///
/// # Returns
/// * `io::Result<(String, CopyStats)>` - The formatted output and copy statistics.
pub fn format_selected_items(
    selected_items: &HashSet<PathBuf>,
    base_dir: &PathBuf,
    output_format: &OutputFormat,
    show_line_numbers: bool,
    ignore_config: &IgnoreConfig,
) -> io::Result<(String, CopyStats)> {
    match output_format {
        OutputFormat::Json => format_json_output(selected_items, base_dir, ignore_config),
        OutputFormat::Markdown => {
            format_markdown_output(selected_items, base_dir, show_line_numbers, ignore_config)
        }
        OutputFormat::Xml => {
            format_xml_output(selected_items, base_dir, show_line_numbers, ignore_config)
        }
        OutputFormat::Llm => {
            // Assuming format_llm_output exists and has a compatible signature
            // We might need to adjust this call later if format_llm_output needs changes
            format_llm_output(selected_items, base_dir, ignore_config)
            // TODO: The monolithic LLM format included dependency analysis.
            // This basic call might need enhancement later to fully match.
        }
    }
}

// TODO: Add additional output formats (YAML, TOML, etc.) as needed.
