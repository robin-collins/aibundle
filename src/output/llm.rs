// src/output/llm.rs
//!
//! # LLM Output Formatter
//!
//! Provides functions for formatting selected files and directories as LLM-friendly output, including dependency analysis and project structure.
//!
//! ## Purpose
//! - Generate structured, annotated output for AI assistants and code analysis tools.
//! - Analyze dependencies and project structure for LLM consumption.
//!
//! ## Organization
//! - [`format_llm_output`]: Main entry point for LLM output formatting.
//! - [`analyze_dependencies`]: Analyzes file dependencies.
//! - [`format_llm_output_internal`]: Formats the final LLM output.
//!
//! ## Example
//! ```rust
//! use crate::output::llm::format_llm_output;
//! use std::collections::HashSet;
//! use std::path::PathBuf;
//! let (output, stats) = format_llm_output(&HashSet::new(), &PathBuf::from("."), &crate::models::IgnoreConfig::default()).unwrap();
//! println!("{}", output);
//! ```

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use itertools::Itertools;
use regex::Regex;

use crate::fs::normalize_path;
use crate::models::app_config::{FileDependencies, Node};
use crate::models::constants::get_language_name;
use crate::models::CopyStats;
use crate::output::format::is_binary_file;

/// Formats selected files and directories as LLM-friendly Markdown output, including dependencies and structure.
///
/// # Arguments
///
/// * `selected_items` - Set of selected file and directory paths.
/// * `current_dir` - The root directory for relative paths.
/// * `ignore_config` - Ignore configuration.
///
/// # Returns
///
/// * `io::Result<(String, CopyStats)>` - The formatted output and copy statistics.
///
/// # Errors
///
/// Returns an error if a file or directory cannot be read.
///
/// # Examples
/// ```rust
/// use crate::output::llm::format_llm_output;
/// use std::collections::HashSet;
/// use std::path::PathBuf;
/// let (output, stats) = format_llm_output(&HashSet::new(), &PathBuf::from("."), &crate::models::IgnoreConfig::default()).unwrap();
/// println!("{}", output);
/// ```
pub fn format_llm_output(
    selected_items: &HashSet<PathBuf>,
    current_dir: &PathBuf,
    ignore_config: &crate::models::IgnoreConfig,
) -> io::Result<(String, CopyStats)> {
    let mut output = String::new();

    // Collect file contents in a format suitable for dependency analysis
    let mut file_contents = Vec::new();
    let mut file_count = 0;
    let mut folder_count = 0;

    // Create a tree structure for the file system
    let root_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("root")
        .to_string();

    let mut root_node = Node {
        name: root_name.clone(),
        is_dir: true,
        children: Some(HashMap::new()),
    };

    // Process selected items to collect files and build tree
    for path in selected_items {
        // Handle the case where paths might be relative and current_dir is "."
        let rel_path = if current_dir == Path::new(".") && path.is_relative() {
            path.as_path()
        } else if let Ok(stripped) = path.strip_prefix(current_dir) {
            stripped
        } else {
            path.as_path()
        };

        if rel_path.as_os_str().is_empty() {
            continue; // Skip root
        }

        // Add to tree structure
        add_path_to_tree(&mut root_node, rel_path, path.is_dir());

        if path.is_file() {
            file_count += 1;

            // Read file content if not binary and not ignored
            if !is_binary_file(path) || ignore_config.include_binary_files {
                if let Ok(content) = fs::read_to_string(path) {
                                            let normalized_path = normalize_path(&rel_path.to_string_lossy());
                        file_contents.push((normalized_path, content));
                                    }
            }
        } else if path.is_dir() {
            folder_count += 1;

            // Add all files in directory to file_contents
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(Result::ok) {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        // Handle the case where paths might be relative and current_dir is "."
                        let rel_entry_path = if current_dir == Path::new(".") && entry_path.is_relative() {
                            entry_path.as_path()
                        } else if let Ok(stripped) = entry_path.strip_prefix(current_dir) {
                            stripped
                        } else {
                            entry_path.as_path()
                        };

                        if !is_binary_file(&entry_path)
                            || ignore_config.include_binary_files
                        {
                            if let Ok(content) = fs::read_to_string(&entry_path) {
                                let normalized_path =
                                    normalize_path(&rel_entry_path.to_string_lossy());
                                file_contents.push((normalized_path, content));
                            }
                        }
                    }
                }
            }
        }
    }

    // Also collect all files for comprehensive dependency analysis
    let all_file_contents = collect_all_files_for_analysis(selected_items, current_dir, ignore_config);

    // Analyze dependencies using all available files for better resolution
    let dependencies = analyze_dependencies(&all_file_contents, current_dir);

    // Generate LLM output
    format_llm_output_internal(
        &mut output,
        &file_contents,
        current_dir,
        &root_node,
        &dependencies,
        file_count,
        folder_count,
    );

    let stats = CopyStats {
        files: file_count,
        folders: folder_count,
    };

    Ok((output, stats))
}

