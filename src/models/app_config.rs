use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

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
