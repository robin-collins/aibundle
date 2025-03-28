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

/// Main dispatcher function for formatting output based on the selected format.
pub fn format_selected_items(
    selected_items: &HashSet<PathBuf>,
    base_dir: &PathBuf,
    output_format: &OutputFormat,
    show_line_numbers: bool,
    ignore_config: &IgnoreConfig,
) -> io::Result<(String, CopyStats)> {
    match output_format {
        OutputFormat::Json => {
            format_json_output(selected_items, base_dir, ignore_config)
        }
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

pub fn get_language_name(extension: &str) -> &'static str {
    match extension {
        "py" => "Python",
        "c" => "C",
        "cpp" => "C++",
        "h" => "C/C++ Header",
        "hpp" => "C++ Header",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "java" => "Java",
        "html" => "HTML",
        "css" => "CSS",
        "php" => "PHP",
        "rb" => "Ruby",
        "go" => "Go",
        "rs" => "Rust",
        "swift" => "Swift",
        "kt" => "Kotlin",
        "sh" => "Shell",
        "md" => "Markdown",
        "json" => "JSON",
        "xml" => "XML",
        "yaml" => "YAML",
        "yml" => "YAML",
        "sql" => "SQL",
        "r" => "R",
        _ => "Plain Text",
    }
}
