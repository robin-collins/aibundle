// src/fs/mod.rs
//!
//! # File System Utilities Module
//!
//! This module provides file system utilities for directory traversal, ignore logic, file listing, and binary detection.
//! It is used throughout the application for loading, filtering, and analyzing files and directories.
//!
//! ## Usage
//! Use these functions for recursive file listing, ignore pattern handling, and file type checks.
//!
//! ## Examples
//! ```rust
//! use crate::fs::{list_files, is_binary_file};
//! let files = list_files(&std::path::PathBuf::from("./src"));
//! assert!(!files.is_empty());
//! assert!(!is_binary_file(std::path::Path::new("main.rs")));
//! ```

use crate::models::IgnoreConfig;
use crate::tui::state::AppState;
use ignore::{gitignore::GitignoreBuilder, Match};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tokio::fs as tokio_fs;
use tokio_stream::StreamExt;

/// Prompts the user to confirm overwriting a file if it exists.
///
/// # Arguments
/// * `file_path` - The path to the file to check.
///
/// # Returns
/// * `io::Result<bool>` - Ok(true) if overwrite is confirmed or file does not exist, Ok(false) otherwise.
///
/// # Examples
/// ```rust
/// // Interactive: cannot test in doctest
/// ```
pub fn confirm_overwrite(file_path: &str) -> io::Result<bool> {
    if Path::new(file_path).exists() {
        println!("File '{}' already exists. Overwrite? (y/n): ", file_path);
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        return Ok(input.trim().eq_ignore_ascii_case("y"));
    }
    Ok(true)
}

/// Recursively lists all files under the given path, applying default ignore rules.
///
/// # Arguments
/// * `path` - The root directory to list files from.
///
/// # Returns
/// * `Vec<PathBuf>` - All files and directories found, excluding ignored ones.
///
/// # Examples
/// ```rust
/// let files = crate::fs::list_files(&std::path::PathBuf::from("./src"));
/// assert!(!files.is_empty());
/// ```
pub fn list_files(path: &PathBuf) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    list_files_inner(path, &mut result, &mut visited);
    result
}

fn list_files_inner(path: &PathBuf, result: &mut Vec<PathBuf>, visited: &mut HashSet<PathBuf>) {
    let canonical = match try_canonicalize(path) {
        Some(c) => c,
        None => return, // Permission denied or error, skip
    };
    if !visited.insert(canonical.clone()) {
        // Symlink loop detected, skip
        return;
    }
    let entries = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            return;
        }
    };
    for entry in entries.flatten() {
        let entry_path = entry.path();
        result.push(entry_path.clone());
        if entry_path.is_dir() {
            list_files_inner(&entry_path, result, visited);
        }
    }
}

// Helper for canonical path tracking in sync code
fn try_canonicalize(path: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(path).ok()
}

/// Recursively adds items to a list, respecting expanded folders and ignore rules.
///
/// # Arguments
/// * `items` - The list to populate.
/// * `root` - The root directory to start from.
/// * `expanded_folders` - Set of expanded folders to traverse.
/// * `ignore_config` - Ignore configuration.
/// * `base_dir` - The base directory for relative ignore checks.
///
/// # Returns
/// * `io::Result<()>` - Ok on success, or error if traversal fails.
///
/// # Examples
/// ```rust
/// // Used internally by TUI file loading logic.
/// ```
pub fn add_items_recursively(
    items: &mut Vec<PathBuf>,
    root: &PathBuf,
    expanded_folders: &HashSet<PathBuf>,
    ignore_config: &IgnoreConfig,
    base_dir: &PathBuf,
) -> io::Result<()> {
    let mut visited = HashSet::new();
    add_items_recursively_inner(
        items,
        root,
        expanded_folders,
        ignore_config,
        base_dir,
        &mut visited,
    )
}

fn add_items_recursively_inner(
    items: &mut Vec<PathBuf>,
    root: &PathBuf,
    expanded_folders: &HashSet<PathBuf>,
    ignore_config: &IgnoreConfig,
    base_dir: &PathBuf,
    visited: &mut HashSet<PathBuf>,
) -> io::Result<()> {
    if items.is_empty() && root == base_dir {
        if let Some(parent) = root.parent() {
            if !parent.as_os_str().is_empty() {
                items.push(root.join(".."));
            }
        }
    }
    let canonical = match try_canonicalize(root) {
        Some(c) => c,
        None => return Ok(()), // Permission denied or error, skip
    };
    if !visited.insert(canonical.clone()) {
        // Symlink loop detected, skip
        return Ok(());
    }
    let entries = match fs::read_dir(root) {
        Ok(rd) => rd,
        Err(e) => {
            return Ok(());
        }
    };
    let mut entry_vec: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| !is_path_ignored_iterative(p, base_dir, ignore_config))
        .collect();
    entry_vec.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });
    for entry in entry_vec {
        items.push(entry.clone());
        if entry.is_dir() && expanded_folders.contains(&entry) {
            add_items_recursively_inner(
                items,
                &entry,
                expanded_folders,
                ignore_config,
                base_dir,
                visited,
            )?;
        }
    }
    Ok(())
}