/// Collects all files from selected items for comprehensive dependency analysis
fn collect_all_files_for_analysis(
    selected_items: &HashSet<PathBuf>,
    current_dir: &PathBuf,
    ignore_config: &crate::models::IgnoreConfig,
) -> Vec<(String, String)> {
    let mut all_files = Vec::new();

    for path in selected_items {
        if path.is_file() {
            // Handle the case where paths might be relative and current_dir is "."
            let rel_path = if current_dir == Path::new(".") && path.is_relative() {
                path.as_path()
            } else if let Ok(stripped) = path.strip_prefix(current_dir) {
                stripped
            } else {
                path.as_path()
            };

            if !is_binary_file(path) || ignore_config.include_binary_files {
                if let Ok(content) = fs::read_to_string(path) {
                    let normalized_path = normalize_path(&rel_path.to_string_lossy());
                    all_files.push((normalized_path, content));
                }
            }
        } else if path.is_dir() {
            // For directories, collect all files within them
            collect_files_from_directory(path, current_dir, ignore_config, &mut all_files);
        }
    }

    all_files
}

/// Recursively collect files from a directory for analysis
fn collect_files_from_directory(
    dir_path: &PathBuf,
    current_dir: &PathBuf,
    ignore_config: &crate::models::IgnoreConfig,
    all_files: &mut Vec<(String, String)>,
) {
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.filter_map(Result::ok) {
            let entry_path = entry.path();

            if entry_path.is_file() {
                // Handle the case where paths might be relative and current_dir is "."
                let rel_path = if current_dir == Path::new(".") && entry_path.is_relative() {
                    entry_path.as_path()
                } else if let Ok(stripped) = entry_path.strip_prefix(current_dir) {
                    stripped
                } else {
                    entry_path.as_path()
                };

                if !is_binary_file(&entry_path) || ignore_config.include_binary_files {
                    if let Ok(content) = fs::read_to_string(&entry_path) {
                        let normalized_path = normalize_path(&rel_path.to_string_lossy());
                        all_files.push((normalized_path, content));
                    }
                }
            } else if entry_path.is_dir() {
                collect_files_from_directory(&entry_path, current_dir, ignore_config, all_files);
            }
        }
    }
}

/// Add a path to the tree structure
fn add_path_to_tree(root: &mut Node, rel_path: &Path, is_dir: bool) {
    let mut current_node = root;
    let components: Vec<_> = rel_path.iter().collect();

    for (i, comp) in components.iter().enumerate() {
        let name = comp.to_string_lossy().to_string();
        let children = current_node.children.get_or_insert_with(HashMap::new);

        let is_final = i == components.len() - 1;
        let node_is_dir = if is_final { is_dir } else { true };

        current_node = children.entry(name.clone()).or_insert_with(|| Node {
            name: name.clone(),
            is_dir: node_is_dir,
            children: if node_is_dir {
                Some(HashMap::new())
            } else {
                None
            },
        });

        // Update the final node to correct type
        if is_final {
            current_node.is_dir = is_dir;
            if !is_dir {
                current_node.children = None;
            }
        }
    }
}

