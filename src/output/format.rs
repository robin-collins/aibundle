use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::fs::normalize_path;
use crate::models::{CopyStats, IgnoreConfig, OutputFormat};

// Get language name from file extension
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

// Helper function to check if a file is binary
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

    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let binary_files = ["index"];
        return binary_files.contains(&name);
    }
    false
}

// Format files with line numbers if needed
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

// Process a directory recursively and add its contents to the output
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
                        let escaped_content = content
                            .replace('\\', "\\\\")
                            .replace('\"', "\\\"")
                            .replace('\n', "\\n")
                            .replace('\r', "\\r");
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
                                    output.push_str(&format_file_content(
                                        &content,
                                        show_line_numbers,
                                    ));
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
