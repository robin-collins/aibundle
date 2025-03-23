## Stage 2: Extract Constants and Types to Models Module

### Stage 2 Goal
Move all constants and types to appropriate model files to create a clear separation of data structures from logic.

### Stage 2 Steps

1. Create `src/models/mod.rs` to declare sub-modules and re-export types:

```rust
mod app_config;
mod enums;
mod constants;

pub use app_config::*;
pub use enums::*;
pub use constants::*;
```

2. Create `src/models/app_config.rs` and move all configuration-related types:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppConfig {
    pub default_format: Option<String>,
    pub default_gitignore: Option<bool>,
    pub default_ignore: Option<Vec<String>>,
    pub default_line_numbers: Option<bool>,
    pub default_recursive: Option<bool>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ModeConfig {
    pub files: Option<String>,
    pub format: Option<String>,
    pub out: Option<String>,
    pub gitignore: Option<bool>,
    pub ignore: Option<Vec<String>>,
    pub line_numbers: Option<bool>,
    pub recursive: Option<bool>,
    pub source_dir: Option<String>,
    pub selection_limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FullConfig {
    pub cli: Option<ModeConfig>,
    pub tui: Option<ModeConfig>,
}

#[derive(Clone)]
pub struct IgnoreConfig {
    pub use_default_ignores: bool,
    pub use_gitignore: bool,
    pub include_binary_files: bool,
    pub extra_ignore_patterns: Vec<String>,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            use_default_ignores: true,
            use_gitignore: true,
            include_binary_files: false,
            extra_ignore_patterns: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct CopyStats {
    pub files: usize,
    pub folders: usize,
}

pub struct Node {
    pub name: String,
    pub is_dir: bool,
    pub children: Option<std::collections::HashMap<String, Node>>,
    pub parent: Option<Box<Node>>,
}

pub struct FileDependencies {
    pub internal_deps: Vec<String>,
    pub external_deps: Vec<String>,
}
```

3. Create `src/models/enums.rs` to hold all enum types:

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Xml,
    Markdown,
    Json,
    Llm,
}

impl OutputFormat {
    pub fn toggle(&self) -> Self {
        match self {
            OutputFormat::Xml => OutputFormat::Markdown,
            OutputFormat::Markdown => OutputFormat::Json,
            OutputFormat::Json => OutputFormat::Llm,
            OutputFormat::Llm => OutputFormat::Xml,
        }
    }
}
```

4. Create `src/models/constants.rs` for all constant values:

```rust
pub const VERSION: &str = "0.6.14";
pub const DEFAULT_SELECTION_LIMIT: usize = 400;

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

pub const DEFAULT_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    "coverage",
    "target",
];
```
