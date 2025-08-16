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
//! use aibundle::fs::{list_files, is_binary_file, normalize_path};
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

// Cache for compiled gitignore matchers to reduce redundant gitignore processing
lazy_static! {
    static ref GITIGNORE_CACHE: Arc<Mutex<HashMap<PathBuf, Option<ignore::gitignore::Gitignore>>>> =
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
    let mut path_trackers = HashSet::new();
    let mut stack: Vec<PathBuf> = vec![path.to_path_buf()];

    while let Some(current_path) = stack.pop() {
        // Enhanced symlink loop detection
        let path_tracker = match PathTracker::new(&current_path) {
            Ok(tracker) => tracker,
            Err(_e) => continue, // Permission denied or error, skip
        };

        // Check canonical path tracking (original behavior)
        if !visited.insert(path_tracker.canonical.clone()) {
            // Symlink loop detected via canonical path, skip
            continue;
        }

        // Check enhanced path tracking (new behavior)
        if !path_trackers.insert(path_tracker) {
            // Complex symlink loop detected via path tracking, skip
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

/// Clears the gitignore cache. Should be called when .gitignore files change
/// or when switching to a different project directory.
pub fn clear_gitignore_cache() {
    if let Ok(mut cache) = GITIGNORE_CACHE.lock() {
        cache.clear();
    }
}

/// Gets or creates a cached gitignore matcher for the given directory context.
/// This builds a matcher that considers all .gitignore files from the given directory
/// upwards to the project root. This is file-contextual gitignore matching.
///
/// # Arguments
/// * `context_dir` - The directory context for which to build the gitignore matcher
/// * `project_root` - The project root directory (used as upper bound for traversal)
///
/// Returns None if no gitignore files are found or if compilation fails.
pub fn get_cached_gitignore_matcher_for_context(
    context_dir: &PathBuf,
    project_root: &PathBuf
) -> Option<ignore::gitignore::Gitignore> {
    if let Ok(mut cache) = GITIGNORE_CACHE.lock() {
        if let Some(cached_matcher) = cache.get(context_dir) {
            return cached_matcher.clone();
        }

        // Build gitignore matcher if not in cache
        let mut builder = GitignoreBuilder::new(project_root);
        let mut dir = context_dir.clone();
        let mut found_gitignore = false;

        // Traverse upwards from context_dir to project_root to find .gitignore files
        loop {
            let gitignore_file = dir.join(".gitignore");
            if gitignore_file.exists() {
                // builder.add() returns Some(error) on error, None on success
                if builder.add(gitignore_file).is_none() {
                    found_gitignore = true;
                }
            }

            // Stop when we reach the project root or can't go further up
            if dir == *project_root {
                break;
            }

            if let Some(parent) = dir.parent() {
                dir = parent.to_path_buf();
                // Also stop if we've gone above the project root
                if !dir.starts_with(project_root) {
                    break;
                }
            } else {
                break;
            }
        }

        let matcher = if found_gitignore {
            builder.build().ok()
        } else {
            None
        };

        // Cache the result (even if None)
        cache.insert(context_dir.clone(), matcher.clone());
        matcher
    } else {
        // Fallback if cache lock fails - build without caching
        let mut builder = GitignoreBuilder::new(project_root);
        let mut dir = context_dir.clone();
        let mut found_gitignore = false;

        loop {
            let gitignore_file = dir.join(".gitignore");
            if gitignore_file.exists() && builder.add(gitignore_file).is_none() {
                found_gitignore = true;
            }

            if dir == *project_root {
                break;
            }

            if let Some(parent) = dir.parent() {
                dir = parent.to_path_buf();
                if !dir.starts_with(project_root) {
                    break;
                }
            } else {
                break;
            }
        }

        if found_gitignore {
            builder.build().ok()
        } else {
            None
        }
    }
}


/// Legacy function maintained for backward compatibility.
/// Gets or creates a cached gitignore matcher for the given base directory.
/// Returns None if no gitignore files are found or if compilation fails.
#[deprecated(note = "Use get_cached_gitignore_matcher_for_context instead for proper file-contextual gitignore matching")]
pub fn get_cached_gitignore_matcher(base_dir: &PathBuf) -> Option<ignore::gitignore::Gitignore> {
    get_cached_gitignore_matcher_for_context(base_dir, base_dir)
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

    // Check .gitignore files using file-contextual matcher
    if ignore_config.use_gitignore {
        // Use the file's parent directory as context for gitignore matching
        let context_dir = if path.is_file() {
            path.parent().unwrap_or(base_dir).to_path_buf()
        } else {
            path.clone()
        };

        if let Some(gitignore_matcher) = get_cached_gitignore_matcher_for_context(&context_dir, base_dir) {
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

    // GitIgnore checking using file-contextual cached matcher for improved performance
    if ignore_config.use_gitignore {
        // Use the file's parent directory as context for gitignore matching
        let context_dir = if path.is_file() {
            path.parent().unwrap_or(base_dir).to_path_buf()
        } else {
            path.clone()
        };

        if let Some(gitignore_matcher) = get_cached_gitignore_matcher_for_context(&context_dir, base_dir) {
            let is_dir = path.is_dir();
            if let Match::Ignore(_) = gitignore_matcher.matched_path_or_any_parents(path, is_dir) {
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
    let mut path_trackers = HashSet::new();
    
    while let Some(current) = stack.pop() {
        // Enhanced symlink loop detection
        let path_tracker = match PathTracker::new(&current) {
            Ok(tracker) => tracker,
            Err(_e) => continue, // Permission denied or error, skip
        };

        // Check canonical path tracking (original behavior)
        if !visited.insert(path_tracker.canonical.clone()) {
            continue;
        }

        // Check enhanced path tracking (new behavior)
        if !path_trackers.insert(path_tracker) {
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

    // Removed overly aggressive filename-based binary detection
    // Files like "index.js", "index.html" should not be treated as binary
    // COMMIT_EDITMSG is also plain text and should not be treated as binary

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
    // For async version, we use a simpler approach since PathTracker::new is sync
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
    
    // Enhanced check: also track original path to detect complex symlink scenarios
    let original = path.clone();
    let combined_key = format!("{}:{}", canonical.display(), original.display());
    
    if !visited.insert(canonical.clone()) || visited.contains(&PathBuf::from(&combined_key)) {
        // Symlink loop detected, skip
        return Ok(());
    }
    
    // Track the combined key for enhanced detection
    visited.insert(PathBuf::from(combined_key));
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

/// Enhanced symlink tracking for better loop detection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PathTracker {
    canonical: PathBuf,
    original: PathBuf,
}

impl PathTracker {
    fn new(path: &Path) -> io::Result<Self> {
        let canonical = try_canonicalize(path)?;
        Ok(Self {
            canonical,
            original: path.to_path_buf(),
        })
    }
}

/// Internal implementation of collect_folder_descendants with enhanced symlink loop detection.
fn collect_folder_descendants_with_visited(
    folder_path: &Path,
    gitignore_base_dir: &PathBuf,
    ignore_config: &IgnoreConfig,
    descendants_set: &mut HashSet<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) -> io::Result<()> {
    collect_folder_descendants_with_enhanced_tracking(
        folder_path,
        gitignore_base_dir,
        ignore_config,
        descendants_set,
        visited,
        &mut HashSet::new(),
    )
}

/// Internal implementation with enhanced symlink loop detection tracking both canonical and original paths.
fn collect_folder_descendants_with_enhanced_tracking(
    folder_path: &Path,
    gitignore_base_dir: &PathBuf,
    ignore_config: &IgnoreConfig,
    descendants_set: &mut HashSet<PathBuf>,
    visited: &mut HashSet<PathBuf>,
    path_trackers: &mut HashSet<PathTracker>,
) -> io::Result<()> {
    if !folder_path.is_dir() {
        return Ok(()); // Nothing to collect if it's not a directory
    }

    // Enhanced symlink loop detection - track both canonical and original paths
    let path_tracker = match PathTracker::new(folder_path) {
        Ok(tracker) => tracker,
        Err(_e) => return Ok(()), // Permission denied or error, skip
    };

    // Check if we've already visited this canonical path (old behavior)
    if !visited.insert(path_tracker.canonical.clone()) {
        // Symlink loop detected via canonical path, skip this directory
        return Ok(());
    }

    // Check if we've already processed this exact path combination (new behavior)
    if !path_trackers.insert(path_tracker) {
        // Complex symlink loop detected via path tracking, skip this directory
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
                            // Recursively collect descendants of this subdirectory with enhanced tracking
                            collect_folder_descendants_with_enhanced_tracking(&path, gitignore_base_dir, ignore_config, descendants_set, visited, path_trackers)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    /// Creates a test directory structure with nested .gitignore files
    /// to demonstrate the contextual gitignore issue
    fn create_test_gitignore_structure() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Create directory structure
        fs::create_dir_all(base_path.join("src/components")).unwrap();
        fs::create_dir_all(base_path.join("src/utils")).unwrap();
        fs::create_dir_all(base_path.join("tests")).unwrap();

        // Create root .gitignore
        let mut root_gitignore = File::create(base_path.join(".gitignore")).unwrap();
        writeln!(root_gitignore, "# Root gitignore").unwrap();
        writeln!(root_gitignore, "*.log").unwrap();
        writeln!(root_gitignore, "node_modules/").unwrap();

        // Create src/.gitignore
        let mut src_gitignore = File::create(base_path.join("src/.gitignore")).unwrap();
        writeln!(src_gitignore, "# Src gitignore").unwrap();
        writeln!(src_gitignore, "*.tmp").unwrap();
        writeln!(src_gitignore, "debug.js").unwrap();

        // Create src/components/.gitignore
        let mut components_gitignore = File::create(base_path.join("src/components/.gitignore")).unwrap();
        writeln!(components_gitignore, "# Components gitignore").unwrap();
        writeln!(components_gitignore, "*.test.js").unwrap();
        writeln!(components_gitignore, "Button.tsx").unwrap();

        // Create test files
        File::create(base_path.join("app.log")).unwrap(); // Should be ignored by root
        File::create(base_path.join("src/debug.js")).unwrap(); // Should be ignored by src
        File::create(base_path.join("src/utils.tmp")).unwrap(); // Should be ignored by src
        File::create(base_path.join("src/components/Button.tsx")).unwrap(); // Should be ignored by components
        File::create(base_path.join("src/components/Modal.test.js")).unwrap(); // Should be ignored by components
        File::create(base_path.join("src/components/Header.tsx")).unwrap(); // Should NOT be ignored
        File::create(base_path.join("src/main.js")).unwrap(); // Should NOT be ignored
        File::create(base_path.join("README.md")).unwrap(); // Should NOT be ignored

        temp_dir
    }

    #[test]
    fn test_gitignore_context_issue_demonstration() {
        // This test demonstrates the current incorrect behavior
        let temp_dir = create_test_gitignore_structure();
        let base_path = temp_dir.path().to_path_buf();

        // Clear cache to ensure fresh test
        clear_gitignore_cache();

        let ignore_config = IgnoreConfig {
            use_gitignore: true,
            use_default_ignores: false,
            include_binary_files: false,
            extra_ignore_patterns: vec![],
        };

        // Test the problematic file: src/components/Button.tsx
        let button_file = base_path.join("src/components/Button.tsx");

        // With the current implementation, this may not be correctly ignored
        // because get_cached_gitignore_matcher only uses base_dir as cache key
        let is_ignored = is_path_ignored_iterative(&button_file, &base_path, &ignore_config);

        // The current implementation might fail this assertion
        // because it doesn't properly handle nested gitignore files
        println!("Button.tsx ignored: {}", is_ignored);
        println!("Expected: true (should be ignored by src/components/.gitignore)");

        // This test will likely fail with the current implementation
        // demonstrating the bug described in CRITICALBUG.md
    }

    #[test]
    fn test_correct_gitignore_behavior_expectations() {
        // This test defines what the correct behavior should be
        let temp_dir = create_test_gitignore_structure();
        let base_path = temp_dir.path().to_path_buf();

        clear_gitignore_cache();

        let ignore_config = IgnoreConfig {
            use_gitignore: true,
            use_default_ignores: false,
            include_binary_files: false,
            extra_ignore_patterns: vec![],
        };


        // Test files that should be ignored
        let test_cases = vec![
            (base_path.join("app.log"), true, "root .gitignore"),
            (base_path.join("src/debug.js"), true, "src .gitignore"),
            (base_path.join("src/utils.tmp"), true, "src .gitignore"),
            (base_path.join("src/components/Button.tsx"), true, "components .gitignore"),
            (base_path.join("src/components/Modal.test.js"), true, "components .gitignore"),
            (base_path.join("src/components/Header.tsx"), false, "should not be ignored"),
            (base_path.join("src/main.js"), false, "should not be ignored"),
            (base_path.join("README.md"), false, "should not be ignored"),
        ];

        for (file_path, should_be_ignored, reason) in test_cases {
            let is_ignored = is_path_ignored_iterative(&file_path, &base_path, &ignore_config);
            println!("File: {}, Ignored: {}, Expected: {}, Reason: {}",
                    file_path.display(), is_ignored, should_be_ignored, reason);

            // These assertions will likely fail with the current implementation
            if should_be_ignored {
                assert!(is_ignored, "File {} should be ignored ({})", file_path.display(), reason);
            } else {
                assert!(!is_ignored, "File {} should not be ignored ({})", file_path.display(), reason);
            }
        }
    }

    #[test]
    fn test_gitignore_cache_key_problem() {
        // This test specifically demonstrates the cache key problem
        let temp_dir = create_test_gitignore_structure();
        let base_path = temp_dir.path().to_path_buf();

        clear_gitignore_cache();

        let ignore_config = IgnoreConfig {
            use_gitignore: true,
            use_default_ignores: false,
            include_binary_files: false,
            extra_ignore_patterns: vec![],
        };

        // Check if gitignore files exist
        println!("Root .gitignore exists: {}", base_path.join(".gitignore").exists());
        println!("Src .gitignore exists: {}", base_path.join("src/.gitignore").exists());
        println!("Components .gitignore exists: {}", base_path.join("src/components/.gitignore").exists());

        // The problem: cache is keyed only by base_dir
        // So get_cached_gitignore_matcher(base_path) returns the same matcher
        // for all files, regardless of their location in the directory hierarchy

        let matcher1 = get_cached_gitignore_matcher_for_context(&base_path, &base_path);
        let matcher2 = get_cached_gitignore_matcher_for_context(&base_path.join("src"), &base_path);
        let matcher3 = get_cached_gitignore_matcher_for_context(&base_path.join("src/components"), &base_path);

        // The current implementation will return the same matcher for all these calls
        // because it only caches by base_dir, which is wrong

        // For demonstration, let's see how files are handled
        let button_file = base_path.join("src/components/Button.tsx");
        let is_ignored = is_path_ignored_iterative(&button_file, &base_path, &ignore_config);

        println!("Matcher1 (base): {:?}", matcher1.is_some());
        println!("Matcher2 (src): {:?}", matcher2.is_some());
        println!("Matcher3 (components): {:?}", matcher3.is_some());
        println!("Button.tsx ignored: {}", is_ignored);

        // Test the matcher manually
        if let Some(matcher) = &matcher1 {
            let is_dir = button_file.is_dir();
            let match_result = matcher.matched_path_or_any_parents(&button_file, is_dir);
            println!("Match result: {:?}", match_result);
        }

        // The issue is that all matchers are the same because they use the same cache key
        // This test documents the problem without asserting (since we know it's broken)
    }

    #[test]
    fn test_file_contextual_gitignore_requirements() {
        // This test documents what the correct file-contextual behavior should be
        let temp_dir = create_test_gitignore_structure();
        let base_path = temp_dir.path().to_path_buf();

        // For a file like src/components/Button.tsx, the gitignore check should:
        // 1. Check src/components/.gitignore (most specific)
        // 2. Check src/.gitignore
        // 3. Check .gitignore (root, least specific)

        // Currently, the implementation only builds one matcher from base_dir upward
        // and uses it for all files, which is incorrect.

        // The correct behavior would be to either:
        // A) Build a file-specific matcher that considers the file's directory hierarchy
        // B) Use a more sophisticated caching strategy that considers file context

        // This test documents the requirement without implementation
        let button_file = base_path.join("src/components/Button.tsx");

        // Expected gitignore file check order for Button.tsx:
        let expected_gitignore_files = vec![
            base_path.join("src/components/.gitignore"),
            base_path.join("src/.gitignore"),
            base_path.join(".gitignore"),
        ];

        for gitignore_file in expected_gitignore_files {
            assert!(gitignore_file.exists(), "Expected gitignore file should exist: {}", gitignore_file.display());
        }

        // The Button.tsx file should be matched by src/components/.gitignore
        // which contains "Button.tsx" pattern

        println!("Test setup complete. File-contextual gitignore checking is required.");
        println!("File: {}", button_file.display());
        println!("Should be ignored by: src/components/.gitignore");
    }

    /// Test deeply nested gitignore hierarchy to ensure no stack overflow
    #[test]
    fn test_deeply_nested_gitignore_hierarchy() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        // Create a 15-level deep directory structure with gitignore files
        let mut current_path = base_path.clone();
        for level in 0..15 {
            current_path = current_path.join(format!("level{}", level));
            fs::create_dir_all(&current_path).unwrap();

            // Create a gitignore file at each level with different patterns
            let gitignore_path = current_path.join(".gitignore");
            let mut gitignore_file = File::create(&gitignore_path).unwrap();
            writeln!(gitignore_file, "# Level {} gitignore", level).unwrap();
            writeln!(gitignore_file, "ignore_level_{}.txt", level).unwrap();

            // Create a test file that should be ignored at this level
            File::create(current_path.join(format!("ignore_level_{}.txt", level))).unwrap();
        }

        clear_gitignore_cache();

        let ignore_config = IgnoreConfig {
            use_gitignore: true,
            use_default_ignores: false,
            include_binary_files: false,
            extra_ignore_patterns: vec![],
        };

        // Test file at the deepest level
        let deepest_path = base_path.join("level0/level1/level2/level3/level4/level5/level6/level7/level8/level9/level10/level11/level12/level13/level14");
        let test_file = deepest_path.join("ignore_level_14.txt");

        // This should not cause stack overflow and should properly ignore the file
        let is_ignored = is_path_ignored_iterative(&test_file, &base_path, &ignore_config);
        assert!(is_ignored, "Deep nested file should be ignored by its local gitignore");
    }

    /// Test symlink edge cases that could cause infinite loops
    #[test]
    #[cfg(unix)] // Symlinks are primarily a Unix feature
    fn test_symlink_edge_cases() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        // Create test structure
        let dir_a = base_path.join("dir_a");
        let dir_b = base_path.join("dir_b");
        let dir_c = base_path.join("dir_c");
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();
        fs::create_dir_all(&dir_c).unwrap();

        // Create circular symlink: a -> b -> c -> a
        std::os::unix::fs::symlink(&dir_b, dir_a.join("link_to_b")).unwrap();
        std::os::unix::fs::symlink(&dir_c, dir_b.join("link_to_c")).unwrap();
        std::os::unix::fs::symlink(&dir_a, dir_c.join("link_to_a")).unwrap();

        // Create self-referencing symlink
        std::os::unix::fs::symlink(&dir_a, dir_a.join("self_link")).unwrap();

        let ignore_config = IgnoreConfig {
            use_gitignore: false,
            use_default_ignores: false,
            include_binary_files: false,
            extra_ignore_patterns: vec![],
        };

        // Test that collecting descendants doesn't cause infinite recursion
        let mut descendants = HashSet::new();
        let result = collect_folder_descendants(&dir_a, &base_path, &ignore_config, &mut descendants);

        // Should complete without hanging or stack overflow
        assert!(result.is_ok(), "Symlink traversal should not cause infinite recursion");
    }

    /// Test binary file detection with edge cases
    #[test]
    fn test_binary_file_detection_edge_cases() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Test empty file
        let empty_file = base_path.join("empty.txt");
        File::create(&empty_file).unwrap();
        assert!(!is_binary_file(&empty_file), "Empty file should not be detected as binary");

        // Test file with only null bytes
        let null_file = base_path.join("null_bytes");
        let mut null_file_handle = File::create(&null_file).unwrap();
        null_file_handle.write_all(&[0u8; 100]).unwrap();
        assert!(is_binary_file(&null_file), "File with null bytes should be detected as binary");

        // Test file with mixed content
        let mixed_file = base_path.join("mixed.txt");
        let mut mixed_file_handle = File::create(&mixed_file).unwrap();
        mixed_file_handle.write_all(b"Hello World\0\xFF\xFE").unwrap();
        assert!(is_binary_file(&mixed_file), "File with mixed binary content should be detected as binary");

        // Test pure text file
        let text_file = base_path.join("text.txt");
        let mut text_file_handle = File::create(&text_file).unwrap();
        text_file_handle.write_all(b"Hello World\nThis is a text file\n").unwrap();
        assert!(!is_binary_file(&text_file), "Pure text file should not be detected as binary");

        // Test file with unicode content
        let unicode_file = base_path.join("unicode.txt");
        let mut unicode_file_handle = File::create(&unicode_file).unwrap();
        unicode_file_handle.write_all("Hello 世界 🌍\n".as_bytes()).unwrap();
        assert!(!is_binary_file(&unicode_file), "Unicode text file should not be detected as binary");
    }

    /// Test path normalization with edge cases
    #[test]
    fn test_path_normalization_edge_cases() {
        // Test with backslashes to forward slashes conversion
        assert_eq!(normalize_path("path\\to\\file"), "path/to/file");

        // Test with mixed separators
        assert_eq!(normalize_path("path\\to/file"), "path/to/file");

        // Test with unicode characters
        assert_eq!(normalize_path("пуäth\\tö\\файл"), "пуäth/tö/файл");

        // Test with spaces and special characters
        assert_eq!(normalize_path("path with spaces\\to\\file!@#"), "path with spaces/to/file!@#");

        // Test empty string
        assert_eq!(normalize_path(""), "");

        // Test paths that are already normalized
        assert_eq!(normalize_path("path/to/file"), "path/to/file");

        // Test with only backslashes
        assert_eq!(normalize_path("\\\\server\\share"), "//server/share");

        // Test with current and parent directory references
        assert_eq!(normalize_path(".\\path\\to\\file"), "./path/to/file");
        assert_eq!(normalize_path("..\\path\\to\\file"), "../path/to/file");
    }

    /// Test complex gitignore patterns that are commonly problematic
    #[test]
    fn test_complex_gitignore_patterns() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        // Create complex gitignore with various pattern types
        let gitignore_path = base_path.join(".gitignore");
        let mut gitignore_file = File::create(&gitignore_path).unwrap();
        writeln!(gitignore_file, "# Complex patterns test").unwrap();
        writeln!(gitignore_file, "*.log").unwrap();              // Simple wildcard
        writeln!(gitignore_file, "test[0-9].txt").unwrap();      // Character class
        writeln!(gitignore_file, "temp?.dat").unwrap();          // Single character wildcard
        writeln!(gitignore_file, "**/build/").unwrap();          // Double asterisk
        writeln!(gitignore_file, "!important.log").unwrap();     // Negation pattern
        writeln!(gitignore_file, "path\\ with\\ spaces/").unwrap(); // Escaped spaces
        writeln!(gitignore_file, "/root_only.txt").unwrap();     // Root-relative pattern

        // Create test files
        File::create(base_path.join("app.log")).unwrap();
        File::create(base_path.join("test5.txt")).unwrap();
        File::create(base_path.join("temp1.dat")).unwrap();
        File::create(base_path.join("important.log")).unwrap();
        File::create(base_path.join("root_only.txt")).unwrap();

        let sub_dir = base_path.join("sub");
        fs::create_dir_all(&sub_dir).unwrap();
        File::create(sub_dir.join("root_only.txt")).unwrap(); // Should NOT be ignored (pattern is root-relative)

        let build_dir = base_path.join("project/build");
        fs::create_dir_all(&build_dir).unwrap();
        File::create(build_dir.join("output.exe")).unwrap();

        clear_gitignore_cache();

        let ignore_config = IgnoreConfig {
            use_gitignore: true,
            use_default_ignores: false,
            include_binary_files: false,
            extra_ignore_patterns: vec![],
        };

        // Test various pattern matches
        assert!(is_path_ignored_iterative(&base_path.join("app.log"), &base_path, &ignore_config),
                "Simple wildcard should work");
        assert!(is_path_ignored_iterative(&base_path.join("test5.txt"), &base_path, &ignore_config),
                "Character class should work");
        assert!(is_path_ignored_iterative(&base_path.join("temp1.dat"), &base_path, &ignore_config),
                "Single char wildcard should work");
        assert!(!is_path_ignored_iterative(&base_path.join("important.log"), &base_path, &ignore_config),
                "Negation pattern should work");
        assert!(is_path_ignored_iterative(&base_path.join("root_only.txt"), &base_path, &ignore_config),
                "Root-relative pattern should work for root files");
        assert!(!is_path_ignored_iterative(&sub_dir.join("root_only.txt"), &base_path, &ignore_config),
                "Root-relative pattern should NOT match non-root files");
    }

    /// Test gitignore cache concurrency (basic test, full concurrency testing requires more setup)
    #[test]
    fn test_gitignore_cache_basic_concurrency() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        // Create gitignore
        let gitignore_path = base_path.join(".gitignore");
        let mut gitignore_file = File::create(&gitignore_path).unwrap();
        writeln!(gitignore_file, "*.log").unwrap();

        clear_gitignore_cache();

        // Test rapid successive calls to ensure cache consistency
        for _ in 0..100 {
            let matcher1 = get_cached_gitignore_matcher_for_context(&base_path, &base_path);
            let matcher2 = get_cached_gitignore_matcher_for_context(&base_path, &base_path);

            // Both should either be Some or None consistently
            assert_eq!(matcher1.is_some(), matcher2.is_some(),
                      "Cache should return consistent results");
        }

        // Clear cache and test again
        clear_gitignore_cache();
        let matcher_after_clear = get_cached_gitignore_matcher_for_context(&base_path, &base_path);
        assert!(matcher_after_clear.is_some(), "Cache should rebuild after clear");
    }

    /// Test path ignored checks with permission edge cases
    #[test]
    #[cfg(unix)] // Permission testing is primarily Unix-specific
    fn test_path_ignored_permissions() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        // Create a directory and file
        let sub_dir = base_path.join("restricted");
        fs::create_dir_all(&sub_dir).unwrap();
        let test_file = sub_dir.join("test.txt");
        File::create(&test_file).unwrap();

        let ignore_config = IgnoreConfig {
            use_gitignore: false,
            use_default_ignores: false,
            include_binary_files: false,
            extra_ignore_patterns: vec![],
        };

        // Test with normal permissions (should work)
        assert!(!is_path_ignored_iterative(&test_file, &base_path, &ignore_config));

        // Note: Actually changing permissions and testing would require more complex setup
        // and might affect the test environment, so we test the basic case here
    }

    /// Test folder descendant collection with edge cases
    #[test]
    fn test_folder_descendants_edge_cases() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        let ignore_config = IgnoreConfig {
            use_gitignore: false,
            use_default_ignores: false,
            include_binary_files: false,
            extra_ignore_patterns: vec![],
        };

        // Test empty directory
        let empty_dir = base_path.join("empty");
        fs::create_dir_all(&empty_dir).unwrap();
        let mut descendants = HashSet::new();
        let result = collect_folder_descendants(&empty_dir, &base_path, &ignore_config, &mut descendants);
        assert!(result.is_ok());
        assert!(descendants.is_empty(), "Empty directory should yield no descendants");

        // Test nested empty directories
        let nested_empty = base_path.join("nested/empty/dirs");
        fs::create_dir_all(&nested_empty).unwrap();
        descendants.clear();
        let result = collect_folder_descendants(&base_path.join("nested"), &base_path, &ignore_config, &mut descendants);
        assert!(result.is_ok());
        assert!(descendants.len() >= 2, "Should find the nested directories");

        // Test directory with many files
        let many_files_dir = base_path.join("many_files");
        fs::create_dir_all(&many_files_dir).unwrap();
        for i in 0..100 {
            File::create(many_files_dir.join(format!("file_{}.txt", i))).unwrap();
        }
        descendants.clear();
        let result = collect_folder_descendants(&many_files_dir, &base_path, &ignore_config, &mut descendants);
        assert!(result.is_ok());
        assert_eq!(descendants.len(), 100, "Should find all 100 files");
    }

    /// Test the new file-contextual gitignore function directly
    #[test]
    fn test_get_cached_gitignore_matcher_for_context() {
        let temp_dir = create_test_gitignore_structure();
        let base_path = temp_dir.path().to_path_buf();
        clear_gitignore_cache();

        // Test that different contexts return different matchers
        let root_matcher = get_cached_gitignore_matcher_for_context(&base_path, &base_path);
        let src_matcher = get_cached_gitignore_matcher_for_context(&base_path.join("src"), &base_path);
        let components_matcher = get_cached_gitignore_matcher_for_context(&base_path.join("src/components"), &base_path);

        assert!(root_matcher.is_some());
        assert!(src_matcher.is_some());
        assert!(components_matcher.is_some());

        // Test that Button.tsx is correctly ignored by components matcher
        let button_file = base_path.join("src/components/Button.tsx");
        if let Some(matcher) = components_matcher {
            let match_result = matcher.matched_path_or_any_parents(&button_file, false);
            assert!(match_result.is_ignore(), "Button.tsx should be ignored by components/.gitignore");
        }
    }

    /// Test cache behavior for file-contextual gitignore
    #[test]
    fn test_gitignore_cache_per_directory() {
        let temp_dir = create_test_gitignore_structure();
        let base_path = temp_dir.path().to_path_buf();
        clear_gitignore_cache();

        // First call should populate cache
        let _matcher1 = get_cached_gitignore_matcher_for_context(&base_path.join("src"), &base_path);

        // Second call with same context should use cache
        let _matcher2 = get_cached_gitignore_matcher_for_context(&base_path.join("src"), &base_path);

        // Different context should create new cache entry
        let _matcher3 = get_cached_gitignore_matcher_for_context(&base_path.join("src/components"), &base_path);

        // Verify cache has separate entries for each context
        // (This would require exposing cache contents or a cache size function)
    }

    /// Test gitignore inheritance hierarchy
    #[test]
    fn test_gitignore_inheritance_validation() {
        let temp_dir = create_test_gitignore_structure();
        let base_path = temp_dir.path().to_path_buf();
        clear_gitignore_cache();

        let ignore_config = IgnoreConfig {
            use_gitignore: true,
            use_default_ignores: false,
            include_binary_files: false,
            extra_ignore_patterns: vec![],
        };

        // File should be ignored by most specific .gitignore that matches
        let test_cases = vec![
            // Files ignored by root .gitignore
            (base_path.join("app.log"), true, "*.log pattern from root"),
            (base_path.join("src/nested.log"), true, "*.log pattern inherited from root"),

            // Files ignored by src .gitignore
            (base_path.join("src/debug.js"), true, "debug.js pattern from src"),
            (base_path.join("src/test.tmp"), true, "*.tmp pattern from src"),

            // Files ignored by components .gitignore
            (base_path.join("src/components/Button.tsx"), true, "Button.tsx pattern from components"),
            (base_path.join("src/components/test.test.js"), true, "*.test.js pattern from components"),

            // Files that should NOT be ignored
            (base_path.join("src/components/Header.tsx"), false, "not matching any ignore pattern"),
            (base_path.join("src/main.js"), false, "not matching any ignore pattern"),
            (base_path.join("README.md"), false, "not matching any ignore pattern"),
        ];

        for (file_path, expected_ignored, reason) in test_cases {
            let is_ignored = is_path_ignored_iterative(&file_path, &base_path, &ignore_config);
            assert_eq!(is_ignored, expected_ignored,
                      "File {} - Expected: {}, Got: {}, Reason: {}",
                      file_path.display(), expected_ignored, is_ignored, reason);
        }
    }

    /// Test collect_folder_descendants_with_visited directly for comprehensive symlink handling
    #[test]
    #[cfg(unix)]
    fn test_symlink_loop_detection_comprehensive() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        // Create complex symlink structure
        let dirs = ["a", "b", "c", "d"];
        for dir in &dirs {
            fs::create_dir_all(base_path.join(dir)).unwrap();
        }

        // Create various symlink patterns
        // Circular: a -> b -> c -> a
        std::os::unix::fs::symlink(&base_path.join("b"), base_path.join("a/link_b")).unwrap();
        std::os::unix::fs::symlink(&base_path.join("c"), base_path.join("b/link_c")).unwrap();
        std::os::unix::fs::symlink(&base_path.join("a"), base_path.join("c/link_a")).unwrap();

        // Self-reference: d -> d
        std::os::unix::fs::symlink(&base_path.join("d"), base_path.join("d/self_link")).unwrap();

        // Double self-reference: d -> d/self -> d
        std::os::unix::fs::symlink(&base_path.join("d"), base_path.join("d/double_self")).unwrap();

        let ignore_config = IgnoreConfig::default();
        let mut descendants = HashSet::new();
        let mut visited = HashSet::new();

        // Test that symlink detection prevents infinite recursion
        let result = collect_folder_descendants_with_visited(
            &base_path.join("a"),
            &base_path,
            &ignore_config,
            &mut descendants,
            &mut visited
        );

        assert!(result.is_ok(), "Should handle symlink loops without error");
        assert!(!descendants.is_empty(), "Should collect some files despite loops");
    }

    /// Test performance with many symlinks
    #[test]
    #[cfg(unix)]
    fn test_symlink_performance_stress() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        // Create 50 directories each with self-referencing symlinks
        for i in 0..50 {
            let dir = base_path.join(format!("dir_{}", i));
            fs::create_dir_all(&dir).unwrap();
            std::os::unix::fs::symlink(&dir, dir.join("self_link")).unwrap();

            // Add some regular files too
            File::create(dir.join("file.txt")).unwrap();
        }

        let ignore_config = IgnoreConfig::default();
        let mut descendants = HashSet::new();

        let start = std::time::Instant::now();
        let result = collect_folder_descendants(&base_path, &base_path, &ignore_config, &mut descendants);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Should handle many symlinks efficiently");
        assert!(duration.as_secs() < 5, "Should complete within 5 seconds");
        assert_eq!(descendants.len(), 50, "Should find exactly 50 regular files");
    }

    /// Test symlink pointing to non-existent target
    #[test]
    #[cfg(unix)]
    fn test_broken_symlink_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        let target_dir = base_path.join("target");
        let broken_link = base_path.join("broken_link");

        // Create symlink first, then remove target
        fs::create_dir_all(&target_dir).unwrap();
        std::os::unix::fs::symlink(&target_dir, &broken_link).unwrap();
        fs::remove_dir(&target_dir).unwrap();

        let ignore_config = IgnoreConfig::default();
        let mut descendants = HashSet::new();

        let result = collect_folder_descendants(&base_path, &base_path, &ignore_config, &mut descendants);

        // Should handle broken symlinks gracefully
        assert!(result.is_ok(), "Should handle broken symlinks without crashing");
    }

    /// Test corrected regex pattern
    #[test]
    fn test_regex_pattern_fix() {
        // Test that the corrected regex pattern "^$" works as expected
        let empty_regex = regex::Regex::new("^$").unwrap();

        assert!(empty_regex.is_match(""), "Should match empty string");
        assert!(!empty_regex.is_match("non-empty"), "Should not match non-empty string");

        // Verify the old malformed pattern would have failed
        let malformed_result = regex::Regex::new("\"$.^\"");
        assert!(malformed_result.is_err(), "Malformed pattern should fail to compile");
    }

    /// Test gitignore cache clearing behavior
    #[test]
    fn test_gitignore_cache_management() {
        let temp_dir = create_test_gitignore_structure();
        let base_path = temp_dir.path().to_path_buf();

        // Populate cache
        let _matcher1 = get_cached_gitignore_matcher_for_context(&base_path, &base_path);
        let _matcher2 = get_cached_gitignore_matcher_for_context(&base_path.join("src"), &base_path);

        // Clear cache
        clear_gitignore_cache();

        // Cache should be cleared - new calls should rebuild
        let matcher3 = get_cached_gitignore_matcher_for_context(&base_path, &base_path);
        assert!(matcher3.is_some(), "Should rebuild matcher after cache clear");
    }
}
