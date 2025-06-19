// src/fs/mod.rs
//!
//! # File System Utilities
//!
//! This module provides robust file system utilities for directory traversal, ignore logic, file listing, binary detection, and path normalization.
//! It is used throughout the application for loading, filtering, and analyzing files and directories, and for enforcing ignore rules and selection limits.
//!
//! ## Organization
//! - Ignore logic: `.gitignore`, default ignores, and custom patterns
//! - File and directory listing (sync and async)
//! - Binary file detection (extension and magic number)
//! - Path normalization and utility helpers
//!
//! ## Example
//! ```rust
//! use aibundle_modular::fs::{list_files, is_binary_file, normalize_path};
//! let files = list_files(std::path::Path::new("./src"));
//! assert!(!files.is_empty());
//! assert!(!is_binary_file(std::path::Path::new("main.rs")));
//! assert_eq!(normalize_path("foo\\bar"), "foo/bar");
//! ```
//!
//! # Doc Aliases
//! - "filesystem"
//! - "ignore"
//! - "binary-detection"
//! - "directory-traversal"
//!
#![doc(alias = "filesystem")]
#![doc(alias = "ignore")]
#![doc(alias = "binary-detection")]
#![doc(alias = "directory-traversal")]

use crate::models::{constants, IgnoreConfig};
use crate::tui::state::AppState;
use ignore::{gitignore::GitignoreBuilder, Match};
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::fs as tokio_fs;

// Cache for compiled ignore patterns to reduce redundant pattern matching
lazy_static! {
    static ref IGNORE_PATTERN_CACHE: Arc<Mutex<HashMap<String, regex::Regex>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Binary file signatures for magic number detection
const BINARY_SIGNATURES: &[(&[u8], &str)] = &[
    // ELF executables
    (b"\x7fELF", "ELF"),
    // PE executables (Windows)
    (b"MZ", "PE"),
    // Mach-O executables (macOS)
    (b"\xfe\xed\xfa\xce", "Mach-O 32-bit"),
    (b"\xfe\xed\xfa\xcf", "Mach-O 64-bit"),
    (b"\xce\xfa\xed\xfe", "Mach-O 32-bit (reverse)"),
    (b"\xcf\xfa\xed\xfe", "Mach-O 64-bit (reverse)"),
    // Common image formats
    (b"\x89PNG\r\n\x1a\n", "PNG"),
    (b"\xff\xd8\xff", "JPEG"),
    (b"GIF87a", "GIF87a"),
    (b"GIF89a", "GIF89a"),
    (b"BM", "BMP"),
    // Archive formats
    (b"PK\x03\x04", "ZIP"),
    (b"PK\x05\x06", "ZIP (empty)"),
    (b"PK\x07\x08", "ZIP (spanned)"),
    (b"\x1f\x8b\x08", "GZIP"),
    (b"7z\xbc\xaf\x27\x1c", "7-Zip"),
    (b"Rar!\x1a\x07\x00", "RAR v1.5+"),
    (b"Rar!\x1a\x07\x01\x00", "RAR v5.0+"),
    // PDF
    (b"%PDF", "PDF"),
    // Class files
    (b"\xca\xfe\xba\xbe", "Java Class"),
    // SQLite
    (b"SQLite format 3\x00", "SQLite3"),
];

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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn list_files(path: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut stack: Vec<PathBuf> = vec![path.to_path_buf()];

    while let Some(current_path) = stack.pop() {
        let canonical = match try_canonicalize(&current_path) {
            Ok(c) => c,
            Err(_e) => continue, // Permission denied or error, skip
        };

        if !visited.insert(canonical.clone()) {
            // Symlink loop detected, skip
            continue;
        }

        let entries = match fs::read_dir(&current_path) {
            Ok(rd) => rd,
            Err(_e) => {
                continue;
            }
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // Check for permission denied before pushing to the stack
                match fs::metadata(&entry_path) {
                    Ok(_) => {
                        stack.push(entry_path.clone());
                    }
                    Err(_e) => {
                        // eprintln!(
                        //     "Permission denied for directory: {:?} - {:?}",
                        //     entry_path, e
                        // );
                        continue;
                    }
                }
            }
            result.push(entry_path.clone());
        }
    }
    result
}

