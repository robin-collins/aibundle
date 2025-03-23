## Stage 5: Expand Filesystem Module

### Stage 5 Goal
Expand the existing filesystem module to include all file and directory operations.

### Stage 5 Steps

1. Update `src/fs/mod.rs` to include additional functions:

```rust
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::fs;
use std::collections::HashSet;
use crate::models::IgnoreConfig;
use ignore::{gitignore::GitignoreBuilder, Match};

pub fn confirm_overwrite(file_path: &str) -> io::Result<bool> {
    if Path::new(file_path).exists() {
        println!("File '{}' already exists. Overwrite? (y/n): ", file_path);
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        return Ok(input.trim().eq_ignore_ascii_case("y"));
    }
    Ok(true)
}

pub fn list_files(path: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !is_excluded(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn is_excluded(path: &std::path::Path) -> bool {
    let excluded = ["node_modules", ".git", "target"];
    path.components()
        .any(|c| excluded.contains(&c.as_os_str().to_str().unwrap_or("")))
}

pub fn add_items_iterative(
    items: &mut Vec<PathBuf>,
    root: &PathBuf,
    expanded_folders: &HashSet<PathBuf>,
    ignore_config: &IgnoreConfig,
    base_dir: &PathBuf,
) -> io::Result<()> {
    // Only add ".." for the root directory (not for expanded subdirectories)
    if items.is_empty() && root == base_dir {
        if let Some(parent) = root.parent() {
            if !parent.as_os_str().is_empty() {
                items.push(root.join(".."));
            }
        }
    }

    // Process current directory
    let mut entries: Vec<PathBuf> = fs::read_dir(root)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    entries.retain(|p| !is_path_ignored_for_iterative(p, base_dir, ignore_config));
    entries.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    // Add entries and recursively process expanded folders
    for entry in entries {
        items.push(entry.clone());
        if entry.is_dir() && expanded_folders.contains(&entry) {
            add_items_iterative(items, &entry, expanded_folders, ignore_config, base_dir)?;
        }
    }

    Ok(())
}

pub fn is_path_ignored_for_iterative(
    path: &PathBuf,
    base_dir: &PathBuf,
    ignore_config: &IgnoreConfig,
) -> bool {
    if !ignore_config.use_default_ignores && !ignore_config.use_gitignore {
        return false;
    }
    if ignore_config.use_default_ignores {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if crate::models::DEFAULT_IGNORED_DIRS.contains(&name) {
                return true;
            }
        }
    }
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if ignore_config
            .extra_ignore_patterns
            .contains(&name.to_string())
        {
            return true;
        }
    }
    if ignore_config.use_gitignore {
        let mut builder = GitignoreBuilder::new(base_dir);
        let mut dir = base_dir.clone();
        while let Some(parent) = dir.parent() {
            let gitignore = dir.join(".gitignore");
            if gitignore.exists() {
                match builder.add(gitignore) {
                    None => (),
                    Some(_) => break,
                }
            }
            dir = parent.to_path_buf();
        }
        if let Ok(gitignore) = builder.build() {
            let is_dir = path.is_dir();
            if let Match::Ignore(_) = gitignore.matched_path_or_any_parents(path, is_dir) {
                return true;
            }
        }
    }
    false
}

pub fn collect_all_subdirs(
    base_dir: &Path,
    ignore_config: &IgnoreConfig,
) -> io::Result<HashSet<PathBuf>> {
    let base_dir_buf = base_dir.to_path_buf();
    let mut dirs = HashSet::new();
    let mut stack = vec![base_dir_buf.clone()];
    while let Some(current) = stack.pop() {
        if current.is_dir() {
            dirs.insert(current.clone());
            for entry in fs::read_dir(&current)?.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && !is_path_ignored_for_iterative(&path, &base_dir_buf, ignore_config)
                {
                    stack.push(path);
                }
            }
        }
    }
    Ok(dirs)
}

pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn count_selection_items_async(
    path: &Path,
    base_dir: &PathBuf,
    ignore_config: &IgnoreConfig,
    selection_limit: usize,
) -> io::Result<usize> {
    if path.is_file() {
        return Ok(1);
    }
    if path.is_dir() {
        let mut count = 0;
        let mut stack = vec![path.to_path_buf()];

        while let Some(current) = stack.pop() {
            if is_path_ignored_for_iterative(&current, base_dir, ignore_config) {
                continue;
            }
            if current.is_file() {
                count += 1;
            } else if current.is_dir() {
                count += 1;
                if count > selection_limit {
                    return Ok(count);
                }
                let entries = fs::read_dir(&current)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path());
                for entry_path in entries {
                    stack.push(entry_path);
                }
            }
            if count > selection_limit {
                return Ok(count);
            }
        }
        Ok(count)
    } else {
        Ok(0)
    }
}

pub fn is_binary_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let binary_extensions = [
            "idx", "pack", "rev", "index", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp",
            "ico", "svg", "mp3", "wav", "ogg", "flac", "m4a", "aac", "wma", "mp4", "avi",
            "mkv", "mov", "wmv", "flv", "webm", "zip", "rar", "7z", "tar", "gz", "iso", "exe",
            "dll", "so", "dylib", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "class",
            "pyc", "pyd", "pyo",
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
```