/// Determines if a path should be ignored based on ignore configuration and .gitignore rules.
///
/// # Arguments
/// * `path` - The path to check.
/// * `base_dir` - The base directory for relative ignore checks.
/// * `ignore_config` - Ignore configuration.
///
/// # Returns
/// * `bool` - True if the path should be ignored, false otherwise.
///
/// # Examples
/// ```rust
/// // Used internally by file loading and filtering logic.
/// ```
pub fn is_path_ignored_iterative(
    path: &PathBuf,
    base_dir: &PathBuf,
    ignore_config: &IgnoreConfig,
) -> bool {
    if !ignore_config.use_default_ignores && !ignore_config.use_gitignore {
        return false;
    }
    if ignore_config.use_default_ignores {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if crate::models::constants::DEFAULT_IGNORED_DIRS.contains(&name) {
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

/// Collects all subdirectories under the given base directory, applying ignore rules.
///
/// # Arguments
/// * `base_dir` - The root directory to start from.
/// * `ignore_config` - Ignore configuration.
///
/// # Returns
/// * `io::Result<HashSet<PathBuf>>` - Set of all subdirectories found.
///
/// # Examples
/// ```rust
/// // Used internally for recursive folder expansion.
/// ```
pub fn collect_all_subdirs(
    base_dir: &Path,
    ignore_config: &IgnoreConfig,
) -> io::Result<HashSet<PathBuf>> {
    let base_dir_buf = base_dir.to_path_buf();
    let mut dirs = HashSet::new();
    let mut stack = vec![base_dir_buf.clone()];
    let mut visited = HashSet::new();
    while let Some(current) = stack.pop() {
        let canonical = match try_canonicalize(&current) {
            Some(c) => c,
            None => continue, // Permission denied or error, skip
        };
        if !visited.insert(canonical.clone()) {
            continue;
        }
        if current.is_dir() {
            dirs.insert(current.clone());
            let entries = match fs::read_dir(&current) {
                Ok(rd) => rd,
                Err(e) => {
                    if e.kind() == io::ErrorKind::PermissionDenied {
                        continue;
                    } else {
                        continue;
                    }
                }
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && !is_path_ignored_iterative(&path, &base_dir_buf, ignore_config)
                {
                    stack.push(path);
                }
            }
        }
    }
    Ok(dirs)
}

/// Normalizes a file path to use forward slashes.
///
/// # Arguments
/// * `path` - The path string to normalize.
///
/// # Returns
/// * `String` - The normalized path.
///
/// # Examples
/// ```rust
/// assert_eq!(crate::fs::normalize_path("foo\\bar"), "foo/bar");
/// ```
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Counts the number of files and directories under a given path, respecting ignore rules and a selection limit.
///
/// # Arguments
/// * `path` - The path to start counting from.
/// * `base_dir` - The base directory for ignore checks.
/// * `ignore_config` - Ignore configuration.
/// * `selection_limit` - Maximum number of items to count before early exit.
///
/// # Returns
/// * `io::Result<usize>` - The number of items found, or early exit if limit exceeded.
///
/// # Examples
/// ```rust
/// // Used internally for selection limit enforcement.
/// ```
pub fn count_selection_items(
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
            if is_path_ignored_iterative(&current, base_dir, ignore_config) {
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

/// Returns true if the given path is a binary file, based on extension or name.
///
/// # Arguments
/// * `path` - The path to check.
///
/// # Returns
/// * `bool` - True if the file is binary, false otherwise.
///
/// # Examples
/// ```rust
/// assert!(!crate::fs::is_binary_file(std::path::Path::new("main.rs")));
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

    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let binary_files = ["index"];
        return binary_files.contains(&name);
    }
    false
}

/// Recursively searches for items matching a query up to a given depth, using a custom matcher.
///
/// # Arguments
/// * `app_state` - The application state (for ignore logic).
/// * `path` - The path to start searching from.
/// * `depth` - Current recursion depth.
/// * `max_depth` - Maximum recursion depth.
/// * `matcher` - Function to determine if a file/directory matches.
/// * `results` - Set to collect matching paths.
///
/// # Returns
/// * `bool` - True if any matches were found, false otherwise.
///
/// # Examples
/// ```rust
/// // Used internally for recursive search/filtering.
/// ```
pub fn recursive_search_helper_generic<F>(
    app_state: &AppState,
    path: &Path,
    depth: usize,
    max_depth: usize,
    matcher: &F,
    results: &mut HashSet<PathBuf>,
) -> bool
where
    F: Fn(&str) -> bool + ?Sized,
{
    if app_state.is_path_ignored(path) {
        return false;
    }
    let mut found = false;
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if matcher(name) {
            results.insert(path.to_path_buf());
            found = true;
        }
    }
    if path.is_dir() && depth < max_depth {
        if let Ok(entries) = fs::read_dir(path) {
            let mut children: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
            children.sort();
            for child in children {
                if recursive_search_helper_generic(
                    app_state,
                    &child,
                    depth + 1,
                    max_depth,
                    matcher,
                    results,
                ) {
                    found = true;
                }
            }
        }
        if found {
            results.insert(path.to_path_buf());
        }
    }
    found
}

/// Async version of list_files
pub async fn list_files_async(path: &PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    list_files_async_inner(path, &mut result, &mut visited).await?;
    Ok(result)
}

async fn list_files_async_inner(
    path: &PathBuf,
    result: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) -> io::Result<()> {
    let canonical = match tokio_fs::canonicalize(path).await {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == io::ErrorKind::PermissionDenied {
                // Permission denied, skip
                return Ok(());
            } else {
                return Err(e);
            }
        }
    };
    if !visited.insert(canonical.clone()) {
        // Symlink loop detected, skip
        return Ok(());
    }
    let mut read_dir = match tokio_fs::read_dir(path).await {
        Ok(rd) => rd,
        Err(e) => {
            if e.kind() == io::ErrorKind::PermissionDenied {
                // Permission denied, skip
                return Ok(());
            } else {
                return Err(e);
            }
        }
    };
    while let Some(entry) = read_dir.next_entry().await? {
        let entry_path = entry.path();
        result.push(entry_path.clone());
        if entry_path.is_dir() {
            Box::pin(list_files_async_inner(&entry_path, result, visited)).await?;
        }
    }
    Ok(())
}

/// Async version of add_items_recursively
pub async fn add_items_recursively_async(
    items: &mut Vec<PathBuf>,
    root: &PathBuf,
    expanded_folders: &HashSet<PathBuf>,
    ignore_config: &IgnoreConfig,
    base_dir: &PathBuf,
) -> io::Result<()> {
    let mut visited = HashSet::new();
    add_items_recursively_async_inner(
        items,
        root,
        expanded_folders,
        ignore_config,
        base_dir,
        &mut visited,
    )
    .await
}

async fn add_items_recursively_async_inner(
    items: &mut Vec<PathBuf>,
    root: &PathBuf,
    expanded_folders: &HashSet<PathBuf>,
    ignore_config: &IgnoreConfig,
    base_dir: &PathBuf,
    visited: &mut HashSet<PathBuf>,
) -> io::Result<()> {
    if items.is_empty() && root == base_dir {
        if let Some(parent) = root.parent() {
            if !parent.as_os_str().is_empty() {
                items.push(root.join(".."));
            }
        }
    }
    let canonical = match tokio_fs::canonicalize(root).await {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == io::ErrorKind::PermissionDenied {
                return Ok(());
            } else {
                return Err(e);
            }
        }
    };
    if !visited.insert(canonical.clone()) {
        // Symlink loop detected, skip
        return Ok(());
    }
    let mut read_dir = match tokio_fs::read_dir(root).await {
        Ok(rd) => rd,
        Err(e) => {
            if e.kind() == io::ErrorKind::PermissionDenied {
                return Ok(());
            } else {
                return Err(e);
            }
        }
    };
    let mut entries: Vec<PathBuf> = Vec::new();
    while let Some(entry) = read_dir.next_entry().await? {
        let entry_path = entry.path();
        if !is_path_ignored_iterative(&entry_path, base_dir, ignore_config) {
            entries.push(entry_path);
        }
    }
    entries.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });
    for entry in entries {
        items.push(entry.clone());
        if entry.is_dir() && expanded_folders.contains(&entry) {
            Box::pin(add_items_recursively_async_inner(
                items,
                &entry,
                expanded_folders,
                ignore_config,
                base_dir,
                visited,
            ))
            .await?;
        }
    }
    Ok(())
}

