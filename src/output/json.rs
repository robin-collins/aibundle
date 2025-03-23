use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::fs::normalize_path;
use crate::models::CopyStats;
use crate::output::format::{is_binary_file, process_directory};

/// Format selected items as JSON
pub fn format_json_output(
    selected_items: &HashSet<PathBuf>,
    current_dir: &PathBuf,
    include_binary_files: bool,
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
                    if include_binary_files {
                        output.push_str(&format!(
                            "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":true}}",
                            normalized_path
                        ));
                        stats.files += 1;
                    }
                } else {
                    if let Ok(content) = fs::read_to_string(path) {
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
                        stats.files += 1;
                    }
                }
            } else if path.is_dir() {
                output.push_str(&format!(
                    "{{\"type\":\"directory\",\"path\":\"{}\",\"contents\":[",
                    normalized_path
                ));
                let mut dir_contents = String::new();
                if let Ok((files, folders)) = process_directory(
                    path,
                    &mut dir_contents,
                    current_dir,
                    selected_items,
                    &crate::models::OutputFormat::Json,
                    false, // JSON format doesn't use line numbers
                    include_binary_files,
                ) {
                    output.push_str(&dir_contents);
                    stats.files += files;
                    stats.folders += folders;
                }
                output.push_str("]}");
                stats.folders += 1;
            }
        }
    }

    output.push(']');

    Ok((output, stats))
}