// Helper for canonical path tracking in sync code
fn try_canonicalize(path: &Path) -> io::Result<PathBuf> {
    std::fs::canonicalize(path)
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
    if !ignore_config.use_default_ignores && !ignore_config.use_gitignore && ignore_config.extra_ignore_patterns.is_empty() {
        return false;
    }

    // Check default ignored directory names
    if ignore_config.use_default_ignores {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if constants::DEFAULT_IGNORED_DIRS.contains(&name) {
                return true;
            }
        }
    }

    // Check extra ignore patterns from config
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if ignore_config
            .extra_ignore_patterns
            .contains(&name.to_string())
        {
            return true;
        }
    }

    // Check .gitignore files
    if ignore_config.use_gitignore {
        let mut builder = GitignoreBuilder::new(base_dir);
        let mut current_dir_for_gitignore = path.parent().unwrap_or(path).to_path_buf();
        let mut gitignore_path_found = false;

        // Traverse upwards from the path itself (or its parent) to find .gitignore files up to base_dir
        // or the root of the filesystem if base_dir is part of a deeper git repo structure.
        loop {
            let gitignore_file = current_dir_for_gitignore.join(".gitignore");
            if gitignore_file.exists() && builder.add(gitignore_file).is_some() {
                gitignore_path_found = true;
            }
            if current_dir_for_gitignore == *base_dir || current_dir_for_gitignore.parent().is_none() {
                break;
            }
            if !current_dir_for_gitignore.pop() {
                break;
            }
        }

        // If no .gitignore files were explicitly added from the path's hierarchy up to base_dir,
        // try adding one from the base_dir itself, as AppState does.
        if !gitignore_path_found {
            let base_gitignore = base_dir.join(".gitignore");
            if base_gitignore.exists() {
                builder.add(base_gitignore);
            }
        }

        // Add .gitignore from initial_dir (usually project root) as a fallback or primary source.
        // This logic mirrors AppState more closely.
        // Consider if `base_dir` for `collect_folder_descendants` should always be `app_state.current_dir`
        // or `app_state.initial_dir` if .gitignores are typically at project root.
        // For now, using `base_dir` as passed.

        if let Ok(gitignore_matcher) = builder.build() {
            let is_dir = path.is_dir();
            if let Match::Ignore(_) = gitignore_matcher.matched_path_or_any_parents(path, is_dir) {
                return true;
            }
        }
    }
    false
}

/// Optimized version of is_path_ignored_iterative with cached pattern compilation.
/// This reduces redundant pattern matching operations by caching compiled regex patterns.
///
/// # Arguments
/// * `path` - The path to check.
/// * `base_dir` - The base directory for relative ignore checks.
/// * `ignore_config` - Ignore configuration.
///
/// # Returns
/// * `bool` - True if the path should be ignored, false otherwise.
pub fn is_path_ignored_iterative_cached(
    path: &PathBuf,
    base_dir: &PathBuf,
    ignore_config: &IgnoreConfig,
) -> bool {
    if !ignore_config.use_default_ignores && !ignore_config.use_gitignore {
        return false;
    }

    // Check default ignored directories with cached pattern matching
    if ignore_config.use_default_ignores {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if crate::models::constants::DEFAULT_IGNORED_DIRS.contains(&name) {
                return true;
            }
        }
    }

    // Check extra ignore patterns with cached regex compilation
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        // Create a fallback regex outside the loop to avoid repeated construction
        let fallback_regex = regex::Regex::new("^$").unwrap(); // Never matches if invalid

        for pattern in &ignore_config.extra_ignore_patterns {
            if let Ok(mut cache) = IGNORE_PATTERN_CACHE.lock() {
                let regex = cache.entry(pattern.clone()).or_insert_with(|| {
                    // Convert glob pattern to regex and cache it
                    let regex_pattern = pattern.replace("*", ".*").replace("?", ".");
                    regex::Regex::new(&format!("^{}$", regex_pattern))
                        .unwrap_or_else(|_| fallback_regex.clone())
                });

                if regex.is_match(name) {
                    return true;
                }
            }
        }
    }

    // GitIgnore checking (unchanged as it's already optimized by the ignore crate)
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
            Ok(c) => c,
            Err(_e) => continue, // Permission denied or error, skip
        };
        if !visited.insert(canonical.clone()) {
            continue;
        }
        if current.is_dir() {
            dirs.insert(current.clone());
            let entries = match fs::read_dir(&current) {
                Ok(rd) => rd,
                Err(_e) => {
                    continue;
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

/// Returns true if the given path is a binary file, using both extension-based and magic number detection.
///
/// This function implements a robust binary file detection mechanism that:
/// 1. First checks file extensions for known binary types (fast path)
/// 2. Then performs magic number/content sniffing for accurate detection
/// 3. Falls back to heuristic analysis of file content
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
/// assert!(crate::fs::is_binary_file(std::path::Path::new("binary.exe")));
/// ```
#[allow(dead_code)]
pub fn is_binary_file(path: &Path) -> bool {
    // First, check extension-based detection (fast path)
    if is_binary_by_extension(path) {
        return true;
    }

    // Then perform magic number detection (more accurate but slower)
    if let Ok(is_binary) = is_binary_by_content(path) {
        return is_binary;
    }

    // Fallback to extension-based detection if content reading fails
    false
}

/// Fast extension-based binary file detection
fn is_binary_by_extension(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let binary_extensions = [
            // Git internals
            "idx", "pack", "rev", "index", // Images
            "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp", "ico", "svg", // Audio
            "mp3", "wav", "ogg", "flac", "m4a", "aac", "wma", // Video
            "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", // Archives
            "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso", // Executables
            "exe", "dll", "so", "dylib", "bin", // Documents
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", // Compiled code
            "class", "pyc", "pyd", "pyo", "o", "obj", // Databases
            "db", "sqlite", "sqlite3", // Fonts
            "ttf", "otf", "woff", "woff2",
        ];
        if binary_extensions.contains(&ext.to_lowercase().as_str()) {
            return true;
        }
    }

    // Check specific filenames that are typically binary
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let binary_files = ["index", "COMMIT_EDITMSG"];
        return binary_files.contains(&name);
    }

    false
}

/// Content-based binary file detection using magic numbers and heuristics
fn is_binary_by_content(path: &Path) -> io::Result<bool> {
    // Don't try to read directories or non-existent files
    if !path.is_file() {
        return Ok(false);
    }

    let mut file = std::fs::File::open(path)?;
    let mut buffer = [0u8; 8192]; // Read first 8KB for analysis
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(false); // Empty files are not binary
    }

    let content = &buffer[..bytes_read];

    // Check for magic number signatures
    for (signature, _description) in BINARY_SIGNATURES {
        if content.len() >= signature.len() && content.starts_with(signature) {
            return Ok(true);
        }
    }

    // Heuristic: Check for null bytes and high ratio of non-printable characters
    let null_count = content.iter().filter(|&&b| b == 0).count();
    if null_count > 0 {
        return Ok(true); // Files with null bytes are typically binary
    }

    // Check ratio of non-printable characters
    let non_printable_count = content
        .iter()
        .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13) // Exclude tab, LF, CR
        .count();

    let non_printable_ratio = non_printable_count as f64 / content.len() as f64;

    // If more than 30% of characters are non-printable, consider it binary
    Ok(non_printable_ratio > 0.30)
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
#[allow(dead_code)]
pub async fn list_files_async(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let path_buf = path.to_path_buf();
    list_files_async_inner(&path_buf, &mut result, &mut visited).await?;
    Ok(result)
}

