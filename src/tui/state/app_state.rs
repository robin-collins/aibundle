use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::models::{AppConfig, CopyStats, IgnoreConfig, OutputFormat};

/// Core application state, separated from behavior
pub struct AppState {
    // File system state
    pub current_dir: PathBuf,
    pub items: Vec<PathBuf>,
    pub filtered_items: Vec<PathBuf>,
    pub expanded_folders: HashSet<PathBuf>,

    // Configuration
    pub config: AppConfig,
    pub ignore_config: IgnoreConfig,
    pub output_format: OutputFormat,
    pub show_line_numbers: bool,
    pub selection_limit: usize,
    pub recursive: bool,

    // UI state
    pub quit: bool,
    pub is_counting: bool,

    // Selection state
    pub selected_items: HashSet<PathBuf>,
    pub counting_path: Option<PathBuf>,
    pub pending_count: Option<mpsc::Receiver<io::Result<usize>>>,

    // Operation results
    pub last_copy_stats: Option<CopyStats>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_default(),
            items: Vec::new(),
            filtered_items: Vec::new(),
            expanded_folders: HashSet::new(),

            config: AppConfig::default(),
            ignore_config: IgnoreConfig::default(),
            output_format: OutputFormat::Xml,
            show_line_numbers: false,
            selection_limit: crate::models::DEFAULT_SELECTION_LIMIT,
            recursive: false,

            quit: false,
            is_counting: false,

            selected_items: HashSet::new(),
            counting_path: None,
            pending_count: None,

            last_copy_stats: None,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn selected_count(&self) -> usize {
        self.selected_items.len()
    }

    pub fn item_count(&self) -> usize {
        self.filtered_items.len()
    }

    pub fn is_file_selected(&self, path: &PathBuf) -> bool {
        self.selected_items.contains(path)
    }
}