/// Analyzes dependencies between files based on language-specific import/include patterns.
///
/// # Arguments
///
/// * `file_contents` - List of (relative path, file content) tuples.
/// * `_base_dir` - The base directory (unused).
///
/// # Returns
///
/// * `HashMap<String, FileDependencies>` - Map of file paths to their dependencies.
///
/// # Examples
/// ```rust
/// use crate::output::llm::analyze_dependencies;
/// let deps = analyze_dependencies(&vec![("main.rs".to_string(), "mod foo;".to_string())], &std::path::PathBuf::from("."));
/// assert!(deps.contains_key("main.rs"));
/// ```
pub fn analyze_dependencies(
    file_contents: &[(String, String)],
    _base_dir: &Path,
) -> HashMap<String, FileDependencies> {
    let mut dependencies = HashMap::new();
    let mut imports: HashMap<String, HashSet<String>> = HashMap::new();

    // Define detection patterns for different languages - improved patterns
    let language_patterns: HashMap<&str, Vec<&str>> = [
        // Python
        (
            ".py",
            vec![
                r"^from\s+([\w.]+)\s+import",
                r"^import\s+([\w.]+)",
                r"^\s*from\s+([\w.]+)\s+import",
                r"^\s*import\s+([\w.]+)",
            ],
        ),
        // C/C++
        (".c", vec![r#"#include\s+[<"]([^>"]+)[>"]"#]),
        (".h", vec![r#"#include\s+[<"]([^>"]+)[>"]"#]),
        (".cpp", vec![r#"#include\s+[<"]([^>"]+)[>"]"#]),
        (".hpp", vec![r#"#include\s+[<"]([^>"]+)[>"]"#]),
        (".cc", vec![r#"#include\s+[<"]([^>"]+)[>"]"#]),
        // Rust
        (
            ".rs",
            vec![
                r"use\s+([\w:]+)",
                r"extern\s+crate\s+([\w]+)",
                r"mod\s+([\w]+)",
                r"pub\s+use\s+([\w:]+)",
            ],
        ),
        // JavaScript/TypeScript
        (
            ".js",
            vec![
                r#"(?:import|require)\s*\(?['"]([^'"]+)['"]"#,
                r#"from\s+['"]([^'"]+)['"]"#,
                r#"import\s+.*?\s+from\s+['"]([^'"]+)['"]"#,
                r#"const\s+.*?\s*=\s*require\s*\(\s*['"]([^'"]+)['"]\s*\)"#,
            ],
        ),
        (
            ".ts",
            vec![
                r#"(?:import|require)\s*\(?['"]([^'"]+)['"]"#,
                r#"from\s+['"]([^'"]+)['"]"#,
                r#"import\s+.*?\s+from\s+['"]([^'"]+)['"]"#,
                r#"const\s+.*?\s*=\s*require\s*\(\s*['"]([^'"]+)['"]\s*\)"#,
            ],
        ),
        (
            ".tsx",
            vec![
                r#"(?:import|require)\s*\(?['"]([^'"]+)['"]"#,
                r#"from\s+['"]([^'"]+)['"]"#,
                r#"import\s+.*?\s+from\s+['"]([^'"]+)['"]"#,
            ],
        ),
        (
            ".jsx",
            vec![
                r#"(?:import|require)\s*\(?['"]([^'"]+)['"]"#,
                r#"from\s+['"]([^'"]+)['"]"#,
                r#"import\s+.*?\s+from\s+['"]([^'"]+)['"]"#,
            ],
        ),
        // Java
        (".java", vec![r"import\s+([\w.]+)"]),
        // Go
        (
            ".go",
            vec![
                r#"import\s+\(\s*(?:[_\w]*\s+)?["]([^"]+)["]"#,
                r#"import\s+(?:[_\w]*\s+)?["]([^"]+)["]"#,
            ],
        ),
        // Ruby
        (
            ".rb",
            vec![
                r#"require\s+['"]([^'"]+)['"]"#,
                r#"require_relative\s+['"]([^'"]+)['"]"#,
                r#"load\s+['"]([^'"]+)['"]"#,
            ],
        ),
        // PHP
        (
            ".php",
            vec![
                r#"(?:require|include)(?:_once)?\s*\(?['"]([^'"]+)['"]"#,
                r"use\s+([\w\\]+)",
            ],
        ),
        // Swift
        (".swift", vec![r"import\s+(\w+)"]),
        // Shell scripts
        (
            ".sh",
            vec![
                r#"source\s+['"]?([^'"]+)['"]?"#,
                r#"\.\s+['"]?([^'"]+)['"]?"#,
            ],
        ),
        (
            ".bash",
            vec![
                r#"source\s+['"]?([^'"]+)['"]?"#,
                r#"\.\s+['"]?([^'"]+)['"]?"#,
            ],
        ),
        // Makefile
        ("Makefile", vec![r"include\s+([^\s]+)"]),
        ("makefile", vec![r"include\s+([^\s]+)"]),
        // TOML
        (".toml", vec![]), // Could add dependency patterns for Cargo.toml etc.
        // YAML
        (".yaml", vec![]),
        (".yml", vec![]),
    ]
    .iter()
    .cloned()
    .collect();

    // Precompile regexes for each language extension
    let mut language_regexes: HashMap<&str, Vec<Regex>> = HashMap::new();
    for (ext, patterns) in &language_patterns {
        let compiled: Vec<Regex> = patterns
            .iter()
            .filter_map(|pat| match Regex::new(pat) {
                Ok(re) => Some(re),
                Err(_err) => {
                    // eprintln!("Invalid regex for {}: {}", ext, err);
                    None
                }
            })
            .collect();
        language_regexes.insert(*ext, compiled);
    }

    // First pass: collect all imports
    for (file_path, content) in file_contents {
        imports.insert(file_path.clone(), HashSet::new());

        let ext = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();

        let basename = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Select appropriate regexes for this file
        let regexes = if let Some(ext_regexes) = language_regexes.get(ext.as_str()) {
            ext_regexes
        } else if let Some(file_regexes) = language_regexes.get(basename) {
            file_regexes
        } else {
            continue;
        };

        // Apply all relevant regexes - search line by line for better matches
        for line in content.lines() {
            for re in regexes {
                for cap in re.captures_iter(line) {
                    if let Some(m) = cap.get(1) {
                        imports
                            .get_mut(file_path)
                            .unwrap()
                            .insert(m.as_str().to_string());
                    }
                }
            }
        }
    }

    // Second pass: resolve references between files
    let mut file_mapping = HashMap::new();

    for (file_path, _) in file_contents {
        let basename = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let name_without_ext = Path::new(&basename)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Add different forms of file name
        file_mapping.insert(basename.clone(), file_path.clone());
        file_mapping.insert(name_without_ext, file_path.clone());
        file_mapping.insert(file_path.clone(), file_path.clone());

        // For paths with folders, also add relative variants
        let mut rel_path = file_path.clone();
        while rel_path.contains('/') {
            rel_path = rel_path[rel_path.find('/').unwrap() + 1..].to_string();
            file_mapping.insert(rel_path.clone(), file_path.clone());

            let without_ext = Path::new(&rel_path)
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            file_mapping.insert(without_ext, file_path.clone());
        }
    }

    // Resolve imports to file dependencies
    for (file_path, imported) in imports {
        let mut internal_deps = Vec::new();
        let mut external_deps = Vec::new();

        for imp in imported {
            // Try to match import with a known file
            let mut matched = false;

            // Try variations of the import to find a match
            let import_variations = vec![
                imp.clone(),
                Path::new(&imp)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                Path::new(&imp)
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                imp.replace('.', "/"),
                imp.replace("::", "/"),                   // For Rust modules
                format!("{}.py", imp.replace('.', "/")),  // For Python
                format!("{}.rs", imp.replace("::", "/")), // For Rust
                format!("{}.h", imp),                     // For C
                format!("{}.hpp", imp),                   // For C++
                format!("{}.js", imp),                    // For JS
                format!("{}.ts", imp),                    // For TS
                format!("{}/mod.rs", imp.replace("::", "/")), // For Rust modules
                format!("{}/index.js", imp),              // For JS modules
                format!("{}/index.ts", imp),              // For TS modules
            ];

            for var in import_variations {
                if let Some(matched_path) = file_mapping.get(&var) {
                    if matched_path != &file_path {
                        // Don't self-reference
                        internal_deps.push(matched_path.clone());
                        matched = true;
                        break;
                    }
                }
            }

            // If no match found, keep the import as is (external dependency)
            if !matched {
                external_deps.push(imp);
            }
        }

        // Remove duplicates
        internal_deps.sort();
        internal_deps.dedup();
        external_deps.sort();
        external_deps.dedup();

        dependencies.insert(
            file_path,
            FileDependencies {
                internal_deps,
                external_deps,
            },
        );
    }

    dependencies
}

// Helper function to write file tree to string with Unicode box drawing characters
fn write_file_tree_to_string(node: &Node, prefix: &str, is_last: bool) -> String {
    let mut result = String::new();

    // Print node (skip root when prefix is empty)
    if !prefix.is_empty() {
        // Use Unicode box drawing characters for better visual appearance
        let branch = if is_last { "└── " } else { "├── " };
        result.push_str(&format!("{}{}{}\n", prefix, branch, node.name));
    }

    if node.is_dir {
        if let Some(children) = node.children.as_ref() {
            let items: Vec<_> = children
                .iter()
                .sorted_by(|a, b| {
                    Ord::cmp(
                        &(!a.1.is_dir, a.0.to_lowercase()),
                        &(!b.1.is_dir, b.0.to_lowercase()),
                    )
                })
                .collect();

            for (i, (_, child)) in items.iter().enumerate() {
                let is_last_child = i == items.len() - 1;
                // Use Unicode vertical bar for the tree structure
                let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                result.push_str(&write_file_tree_to_string(
                    child,
                    &new_prefix,
                    is_last_child,
                ));
            }
        }
    }

    result
}

// Helper function to count files in a node tree
#[allow(dead_code)]
fn count_files(node: &Node) -> usize {
    if !node.is_dir {
        return 1;
    }

    let mut count = 0;
    if let Some(children) = &node.children {
        for child in children.values() {
            count += count_files(child);
        }
    }
    count
}

/// Formats the LLM output, including project info, structure, dependencies, and file contents.
///
/// # Arguments
///
/// * `output` - Mutable string to write output to.
/// * `file_contents` - List of (relative path, file content) tuples.
/// * `root_path` - The root directory path.
/// * `root_node` - The root node of the file tree.
/// * `dependencies` - Map of file dependencies.
/// * `total_files` - Total number of files selected.
/// * `_total_folders` - Total number of folders selected (unused).
///
/// # Examples
/// ```rust
/// use crate::output::llm::format_llm_output_internal;
/// use crate::models::app_config::{FileDependencies, Node};
/// use std::collections::HashMap;
/// let mut output = String::new();
/// let file_contents = vec![("main.rs".to_string(), "mod foo;".to_string())];
/// let root_node = Node { name: "root".to_string(), is_dir: true, children: Some(HashMap::new()) };
/// let dependencies = HashMap::new();
/// format_llm_output_internal(&mut output, &file_contents, &std::path::PathBuf::from("."), &root_node, &dependencies, 1, 0);
/// assert!(output.contains("PROJECT ANALYSIS"));
/// ```
pub fn format_llm_output_internal(
    output: &mut String,
    file_contents: &[(String, String)],
    root_path: &Path,
    root_node: &Node,
    dependencies: &HashMap<String, FileDependencies>,
    total_files: usize,
    _total_folders: usize,
) {
    // Header and overview
    output.push_str("# PROJECT ANALYSIS FOR AI ASSISTANT\n\n");

    // General project information
    let selected_files = file_contents.len();
    output.push_str("## 📦 GENERAL INFORMATION\n\n");
    output.push_str(&format!("- **Project path**: `{}`\n", root_path.display()));
    output.push_str(&format!("- **Total files**: {}\n", total_files));
    output.push_str(&format!(
        "- **Files included in this analysis**: {}\n",
        selected_files
    ));

    // Detect languages used
    let mut languages: HashMap<String, usize> = HashMap::new();
    for (path, _) in file_contents {
        if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
            *languages.entry(ext.to_string()).or_insert(0) += 1;
        }
    }

    if !languages.is_empty() {
        output.push_str("- **Main languages used**:\n");
        let mut lang_counts: Vec<_> = languages.iter().collect();
        lang_counts.sort_by(|a, b| b.1.cmp(a.1));

        for (i, (ext, count)) in lang_counts.iter().enumerate() {
            if i >= 5 {
                break;
            } // Show top 5 languages
            let lang_name = get_language_name(ext);
            output.push_str(&format!("  - {} ({} files)\n", lang_name, count));
        }
    }
    output.push('\n');

    // Project structure
    output.push_str("## 🗂️ PROJECT STRUCTURE\n\n");
    output.push_str("```\n");
    output.push_str(&format!("{}\n", root_path.display()));
    output.push_str(&write_file_tree_to_string(root_node, "", true));
    output.push_str("```\n\n");

    // Main directories and components
    let main_dirs: Vec<_> = root_node
        .children
        .as_ref()
        .map(|children| children.values().filter(|node| node.is_dir).collect())
        .unwrap_or_default();

    if !main_dirs.is_empty() {
        output.push_str("### 📂 Main Components\n\n");
        for dir_node in main_dirs {
            let dir_files: Vec<_> = file_contents
                .iter()
                .filter(|(p, _)| p.starts_with(&format!("{}/", dir_node.name)))
                .collect();

            output.push_str(&format!("- **`{}/`** - ", dir_node.name));
            if !dir_files.is_empty() {
                output.push_str(&format!("Contains {} files", dir_files.len()));

                // Languages in this directory
                let mut dir_exts: HashMap<String, usize> = HashMap::new();
                for (path, _) in &dir_files {
                    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
                        *dir_exts.entry(ext.to_string()).or_insert(0) += 1;
                    }
                }

                if !dir_exts.is_empty() {
                    let main_langs = dir_exts
                        .iter()
                        .sorted_by(|a, b| b.1.cmp(a.1))
                        .take(2)
                        .map(|(ext, _)| get_language_name(ext))
                        .collect::<Vec<_>>();

                    if !main_langs.is_empty() {
                        output.push_str(&format!(" mainly in {}", main_langs.join(", ")));
                    }
                }
            }
            output.push('\n');
        }
        output.push('\n');
    }

    // File relationship graph
    output.push_str("## 🔄 FILE RELATIONSHIPS\n\n");

    // Find most referenced files
    let mut referenced_by: HashMap<String, Vec<String>> = HashMap::new();
    for (file, deps) in dependencies {
        for dep in &deps.internal_deps {
            referenced_by
                .entry(dep.clone())
                .or_default()
                .push(file.clone());
        }
    }

    // Display important relationships
    if !referenced_by.is_empty() {
        output.push_str("### Core Files (most referenced)\n\n");
        let mut refs: Vec<_> = referenced_by.iter().collect();
        refs.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (file, refs) in refs.iter().take(10) {
            if refs.len() > 1 {
                // Only files referenced multiple times
                output.push_str(&format!(
                    "- **`{}`** is imported by {} files\n",
                    file,
                    refs.len()
                ));
            }
        }
        output.push('\n');
    }

    // Display dependencies per file
    output.push_str("### Dependencies by File\n\n");
    let mut dep_items: Vec<_> = dependencies.iter().collect();
    dep_items.sort_by(|a, b| a.0.cmp(b.0));

    for (file, deps) in dep_items {
        if !deps.internal_deps.is_empty() || !deps.external_deps.is_empty() {
            output.push_str(&format!("- **`{}`**:\n", file));

            if !deps.internal_deps.is_empty() {
                output.push_str("  - *Internal dependencies*: ");
                let mut sorted_deps = deps.internal_deps.clone();
                sorted_deps.sort();
                let display_deps: Vec<_> = sorted_deps
                    .iter()
                    .take(5)
                    .map(|d| format!("`{}`", d))
                    .collect();
                output.push_str(&display_deps.join(", "));
                if deps.internal_deps.len() > 5 {
                    output.push_str(&format!(" and {} more", deps.internal_deps.len() - 5));
                }
                output.push('\n');
            }

            if !deps.external_deps.is_empty() {
                output.push_str("  - *External dependencies*: ");
                let mut sorted_deps = deps.external_deps.clone();
                sorted_deps.sort();
                let display_deps: Vec<_> = sorted_deps
                    .iter()
                    .take(5)
                    .map(|d| format!("`{}`", d))
                    .collect();
                output.push_str(&display_deps.join(", "));
                if deps.external_deps.len() > 5 {
                    output.push_str(&format!(" and {} more", deps.external_deps.len() - 5));
                }
                output.push('\n');
            }
        }
    }
    output.push('\n');

    // File contents
    output.push_str("## 📄 FILE CONTENTS\n\n");
    output.push_str("*Note: The content below includes only selected files.*\n\n");

    for (path, content) in file_contents {
        output.push_str(&format!("### {}\n\n", path));

        // Add file info if available
        if let Some(file_deps) = dependencies.get(path) {
            if !file_deps.internal_deps.is_empty() || !file_deps.external_deps.is_empty() {
                output.push_str("**Dependencies:**\n");

                if !file_deps.internal_deps.is_empty() {
                    let mut sorted_deps = file_deps.internal_deps.clone();
                    sorted_deps.sort();
                    output.push_str("- Internal: ");
                    let display_deps: Vec<_> = sorted_deps
                        .iter()
                        .take(3)
                        .map(|d| format!("`{}`", d))
                        .collect();
                    output.push_str(&display_deps.join(", "));
                    if file_deps.internal_deps.len() > 3 {
                        output
                            .push_str(&format!(" and {} more", file_deps.internal_deps.len() - 3));
                    }
                    output.push('\n');
                }

                if !file_deps.external_deps.is_empty() {
                    let mut sorted_deps = file_deps.external_deps.clone();
                    sorted_deps.sort();
                    output.push_str("- External: ");
                    let display_deps: Vec<_> = sorted_deps
                        .iter()
                        .take(3)
                        .map(|d| format!("`{}`", d))
                        .collect();
                    output.push_str(&display_deps.join(", "));
                    if file_deps.external_deps.len() > 3 {
                        output
                            .push_str(&format!(" and {} more", file_deps.external_deps.len() - 3));
                    }
                    output.push('\n');
                }

                output.push('\n');
            }
        }

        // Syntax highlighting based on extension
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        output.push_str(&format!("```{}\n", ext));
        output.push_str(content);
        if !content.ends_with('\n') {
            output.push('\n');
        }
        output.push_str("```\n\n");
    }
}

// TODO: Add support for more language-specific dependency patterns.
// TODO: Add error handling for malformed or missing files.