#[allow(dead_code)]
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

/// Async version of collect_all_subdirs
#[allow(dead_code)]
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

/// Recursively collects all non-ignored descendant files and directories of a given folder path.
/// Includes the directories themselves in the output set.
/// 
/// This function includes symlink loop detection to prevent infinite recursion.
///
/// # Arguments
/// * `folder_path` - The path to the folder to scan.
/// * `gitignore_base_dir` - The base directory for resolving .gitignore files (usually app_state.current_dir).
/// * `ignore_config` - The ignore configuration to use.
/// * `descendants_set` - A mutable HashSet to which the paths of non-ignored descendants will be added.
///
/// # Returns
/// * `io::Result<()>` - Ok if the operation completes, or an io::Error if directory reading fails.
pub fn collect_folder_descendants(
    folder_path: &Path,
    gitignore_base_dir: &PathBuf,
    ignore_config: &IgnoreConfig,
    descendants_set: &mut HashSet<PathBuf>,
) -> io::Result<()> {
    collect_folder_descendants_with_visited(
        folder_path,
        gitignore_base_dir,
        ignore_config,
        descendants_set,
        &mut HashSet::new(),
    )
}

/// Internal implementation of collect_folder_descendants with symlink loop detection.
fn collect_folder_descendants_with_visited(
    folder_path: &Path,
    gitignore_base_dir: &PathBuf,
    ignore_config: &IgnoreConfig,
    descendants_set: &mut HashSet<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) -> io::Result<()> {
    if !folder_path.is_dir() {
        return Ok(()); // Nothing to collect if it's not a directory
    }

    // Symlink loop detection
    let canonical = match try_canonicalize(folder_path) {
        Ok(c) => c,
        Err(_e) => return Ok(()), // Permission denied or error, skip
    };

    if !visited.insert(canonical.clone()) {
        // Symlink loop detected, skip this directory
        return Ok(());
    }

    match fs::read_dir(folder_path) {
        Ok(entries) => {
            for entry_result in entries {
                match entry_result {
                    Ok(entry) => {
                        let path = entry.path();

                        // Use the existing cached ignore checker
                        if is_path_ignored_iterative_cached(&path, gitignore_base_dir, ignore_config) {
                            continue;
                        }

                        // Add the item itself (file or directory)
                        descendants_set.insert(path.clone());

                        if path.is_dir() {
                            // Recursively collect descendants of this subdirectory
                            collect_folder_descendants_with_visited(&path, gitignore_base_dir, ignore_config, descendants_set, visited)?;
                        }
                    }
                    Err(_e) => {
                        // Log or handle specific errors if needed, for now, skip unreadable entries
                        // eprintln!("Skipping unreadable entry: {:?}, error: {}", folder_path.join("..."), e);
                        continue;
                    }
                }
            }
        }
        Err(e) => {
            // eprintln!("Failed to read directory {}: {}", folder_path.display(), e);
            return Err(e); // Propagate the error if the directory itself cannot be read
        }
    }
    Ok(())
}

// TODO: Add error handling for permission denied and symlink loops.

// Phase 1 File System Optimizations - COMPLETED
// ✅ 1. Robust binary file detection with magic numbers and content sniffing
// ✅ 3. Lazy sorting only when UI needs update with state tracking
// ✅ 4. Optimized ignore checks with cached pattern compilation