/// Async version of collect_all_subdirs
pub async fn collect_all_subdirs_async(
    base_dir: &Path,
    ignore_config: &IgnoreConfig,
) -> io::Result<HashSet<PathBuf>> {
    let base_dir_buf = base_dir.to_path_buf();
    let mut dirs = HashSet::new();
    let mut stack = vec![base_dir_buf.clone()];
    let mut visited = HashSet::new();
    while let Some(current) = stack.pop() {
        let canonical = match tokio_fs::canonicalize(&current).await {
            Ok(c) => c,
            Err(e) => {
                if e.kind() == io::ErrorKind::PermissionDenied {
                    continue;
                } else {
                    return Err(e);
                }
            }
        };
        if !visited.insert(canonical.clone()) {
            continue;
        }
        if current.is_dir() {
            dirs.insert(current.clone());
            let mut read_dir = match tokio_fs::read_dir(&current).await {
                Ok(rd) => rd,
                Err(e) => {
                    if e.kind() == io::ErrorKind::PermissionDenied {
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            };
            while let Some(entry) = read_dir.next_entry().await? {
                let path = entry.path();
                if path.is_dir() && !is_path_ignored_iterative(&path, &base_dir_buf, ignore_config)
                {
                    stack.push(path);
                }
            }
        }
    }
    Ok(dirs)
}

// TODO: Add more robust binary file detection (magic numbers, content sniffing).
// TODO: Add error handling for permission denied and symlink loops.
