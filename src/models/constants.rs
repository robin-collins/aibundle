// src/models/constants.rs
// CANONICAL: All icons, ignored directories, and language mappings must be defined here. Do not redefine elsewhere.
//
// Use get_language_name for extension → language name mapping everywhere.
//
//!
//! # Constants Module
//!
//! This module defines all canonical constants for the application, including:
//! - Application version
//! - Default selection limits
//! - File and folder icons for UI display
//! - Default ignored directories
//! - Language name mappings for file extensions
//!
//! ## Purpose
//! - Provide a single source of truth for all constants used throughout the application.
//! - Ensure consistency for icons, ignored directories, and language mappings.
//!
//! ## Example
//! ```rust
//! use crate::models::constants::{ICONS, DEFAULT_IGNORED_DIRS, get_language_name};
//! let lang = get_language_name("rs");
//! assert_eq!(lang, "Rust");
//! ```

/// The current version of the application.
///
/// Used for CLI and TUI version display and for embedding version info in output files.
///
/// # Example
/// ```rust
/// use crate::models::constants::VERSION;
/// assert_eq!(VERSION, "0.7.7");
/// ```
#[doc(alias = "version")]
pub const VERSION: &str = "0.7.7";

/// The default maximum number of items that can be selected at once.
///
/// Used to prevent excessive memory usage or accidental large operations.
///
/// # Example
/// ```rust
/// use crate::models::constants::DEFAULT_SELECTION_LIMIT;
/// assert_eq!(DEFAULT_SELECTION_LIMIT, 400);
/// ```
#[doc(alias = "selection-limit")]
pub const DEFAULT_SELECTION_LIMIT: usize = 400;

/// Icon mappings for file types, extensions, and special files.
///
/// Maps file extensions and special names to Unicode icons for display in the TUI and CLI outputs.
///
/// # Format
/// Each tuple is (extension_or_name, icon_unicode_str).
///
/// # Example
/// ```rust
/// use crate::models::constants::ICONS;
/// let icon = ICONS.iter().find(|(k, _)| *k == "rs").unwrap().1;
/// assert_eq!(icon, "🦀");
/// ```
///
/// # TODO : Consider moving to a HashMap for faster lookup if performance becomes an issue.
///
#[doc(alias = "icons")]
pub const ICONS: &[(&str, &str)] = &[
    // Folders
    ("folder", "📁"),
    ("folder_open", "📂"),
    // Text files
    ("txt", "📄"),
    ("md", "📝"),
    ("markdown", "📝"),
    ("rst", "📝"),
    // Code files
    ("rs", "🦀"),
    ("py", "🐍"),
    ("js", "🟨"),
    ("jsx", "⚛️"),
    ("ts", "🔷"),
    ("tsx", "🔷"),
    ("html", "🌐"),
    ("htm", "🌐"),
    ("css", "🎨"),
    ("scss", "🎨"),
    ("less", "🎨"),
    ("json", "📋"),
    ("toml", "⚙️"),
    ("yaml", "⚙️"),
    ("yml", "⚙️"),
    ("xml", "📋"),
    ("c", "🔧"),
    ("cpp", "🔧"),
    ("h", "🔧"),
    ("hpp", "🔧"),
    ("go", "🔹"),
    ("java", "☕"),
    ("class", "☕"),
    ("rb", "💎"),
    ("php", "🐘"),
    ("sh", "🐚"),
    ("bash", "🐚"),
    ("zsh", "🐚"),
    ("fish", "🐚"),
    ("bat", "🖥️"),
    ("cmd", "🖥️"),
    ("ps1", "🖥️"),
    ("sql", "🗄️"),
    ("db", "🗄️"),
    ("sqlite", "🗄️"),
    // Configuration
    ("config", "⚙️"),
    ("conf", "⚙️"),
    ("ini", "⚙️"),
    ("env", "⚙️"),
    // Binaries and executables
    ("exe", "⚡"),
    ("dll", "⚡"),
    ("so", "⚡"),
    ("o", "⚡"),
    ("bin", "⚡"),
    // Archives
    ("zip", "📦"),
    ("tar", "📦"),
    ("gz", "📦"),
    ("rar", "📦"),
    ("7z", "📦"),
    // Images
    ("jpg", "🖼️"),
    ("jpeg", "🖼️"),
    ("png", "🖼️"),
    ("gif", "🖼️"),
    ("bmp", "🖼️"),
    ("svg", "🖼️"),
    ("webp", "🖼️"),
    ("ico", "🖼️"),
    // Documents
    ("pdf", "📕"),
    ("doc", "📘"),
    ("docx", "📘"),
    ("xls", "📗"),
    ("xlsx", "📗"),
    ("ppt", "📙"),
    ("pptx", "📙"),
    // Audio/Video
    ("mp3", "🎵"),
    ("wav", "🎵"),
    ("ogg", "🎵"),
    ("flac", "🎵"),
    ("mp4", "🎬"),
    ("avi", "🎬"),
    ("mkv", "🎬"),
    ("mov", "🎬"),
    ("webm", "🎬"),
    // Git-related
    ("git", "🔄"),
    ("gitignore", "🔄"),
    ("gitmodules", "🔄"),
    // Dotfiles
    ("bashrc", "⚙️"),
    ("zshrc", "⚙️"),
    ("vimrc", "⚙️"),
    ("npmrc", "⚙️"),
    // Package management
    ("package.json", "📦"),
    ("Cargo.toml", "📦"),
    ("Cargo.lock", "🔒"),
    ("Gemfile", "💎"),
    ("requirements.txt", "🐍"),
    // Default fallback
    ("default", "📄"),
];

/// Default directories to ignore during file system traversal.
///
/// Commonly ignored in most projects and used by default in both CLI and TUI modes.
///
/// # Example
/// ```rust
/// use crate::models::constants::DEFAULT_IGNORED_DIRS;
/// assert!(DEFAULT_IGNORED_DIRS.contains(&"node_modules"));
/// ```
#[doc(alias = "ignored-dirs")]
pub const DEFAULT_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    "coverage",
    "target",
];

/// Returns the canonical language name for a given file extension.
///
/// Maps file extensions to their canonical language names for display, formatting, and output purposes.
///
/// # Arguments
/// * `extension` - The file extension (without dot), e.g., "rs", "py", "js".
///
/// # Returns
/// * The canonical language name as a `&'static str`. Returns "Plain Text" if the extension is not recognized.
///
/// # Example
/// ```rust
/// use crate::models::constants::get_language_name;
/// assert_eq!(get_language_name("rs"), "Rust");
/// assert_eq!(get_language_name("unknown"), "Plain Text");
/// ```
#[doc(alias = "language-mapping")]
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

// TODO: Add more language mappings as new file types are supported.
// TODO: Consider making ICONS and DEFAULT_IGNORED_DIRS configurable at runtime if user customization is needed.
