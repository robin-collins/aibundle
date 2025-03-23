use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;

use itertools::Itertools;

use crate::fs::normalize_path;
use crate::models::{CopyStats, FileDependencies, Node};
use crate::output::get_language_name;
use crate::output::format::is_binary_file;

/// Format selected items as LLM (AI-friendly markdown)
pub fn format_llm_output(
    selected_items: &HashSet<PathBuf>,
    current_dir: &PathBuf,
) -> io::Result<(String, CopyStats)> {
    let mut output = String::new();
    let mut stats = CopyStats {
        files: 0,
        folders: 0,
    };

    // Collect file contents in a format suitable for dependency analysis
    let mut file_contents = Vec::new();

    // Create a tree structure for the file system
    let root_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("root")
        .to_string();

    let mut root_node = Node {
        name: root_name,
        is_dir: true,
        children: Some(HashMap::new()),
        parent: None,
    };

    // Build the tree structure from selected items
    let mut node_map: HashMap<PathBuf, *mut Node> = HashMap::new();
    let root_ptr: *mut Node = &mut root_node;
    node_map.insert(current_dir.clone(), root_ptr);

    // First add directories
    let mut sorted_items: Vec<_> = selected_items.iter().collect();
    sorted_items.sort_by_key(|p| (p.is_dir(), p.to_string_lossy().to_string()));

    for path in sorted_items {
        if let Ok(rel_path) = path.strip_prefix(current_dir) {
            if rel_path.as_os_str().is_empty() {
                continue; // Skip root
            }

            // Get the parent path
            let parent_path = if let Some(parent) = path.parent() {
                parent.to_path_buf()
            } else {
                current_dir.clone()
            };

            // Get parent node pointer
            let parent_ptr = *node_map.get(&parent_path).unwrap_or(&root_ptr);

            // Create and add the node
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Only add if not already in the tree
            if let Some(children) = unsafe { &mut (*parent_ptr).children } {
                let name_clone = name.clone(); // Clone before using in entry
                children.entry(name_clone).or_insert_with(|| {
                    let mut node = Node {
                        name: name.clone(),
                        is_dir: path.is_dir(),
                        children: if path.is_dir() {
                            Some(HashMap::new())
                        } else {
                            None
                        },
                        parent: None, // We don't need this for tree rendering
                    };

                    let node_ptr: *mut Node = &mut node;
                    node_map.insert(path.clone(), node_ptr);
                    node
                });
            }
        }
    }

    // Process selected items
    for path in selected_items {
        if path.is_file() {
            stats.files += 1;

            if !is_binary_file(path) {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(rel_path) = path.strip_prefix(current_dir) {
                        let normalized_path = normalize_path(&rel_path.to_string_lossy());
                        file_contents.push((normalized_path, content));
                    }
                }
            }
        } else if path.is_dir() {
            stats.folders += 1;
        }
    }

    // Analyze dependencies
    let dependencies = analyze_dependencies(&file_contents, current_dir);

    // Generate LLM output
    format_llm_output_internal(
        &mut output,
        &file_contents,
        current_dir,
        &root_node,
        &dependencies,
    );

    Ok((output, stats))
}

// Helper function to analyze dependencies between files
fn analyze_dependencies(
    file_contents: &[(String, String)],
    _base_dir: &Path,
) -> HashMap<String, FileDependencies> {
    let mut dependencies = HashMap::new();
    let mut imports: HashMap<String, HashSet<String>> = HashMap::new();

    // Define detection patterns for different languages
    let language_patterns: HashMap<&str, Vec<&str>> = [
        // Python
        (
            ".py",
            vec![r"^from\s+([\w.]+)\s+import", r"^import\s+([\w.]+)"],
        ),
        // C/C++
        (".c", vec![r#"#include\s+[<"]([^>"]+)[>"]"#]),
        (".h", vec![r#"#include\s+[<"]([^>"]+)[>"]"#]),
        (".cpp", vec![r#"#include\s+[<"]([^>"]+)[>"]"#]),
        (".hpp", vec![r#"#include\s+[<"]([^>"]+)[>"]"#]),
        // JavaScript/TypeScript
        (
            ".js",
            vec![
                r#"(?:import|require)\s*\(?['"]([^'"]+)['"]"#,
                r#"from\s+['"]([^'"]+)['"]"#,
            ],
        ),
        (
            ".ts",
            vec![
                r#"(?:import|require)\s*\(?['"]([^'"]+)['"]"#,
                r#"from\s+['"]([^'"]+)['"]"#,
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
        // Rust
        (".rs", vec![r"use\s+([\w:]+)", r"extern\s+crate\s+([\w]+)"]),
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
        // Makefile
        ("Makefile", vec![r"include\s+([^\s]+)"]),
    ]
    .iter()
    .cloned()
    .collect();

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

        // Select appropriate patterns
        let patterns = if let Some(ext_patterns) = language_patterns.get(ext.as_str()) {
            ext_patterns
        } else if let Some(file_patterns) = language_patterns.get(basename) {
            file_patterns
        } else {
            continue;
        };

        // Apply all relevant patterns
        for pattern in patterns {
            let regex = match regex::Regex::new(pattern) {
                Ok(re) => re,
                Err(_) => continue,
            };

            for cap in regex.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    imports
                        .get_mut(file_path)
                        .unwrap()
                        .insert(m.as_str().to_string());
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
                format!("{}.py", imp.replace('.', "/")),
                format!("{}.h", imp),
                format!("{}.hpp", imp),
                format!("{}.js", imp),
            ];

            for var in import_variations {
                if let Some(matched_path) = file_mapping.get(&var) {
                    internal_deps.push(matched_path.clone());
                    matched = true;
                    break;
                }
            }

            // If no match found, keep the import as is
            if !matched {
                external_deps.push(imp);
            }
        }

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

// Helper function to write file tree to string
fn write_file_tree_to_string(node: &Node, prefix: &str, is_last: bool) -> String {
    let mut result = String::new();

    if node.parent.is_some() {
        // Skip root node
        let branch = if is_last { "── " } else { "├─ " };
        result.push_str(&format!("{}{}{}\n", prefix, branch, node.name));
    }

    if node.is_dir && node.children.is_some() {
        let children = node.children.as_ref().unwrap();
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
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            result.push_str(&write_file_tree_to_string(
                child,
                &new_prefix,
                is_last_child,
            ));
        }
    }

    result
}

// Helper function to count files in a node tree
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

// Internal function to format LLM output
fn format_llm_output_internal(
    output: &mut String,
    file_contents: &[(String, String)],
    root_path: &Path,
    root_node: &Node,
    dependencies: &HashMap<String, FileDependencies>,
) {
    // Header and overview
    output.push_str("# PROJECT ANALYSIS FOR AI ASSISTANT\n\n");

    // General project information
    let total_files = count_files(root_node);
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
    for (file, deps) in dependencies {
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
    output.push_str("## 🔄 FILE CONTENTS\n\n");
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