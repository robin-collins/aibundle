## Stage 9: Extract and Modularize TUI Components

### Stage 9 Goal
Transform the monolithic TUI App structure into a collection of smaller, focused components with clear responsibilities.

### Stage 9 Steps

1. Create a more granular TUI module structure:

```
src/tui/
├── mod.rs               - TUI module exports
├── app.rs               - Core App coordinator (significantly simplified)
├── components/
│   ├── mod.rs           - Components module exports
│   ├── modal.rs         - Modal dialog component
│   ├── file_list.rs     - File list display component
│   └── status_bar.rs    - Status bar component
├── state/
│   ├── mod.rs           - State module exports
│   ├── app_state.rs     - Main application state
│   ├── selection.rs     - Selection state management 
│   └── search.rs        - Search state management
├── handlers/
│   ├── mod.rs           - Event handlers module exports
│   ├── keyboard.rs      - Keyboard event handlers
│   ├── clipboard.rs     - Clipboard operation handlers
│   └── file_ops.rs      - File operation handlers
└── views/
    ├── mod.rs           - Views module exports
    ├── main_view.rs     - Main application view
    ├── help_view.rs     - Help screen view
    └── message_view.rs  - Message/notification view
```

2. Create `src/tui/mod.rs` to define the modular TUI structure:

```rust
// Public modules
pub mod components;
pub mod state;
pub mod handlers;
pub mod views;

// App re-export
mod app;
pub use app::App;

// Internal types for TUI system communication
pub(crate) type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
```

3. Create `src/tui/state/mod.rs` to manage application state:

```rust
mod app_state;
mod selection;
mod search;

pub use app_state::AppState;
pub use selection::SelectionState;
pub use search::SearchState;
```

4. Create `src/tui/state/app_state.rs` to hold the application state:

```rust
use std::path::PathBuf;
use std::collections::HashSet;
use std::io;
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
```

5. Create `src/tui/state/selection.rs` to manage item selection:

```rust
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use std::sync::mpsc;
use std::io;
use std::thread;

use crate::fs;
use crate::models::IgnoreConfig;
use crate::tui::state::AppState;

/// Handles selection state operations
pub struct SelectionState {
    pub list_state: ratatui::widgets::ListState,
}

impl Default for SelectionState {
    fn default() -> Self {
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(0));
        Self { list_state }
    }
}

impl SelectionState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn move_selection(&mut self, delta: i32, item_count: usize) {
        if item_count == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_selected = (current as i32 + delta)
            .clamp(0, item_count as i32 - 1) as usize;
        self.list_state.select(Some(new_selected));
    }
    
    pub fn toggle_selection(&mut self, app_state: &mut AppState) -> io::Result<()> {
        if let Some(selected_index) = self.list_state.selected() {
            if selected_index >= app_state.filtered_items.len() {
                return Ok(());
            }
            
            let path = app_state.filtered_items[selected_index].clone();
            if path.file_name().map_or(false, |n| n == "..") {
                return Ok(());
            }
            
            let is_selected = app_state.selected_items.contains(&path);
            
            // If already selected, unselect immediately (no counting needed)
            if is_selected {
                if path.is_dir() {
                    Self::update_folder_selection(app_state, &path, false)?;
                } else {
                    app_state.selected_items.remove(&path);
                }
                return Ok(());
            }
            
            // If not selected, start an async count
            if !app_state.is_counting {
                let (tx, rx) = mpsc::channel();
                let base_path = app_state.current_dir.clone();
                let ignore_config = app_state.ignore_config.clone();
                let path_clone = path.clone();
                let selection_limit = app_state.selection_limit;
                
                thread::spawn(move || {
                    let result = fs::count_selection_items_async(
                        &path_clone,
                        &base_path,
                        &ignore_config,
                        selection_limit,
                    );
                    let _ = tx.send(result);
                });
                
                app_state.pending_count = Some(rx);
                app_state.counting_path = Some(path);
                app_state.is_counting = true;
            }
        }
        
        Ok(())
    }
    
    pub fn update_folder_selection(
        app_state: &mut AppState,
        path: &PathBuf,
        selected: bool
    ) -> io::Result<()> {
        if path.is_dir() {
            if selected {
                app_state.selected_items.insert(path.clone());
            } else {
                app_state.selected_items.remove(path);
            }
            
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let child_path = entry.path();
                    if child_path.is_dir() {
                        Self::update_folder_selection(app_state, &child_path, selected)?;
                    } else if selected {
                        app_state.selected_items.insert(child_path);
                    } else {
                        app_state.selected_items.remove(&child_path);
                    }
                }
            }
        } else if selected {
            app_state.selected_items.insert(path.clone());
        } else {
            app_state.selected_items.remove(path);
        }
        
        Ok(())
    }
    
    pub fn toggle_select_all(&mut self, app_state: &mut AppState) -> io::Result<()> {
        let all_selected = app_state.filtered_items
            .iter()
            .filter(|path| !path.ends_with(".."))
            .all(|path| app_state.selected_items.contains(path));
            
        if all_selected {
            app_state.selected_items.clear();
        } else {
            // Select all items in filtered_items, except ".."
            for path in &app_state.filtered_items {
                if path.file_name().map_or(false, |n| n == "..") {
                    continue;
                }
                
                app_state.selected_items.insert(path.clone());
                
                // If this is a directory, also select all its children
                // but only if we're in recursive mode or the folder is expanded
                if path.is_dir() && (app_state.recursive || app_state.expanded_folders.contains(path)) {
                    Self::update_folder_selection(app_state, path, true)?;
                }
            }
            
            // Count total selected items for the warning
            if !app_state.is_counting && !app_state.filtered_items.is_empty() {
                let (tx, rx) = mpsc::channel();
                let counting_path = app_state.current_dir.clone();
                let base_dir = app_state.current_dir.clone();
                let ignore_config = app_state.ignore_config.clone();
                let selection_limit = app_state.selection_limit;
                
                // Spawn a background thread to count items
                thread::spawn(move || {
                    let result = fs::count_selection_items_async(
                        &counting_path,
                        &base_dir,
                        &ignore_config,
                        selection_limit,
                    );
                    let _ = tx.send(result);
                });
                
                app_state.pending_count = Some(rx);
                app_state.counting_path = Some(counting_path);
                app_state.is_counting = true;
            }
        }
        
        Ok(())
    }
}
```

6. Create `src/tui/state/search.rs` to handle search functionality:

```rust
use std::path::Path;
use glob::Pattern;
use std::collections::HashSet;

/// Manages search state and operations
pub struct SearchState {
    pub search_query: String,
    pub is_searching: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            is_searching: false,
        }
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn toggle_search(&mut self) {
        self.is_searching = !self.is_searching;
    }
    
    pub fn clear_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
    }
    
    pub fn handle_search_input(&mut self, c: char) {
        if !self.is_searching {
            return;
        }
        
        match c {
            '/' => {
                self.is_searching = false;
            }
            _ if !c.is_control() => {
                self.search_query.push(c);
            }
            _ => {}
        }
    }
    
    pub fn handle_backspace(&mut self) {
        if self.is_searching {
            self.search_query.pop();
        }
    }
    
    pub fn create_matcher(&self) -> Box<dyn Fn(&str) -> bool> {
        if self.search_query.is_empty() {
            return Box::new(|_| true);
        }
        
        let query = self.search_query.to_lowercase();
        
        // If query contains wildcards, use glob pattern; otherwise, plain substring
        if query.contains('*') || query.contains('?') {
            match Pattern::new(&query) {
                Ok(pattern) => Box::new(move |name: &str| 
                    pattern.matches(&name.to_lowercase())),
                Err(_) => Box::new(move |name: &str| 
                    name.to_lowercase().contains(&query)),
            }
        } else {
            Box::new(move |name: &str| name.to_lowercase().contains(&query))
        }
    }
}
```

7. Create `src/tui/components/mod.rs` to manage UI components:

```rust
mod modal;
mod file_list;
mod status_bar;

pub use modal::Modal;
pub use file_list::FileList;
pub use status_bar::StatusBar;
```

8. Create `src/tui/components/modal.rs` for the Modal component (similar to the original plan)

```rust
use std::time::Instant;

pub struct Modal {
    pub message: String,
    pub timestamp: Instant,
    pub width: u16,
    pub height: u16,
    pub page: usize,
}

impl Modal {
    pub fn new(message: String, width: u16, height: u16) -> Self {
        Self {
            message,
            timestamp: Instant::now(),
            width,
            height,
            page: 0,
        }
    }

    pub fn copy_stats(
        file_count: usize,
        folder_count: usize,
        line_count: usize,
        byte_size: usize,
        format: &crate::models::OutputFormat,
    ) -> Self {
        Self::new(
            format!(
                "Files copied: {}\n\
                 Folders copied: {}\n\
                 Total lines: {}\n\
                 Total size: {}\n\
                 Format: {}\n",
                file_count,
                folder_count,
                line_count,
                crate::utils::human_readable_size(byte_size),
                match format {
                    crate::models::OutputFormat::Xml => "XML",
                    crate::models::OutputFormat::Markdown => "Markdown",
                    crate::models::OutputFormat::Json => "JSON",
                    crate::models::OutputFormat::Llm => "LLM",
                }
            ),
            45,
            8,
        )
    }

    pub fn help() -> Self {
        let help_text = "Keyboard Shortcuts\n\
═════════════════\n\
\n\
Navigation\n\
──────────\n\
↑/↓        - Move selection\n\
PgUp/PgDn  - Move by 10 items\n\
Enter      - Open directory\n\
Tab        - Expand/collapse folder\n\
\n\
Selection\n\
─────────\n\
Space      - Select/deselect item\n\
*          - Select/deselect all\n\
\n\
Actions\n\
───────\n\
c          - Copy to clipboard\n\
f          - Toggle format (XML/MD/JSON)\n\
n          - Toggle line numbers\n\
/          - Search (ESC to cancel)\n\
\n\
Filters\n\
───────\n\
i          - Toggle default ignores\n\
g          - Toggle .gitignore\n\
b          - Toggle binary files\n\
\n\
Other\n\
─────\n\
h          - Show this help\n\
q          - Quit (copies if items selected)\n\
\n\
Help Navigation\n\
──────────────\n\
PgUp/PgDn  - Scroll help pages\n\
Any key    - Close help"
            .to_string();
        Self {
            message: help_text,
            timestamp: Instant::now(),
            width: 60,
            height: 30,
            page: 0,
        }
    }

    pub fn get_visible_content(&self, available_height: u16) -> (String, bool) {
        let content_height = (available_height - 4) as usize;
        let lines: Vec<&str> = self.message.lines().collect();
        let total_lines = lines.len();
        let total_pages = total_lines.div_ceil(content_height);
        let has_more_pages = total_lines > content_height;
        let start = self.page * content_height;
        let end = (start + content_height).min(total_lines);
        let visible_content = lines[start..end].join("\n");
        let content = if has_more_pages {
            format!(
                "{}\n\nPage {} of {}",
                visible_content,
                self.page + 1,
                total_pages
            )
        } else {
            visible_content
        };
        (content, has_more_pages)
    }

    pub fn next_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = total_lines.div_ceil(content_height);
        if total_pages > 1 {
            self.page = (self.page + 1) % total_pages;
        }
    }

    pub fn prev_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = total_lines.div_ceil(content_height);
        if total_pages > 1 {
            self.page = (self.page + total_pages - 1) % total_pages;
        }
    }
}
```

9. Create `src/tui/components/file_list.rs` to handle file list display:

```rust
use ratatui::{
    widgets::{List, ListItem, Block, Borders},
    style::{Style, Color},
};
use std::path::{Path, PathBuf};

use crate::models::ICONS;
use crate::tui::state::AppState;

pub struct FileList {
    pub current_dir: PathBuf,
}

impl FileList {
    pub fn new(current_dir: PathBuf) -> Self {
        Self { current_dir }
    }
    
    pub fn render<'a>(&self, app_state: &AppState) -> List<'a> {
        let items: Vec<ListItem> = app_state.filtered_items
            .iter()
            .map(|path| {
                let depth = path
                    .strip_prefix(&app_state.current_dir)
                    .map(|p| p.components().count())
                    .unwrap_or(0)
                    .saturating_sub(1);
                let indent = "  ".repeat(depth);
                
                let name = if path.ends_with("..") {
                    "../".to_string()
                } else {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("???")
                        .to_string()
                };
                
                let icon = Self::get_icon(path);
                let prefix = if app_state.selected_items.contains(path) {
                    "[X] "
                } else {
                    "[ ] "
                };
                
                let display_name = if path.is_dir() && !path.ends_with("..") {
                    format!("{}{}{} {}/", indent, prefix, icon, name)
                } else {
                    format!("{}{}{} {}", indent, prefix, icon, name)
                };
                
                ListItem::new(display_name)
            })
            .collect();
            
        List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Gray))
            .highlight_symbol("> ")
    }
    
    fn get_icon(path: &Path) -> &'static str {
        if path.is_dir() {
            return ICONS
                .iter()
                .find(|(k, _)| *k == "folder")
                .map(|(_, v)| *v)
                .unwrap_or("ðŸ"");
        }
        
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| ICONS.iter().find(|(k, _)| *k == ext))
            .map(|(_, v)| *v)
            .unwrap_or(
                ICONS
                    .iter()
                    .find(|(k, _)| *k == "default")
                    .map(|(_, v)| *v)
                    .unwrap_or("ðŸ"„"),
            )
    }
}
```

10. Create `src/tui/components/status_bar.rs` for the status bar:

```rust
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    layout::Alignment,
    style::Style,
};

use crate::tui::state::AppState;

pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }
    
    pub fn render<'a>(&self, app_state: &AppState) -> Block<'a> {
        let status_text = format!(
            " {} items ({} selected) - Space: select, Enter: open dir, c: copy, i: ignores [{}], g: gitignore [{}], b: binary [{}], f: format [{}], n: line numbers [{}], /: search, q: quit ",
            app_state.filtered_items.len(),
            app_state.selected_items.len(),
            if app_state.ignore_config.use_default_ignores { "x" } else { " " },
            if app_state.ignore_config.use_gitignore { "x" } else { " " },
            if app_state.ignore_config.include_binary_files { "x" } else { " " },
            match app_state.output_format {
                crate::models::OutputFormat::Xml => "XML",
                crate::models::OutputFormat::Markdown => "Markdown",
                crate::models::OutputFormat::Json => "JSON",
                crate::models::OutputFormat::Llm => "LLM",
            },
            if app_state.show_line_numbers { "x" } else { " " },
        );
        
        Block::default().title(status_text).borders(Borders::ALL)
    }
}
```

11. Create `src/tui/handlers/mod.rs` to organize event handlers:

```rust
mod keyboard;
mod clipboard;
mod file_ops;

pub use keyboard::KeyboardHandler;
pub use clipboard::ClipboardHandler;
pub use file_ops::FileOpsHandler;
```

12. Create `src/tui/handlers/keyboard.rs` to handle keyboard input:

```rust
use std::io;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::tui::state::{AppState, SelectionState, SearchState};
use crate::tui::handlers::{ClipboardHandler, FileOpsHandler};

pub struct KeyboardHandler;

impl KeyboardHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub fn handle_key(
        &self,
        key: KeyEvent,
        app_state: &mut AppState,
        selection_state: &mut SelectionState,
        search_state: &mut SearchState,
    ) -> io::Result<()> {
        // Only handle key press events
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        
        // If searching, handle search input
        if search_state.is_searching {
            match key.code {
                KeyCode::Esc => search_state.clear_search(),
                KeyCode::Backspace => {
                    search_state.handle_backspace();
                    FileOpsHandler::update_search(app_state, search_state)?;
                }
                KeyCode::Char(c) => {
                    search_state.handle_search_input(c);
                    FileOpsHandler::update_search(app_state, search_state)?;
                }
                _ => {}
            }
            return Ok(());
        }
        
        // Normal mode key handling
        match key.code {
            KeyCode::Char('q') => {
                if !app_state.selected_items.is_empty() {
                    ClipboardHandler::copy_selected_to_clipboard(app_state)?;
                }
                app_state.quit = true;
            }
            KeyCode::Char('*') => selection_state.toggle_select_all(app_state)?,
            KeyCode::Char(' ') => selection_state.toggle_selection(app_state)?,
            KeyCode::Char('c') => ClipboardHandler::copy_selected_to_clipboard(app_state)?,
            KeyCode::Char('i') => FileOpsHandler::toggle_default_ignores(app_state)?,
            KeyCode::Char('g') => FileOpsHandler::toggle_gitignore(app_state)?,
            KeyCode::Char('b') => FileOpsHandler::toggle_binary_files(app_state)?,
            KeyCode::Char('f') => FileOpsHandler::toggle_output_format(app_state),
            KeyCode::Char('n') => FileOpsHandler::toggle_line_numbers(app_state),
            KeyCode::Char('s') => FileOpsHandler::save_config(app_state)?,
            KeyCode::Char('/') => {
                search_state.toggle_search();
                if !search_state.is_searching {
                    FileOpsHandler::update_search(app_state, search_state)?;
                }
            }
            KeyCode::Tab => FileOpsHandler::toggle_folder_expansion(app_state, selection_state)?,
            KeyCode::Up => selection_state.move_selection(-1, app_state.filtered_items.len()),
            KeyCode::Down => selection_state.move_selection(1, app_state.filtered_items.len()),
            KeyCode::PageUp => selection_state.move_selection(-10, app_state.filtered_items.len()),
            KeyCode::PageDown => selection_state.move_selection(10, app_state.filtered_items.len()),
            KeyCode::Enter => FileOpsHandler::handle_enter(app_state, selection_state)?,
            KeyCode::Char('h') => FileOpsHandler::show_help(app_state),
            _ => {}
        }
        
        Ok(())
    }
}
```

13. Create `src/tui/handlers/clipboard.rs` for clipboard operations:

```rust
use std::io;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

use crate::models::{CopyStats, Node, OutputFormat};
use crate::tui::state::AppState;

pub struct ClipboardHandler;

impl ClipboardHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub fn copy_selected_to_clipboard(app_state: &mut AppState) -> io::Result<()> {
        let output = Self::format_selected_items(app_state)?;
        
        let result = crate::clipboard::copy_to_clipboard(&output);
        
        // Get the counts for the status message
        let (file_count, folder_count) = Self::count_selected_items(app_state);
        let line_count = output.lines().count();
        let byte_size = output.len();
        
        // Set last copy stats for display in the UI
        app_state.last_copy_stats = Some(CopyStats {
            files: file_count,
            folders: folder_count,
        });
        
        // Display a modal with the copy stats
        app_state.modal = Some(crate::tui::components::Modal::copy_stats(
            file_count,
            folder_count,
            line_count,
            byte_size,
            &app_state.output_format,
        ));
        
        result
    }
    
    fn count_selected_items(app_state: &AppState) -> (usize, usize) {
        let mut files = 0;
        let mut folders = 0;
        
        for path in &app_state.selected_items {
            if path.is_dir() {
                folders += 1;
            } else {
                files += 1;
            }
        }
        
        (files, folders)
    }
    
    pub fn format_selected_items(app_state: &AppState) -> io::Result<String> {
        let mut output = String::new();
        let selected_items: Vec<_> = app_state.selected_items
            .iter()
            .filter(|p| !app_state.is_path_ignored(p))
            .cloned()
            .collect();
            
        if selected_items.is_empty() {
            return Ok("No items selected or all items are ignored.".to_string());
        }
        
        let base_path = &app_state.current_dir;
        let mut file_contents = Vec::new();
        
        // Process all selected items
        for path in &selected_items {
            if path.is_dir() {
                Self::process_directory(app_state, path, &mut file_contents, base_path)?;
            } else {
                Self::process_file(app_state, path, &mut file_contents, base_path)?;
            }
        }
        
        // Sort file contents by path for consistent output
        file_contents.sort_by(|(a, _), (b, _)| a.cmp(b));
        
        // Format the output based on the selected format
        match app_state.output_format {
            OutputFormat::Xml => {
                crate::output::format_xml_output(&mut output, &file_contents, app_state.show_line_numbers);
            }
            OutputFormat::Markdown => {
                crate::output::format_markdown_output(&mut output, &file_contents, app_state.show_line_numbers);
            }
            OutputFormat::Json => {
                crate::output::format_json_output(&mut output, &file_contents);
            }
            OutputFormat::Llm => {
                // For LLM format, we need to build a tree structure
                let mut root_node = Node {
                    name: base_path.display().to_string(),
                    is_dir: true,
                    children: Some(std::collections::HashMap::new()),
                    parent: None,
                };
                
                // Build the tree structure
                for (path, _) in &file_contents {
                    Self::add_to_tree(path, &mut root_node, base_path);
                }
                
                // Analyze dependencies if we're doing LLM format
                let dependencies = crate::output::analyze_dependencies(&file_contents, base_path);
                
                // Format the output
                crate::output::format_llm_output(
                    &mut output,
                    &file_contents,
                    base_path,
                    &root_node,
                    &dependencies,
                );
            }
        }
        
        Ok(output)
    }
    
    fn process_directory(
        app_state: &AppState,
        path: &PathBuf,
        file_contents: &mut Vec<(String, String)>,
        base_path: &PathBuf,
    ) -> io::Result<()> {
        // Skip if this directory should be ignored
        if app_state.is_path_ignored(path) {
            return Ok(());
        }
        
        // Try to read the directory entries
        match std::fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.filter_map(Result::ok) {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        Self::process_directory(app_state, &entry_path, file_contents, base_path)?;
                    } else {
                        Self::process_file(app_state, &entry_path, file_contents, base_path)?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading directory {}: {}", path.display(), e);
            }
        }
        
        Ok(())
    }
    
    fn process_file(
        app_state: &AppState,
        path: &PathBuf,
        file_contents: &mut Vec<(String, String)>,
        base_path: &PathBuf,
    ) -> io::Result<()> {
        // Skip if this file should be ignored
        if app_state.is_path_ignored(path) {
            return Ok(());
        }
        
        // Skip binary files unless explicitly included
        if !app_state.ignore_config.include_binary_files && crate::tui::App::is_binary_file(path) {
            return Ok(());
        }
        
        // Try to read the file contents
        match std::fs::read_to_string(path) {
            Ok(content) => {
                if let Ok(relative_path) = path.strip_prefix(base_path) {
                    let path_str = relative_path.to_string_lossy().replace('\\', "/");
                    file_contents.push((path_str.to_string(), content));
                }
            }
            Err(e) => {
                eprintln!("Error reading file {}: {}", path.display(), e);
            }
        }
        
        Ok(())
    }
    
    fn add_to_tree(path_str: &str, root: &mut Node, base_dir: &Path) {
        let path = Path::new(path_str);
        let mut current = root;
        
        for component in path.components() {
            let name = component.as_os_str().to_string_lossy().to_string();
            if name.is_empty() {
                continue;
            }
            
            let is_dir = component != path.components().last().unwrap();
            
            if current.children.is_none() {
                current.children = Some(std::collections::HashMap::new());
            }
            
            let children = current.children.as_mut().unwrap();
            
            if !children.contains_key(&name) {
                children.insert(
                    name.clone(),
                    Node {
                        name,
                        is_dir,
                        children: if is_dir {
                            Some(std::collections::HashMap::new())
                        } else {
                            None
                        },
                        parent: None,
                    },
                );
            }
            
            current = children.get_mut(&name).unwrap();
        }
    }
}
```

14. Create `src/tui/handlers/file_ops.rs` for file operations:

```rust
use std::io;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

use crate::models::{CopyStats, Node, OutputFormat};
use crate::tui::state::AppState;

pub struct FileOpsHandler;

impl FileOpsHandler {
    pub fn load_items(app_state: &mut AppState) -> io::Result<()> {
        // Implementation for loading items
        Ok(())
    }

    pub fn load_items_nonrecursive(app_state: &mut AppState) -> io::Result<()> {
        // Implementation for loading non-recursive items
        Ok(())
    }

    pub fn update_search(app_state: &mut AppState, search_state: &mut SearchState) -> io::Result<()> {
        // Implementation for updating search state
        Ok(())
    }

    pub fn format_selected_items(app_state: &mut AppState) -> io::Result<String> {
        // Implementation for formatting selected items
        Ok(String::new())
    }

    pub fn handle_enter(app_state: &mut AppState, selection_state: &mut SelectionState) -> io::Result<()> {
        // Implementation for handling enter key
        Ok(())
    }

    pub fn toggle_default_ignores(app_state: &mut AppState) -> io::Result<()> {
        // Implementation for toggling default ignores
        Ok(())
    }

    pub fn toggle_gitignore(app_state: &mut AppState) -> io::Result<()> {
        // Implementation for toggling .gitignore
        Ok(())
    }

    pub fn toggle_binary_files(app_state: &mut AppState) -> io::Result<()> {
        // Implementation for toggling binary files
        Ok(())
    }

    pub fn toggle_output_format(app_state: &mut AppState) -> io::Result<()> {
        // Implementation for toggling output format
        Ok(())
    }

    pub fn toggle_line_numbers(app_state: &mut AppState) -> io::Result<()> {
        // Implementation for toggling line numbers
        Ok(())
    }

    pub fn save_config(app_state: &mut AppState) -> io::Result<()> {
        // Implementation for saving configuration
        Ok(())
    }

    pub fn check_pending_selection(app_state: &mut AppState, selection_state: &mut SelectionState) -> io::Result<()> {
        // Implementation for checking pending selection count
        Ok(())
    }

    pub fn show_help(app_state: &mut AppState) -> io::Result<()> {
        // Implementation for showing help
        Ok(())
    }
}
```

15. Create `src/tui/views/mod.rs` to organize the UI views:

```rust
mod main_view;
mod help_view;
mod message_view;

pub use main_view::MainView;
pub use help_view::HelpView;
pub use message_view::MessageView;
```

16. Create view implementations to render different UI states

16a. Create `src/tui/views/main_view.rs` for the main view:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::models::constants::ICONS;
use crate::tui::components::{FileList, Modal, StatusBar};
use crate::tui::state::{AppState, SearchState, SelectionState};
use crate::tui::views::{HelpView, MessageView};

pub struct MainView {
    file_list: FileList,
    status_bar: StatusBar,
    help_view: HelpView,
    message_view: MessageView,
}

impl MainView {
    pub fn new() -> Self {
        Self {
            file_list: FileList::new(),
            status_bar: StatusBar::new(),
            help_view: HelpView::new(),
            message_view: MessageView::new(),
        }
    }

    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState,
        selection_state: &SelectionState,
        search_state: &Option<SearchState>,
    ) {
        // Create the main layout with file list and status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),     // File list takes most of the space
                Constraint::Length(1),  // Status bar at bottom
            ])
            .split(area);

        // Render file list
        self.file_list.render(f, chunks[0], app_state, selection_state);
        
        // Render status bar
        self.status_bar.render(f, chunks[1], app_state, selection_state);

        // Render search UI if in search mode
        if let Some(search_state) = search_state {
            let search_area = Rect {
                x: area.x + 1,
                y: area.y + area.height - 2,
                width: area.width - 2,
                height: 1,
            };
            let search_text = format!(
                "{} {}",
                if search_state.is_regex { "Regex:" } else { "Search:" },
                search_state.query
            );
            let search_para = Paragraph::new(search_text)
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(search_para, search_area);
        }

        // Render help view or message view if active
        if app_state.show_help {
            self.help_view.render(f, area, app_state);
        } else if app_state.show_message {
            self.message_view.render(f, area, app_state);
        }

        // Render modal if active
        if let Some(modal) = &app_state.modal {
            let modal_area = centered_rect(60, 20, area);
            modal.render(f, modal_area);
        }
    }
}

// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

16b. Create `src/tui/views/help_view.rs` for the help view:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct HelpView;

impl HelpView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, area: Rect, app_state: &crate::tui::state::AppState) {
        // Create a centered area for the help content
        let help_area = centered_rect(80, 90, area);

        // Create a block with a border for the help view
        let help_block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        // Help content
        let help_text = vec![
            Spans::from(Span::styled(
                "Keyboard Controls",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Spans::from(""),
            Spans::from(Span::styled(
                "Navigation",
                Style::default().fg(Color::Green),
            )),
            Spans::from("↑/↓         - Move selection up/down"),
            Spans::from("PgUp/PgDown - Move selection by page"),
            Spans::from("Enter       - Open directory"),
            Spans::from("Backspace   - Go to parent directory"),
            Spans::from("Tab         - Expand/collapse directory"),
            Spans::from(""),
            Spans::from(Span::styled("Selection", Style::default().fg(Color::Green))),
            Spans::from("Space       - Select/deselect item"),
            Spans::from("*           - Select/deselect all"),
            Spans::from(""),
            Spans::from(Span::styled("Search", Style::default().fg(Color::Green))),
            Spans::from("/           - Start search"),
            Spans::from("ESC         - Clear search"),
            Spans::from(""),
            Spans::from(Span::styled("Actions", Style::default().fg(Color::Green))),
            Spans::from("c           - Copy selection to clipboard"),
            Spans::from("f           - Toggle output format (XML/MD/JSON/LLM)"),
            Spans::from("n           - Toggle line numbers"),
            Spans::from(""),
            Spans::from(Span::styled("Filters", Style::default().fg(Color::Green))),
            Spans::from("i           - Toggle default ignore patterns"),
            Spans::from("g           - Toggle .gitignore"),
            Spans::from("b           - Toggle binary files"),
            Spans::from("r           - Toggle recursive mode"),
            Spans::from(""),
            Spans::from(Span::styled("Other", Style::default().fg(Color::Green))),
            Spans::from("h           - Show/hide this help"),
            Spans::from("q           - Quit"),
            Spans::from(""),
            Spans::from(Span::styled(
                "Press any key to close help",
                Style::default().fg(Color::Yellow),
            )),
        ];

        // Render the help paragraph
        let help_paragraph = Paragraph::new(help_text)
            .block(help_block)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        f.render_widget(help_paragraph, help_area);
    }
}

// Helper function to create a centered rectangle for the help view
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

16c. Create `src/tui/views/message_view.rs` for the message view:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::time::{Duration, Instant};

pub struct MessageView {
    // Duration to show messages for
    message_duration: Duration,
}

impl MessageView {
    pub fn new() -> Self {
        Self {
            message_duration: Duration::from_secs(3),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, app_state: &crate::tui::state::AppState) {
        // Only render if there's a message to show
        if let Some(message) = &app_state.message {
            // Check if the message should be expired
            if Instant::now().duration_since(message.timestamp) > self.message_duration {
                return;
            }

            // Calculate a smaller centered area for the message
            let message_area = centered_rect(60, 15, area);

            // Create a block with a border for the message
            let message_block = Block::default()
                .title(" Message ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow));

            // Determine message style based on message type
            let message_style = match message.message_type {
                crate::tui::state::MessageType::Info => Style::default().fg(Color::Cyan),
                crate::tui::state::MessageType::Success => Style::default().fg(Color::Green),
                crate::tui::state::MessageType::Warning => Style::default().fg(Color::Yellow),
                crate::tui::state::MessageType::Error => Style::default().fg(Color::Red),
            };

            // Create the message paragraph with styled spans
            let message_content = Spans::from(vec![
                Span::styled(
                    match message.message_type {
                        crate::tui::state::MessageType::Info => "INFO: ",
                        crate::tui::state::MessageType::Success => "SUCCESS: ",
                        crate::tui::state::MessageType::Warning => "WARNING: ",
                        crate::tui::state::MessageType::Error => "ERROR: ",
                    },
                    message_style.add_modifier(Modifier::BOLD),
                ),
                Span::styled(&message.content, message_style),
            ]);

            let message_paragraph = Paragraph::new(vec![
                Spans::from(""),
                message_content,
                Spans::from(""),
                Spans::from(Span::styled(
                    "Press any key to dismiss",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(message_block)
            .wrap(Wrap { trim: true });

            f.render_widget(message_paragraph, message_area);
        }
    }

    pub fn set_message_duration(&mut self, duration: Duration) {
        self.message_duration = duration;
    }
}

// Helper function to create a centered rectangle for the message
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

17. Finally, create a simplified `src/tui/app.rs` that coordinates all components:

```rust
use std::io;
use std::path::PathBuf;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::time::Duration;

use crate::models::{AppConfig, OutputFormat};
use crate::tui::state::{AppState, SelectionState, SearchState};
use crate::tui::handlers::{KeyboardHandler, ClipboardHandler, FileOpsHandler};
use crate::tui::views::MainView;

pub struct App {
    // Application state
    pub state: AppState,
    
    // UI state managers
    selection_state: SelectionState,
    search_state: SearchState,
    
    // Event handlers
    keyboard_handler: KeyboardHandler,
    
    // Views
    main_view: MainView,
    
    // Public-facing properties (for compatibility with existing code)
    pub current_dir: PathBuf,
    pub config: AppConfig,
    pub ignore_config: crate::models::IgnoreConfig,
    pub selected_items: std::collections::HashSet<PathBuf>,
    pub output_format: OutputFormat,
    pub show_line_numbers: bool,
    pub recursive: bool,
    pub expanded_folders: std::collections::HashSet<PathBuf>,
    pub search_query: String,
    pub filtered_items: Vec<PathBuf>,
    pub items: Vec<PathBuf>,
    pub selection_limit: usize,
}

impl App {
    pub fn new() -> Self {
        let app_state = AppState::new();
        
        // Create a new instance with all components initialized
        let mut app = Self {
            current_dir: app_state.current_dir.clone(),
            config: app_state.config.clone(),
            ignore_config: app_state.ignore_config.clone(),
            selected_items: std::collections::HashSet::new(),
            output_format: app_state.output_format,
            show_line_numbers: app_state.show_line_numbers,
            recursive: app_state.recursive,
            expanded_folders: std::collections::HashSet::new(),
            search_query: String::new(),
            filtered_items: Vec::new(),
            items: Vec::new(),
            selection_limit: app_state.selection_limit,
            
            state: app_state,
            selection_state: SelectionState::new(),
            search_state: SearchState::new(),
            keyboard_handler: KeyboardHandler::new(),
            main_view: MainView::new(),
        };
        
        // Initialize compatibility properties
        app.sync_state_to_properties();
        
        app
    }
    
    // Synchronizes internal state to public properties for compatibility
    fn sync_state_to_properties(&mut self) {
        self.current_dir = self.state.current_dir.clone();
        self.config = self.state.config.clone();
        self.ignore_config = self.state.ignore_config.clone();
        self.selected_items = self.state.selected_items.clone();
        self.output_format = self.state.output_format;
        self.show_line_numbers = self.state.show_line_numbers;
        self.recursive = self.state.recursive;
        self.expanded_folders = self.state.expanded_folders.clone();
        self.search_query = self.search_state.search_query.clone();
        self.filtered_items = self.state.filtered_items.clone();
        self.items = self.state.items.clone();
        self.selection_limit = self.state.selection_limit;
    }
    
    // Synchronizes public properties to internal state for compatibility
    fn sync_properties_to_state(&mut self) {
        self.state.current_dir = self.current_dir.clone();
        self.state.config = self.config.clone();
        self.state.ignore_config = self.ignore_config.clone();
        self.state.selected_items = self.selected_items.clone();
        self.state.output_format = self.output_format;
        self.state.show_line_numbers = self.show_line_numbers;
        self.state.recursive = self.recursive;
        self.state.expanded_folders = self.expanded_folders.clone();
        self.search_state.search_query = self.search_query.clone();
        self.state.filtered_items = self.filtered_items.clone();
        self.state.items = self.items.clone();
        self.state.selection_limit = self.selection_limit;
    }
    
    pub fn run(&mut self) -> io::Result<()> {
        // Synchronize any external changes to internal state
        self.sync_properties_to_state();
        
        // Setup terminal
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Load initial items
        FileOpsHandler::load_items(&mut self.state)?;
        
        // Main event loop
        while !self.state.quit {
            // Check for pending selection count results
            FileOpsHandler::check_pending_selection(&mut self.state, &mut self.selection_state)?;
            
            // Render UI
            terminal.draw(|f| self.main_view.render(f, &self.state, &self.selection_state, &self.search_state))?;
            
            // Handle events with timeout
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    self.keyboard_handler.handle_key(key, &mut self.state, &mut self.selection_state, &mut self.search_state)?;
                }
            }
            
            // Keep compatibility properties in sync
            self.sync_state_to_properties();
        }
        
        // Restore terminal
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        
        // Print final statistics if anything was copied
        if let Some(stats) = &self.state.last_copy_stats {
            if let Ok(contents) = crate::clipboard::get_clipboard_contents() {
                let line_count = contents.lines().count();
                let byte_size = contents.len();
                println!("\nCopied to clipboard:");
                println!("  Files copied: {}", stats.files);
                println!("  Folders copied: {}", stats.folders);
                println!("  Total lines: {}", line_count);
                println!("  Total size: {}", crate::utils::human_readable_size(byte_size));
                println!();
            }
        }
        
        Ok(())
    }
    
    // Delegating methods for backward compatibility
    
    pub fn load_items(&mut self) -> io::Result<()> {
        self.sync_properties_to_state();
        let result = FileOpsHandler::load_items(&mut self.state);
        self.sync_state_to_properties();
        result
    }
    
    pub fn load_items_nonrecursive(&mut self) -> io::Result<()> {
        self.sync_properties_to_state();
        let result = FileOpsHandler::load_items_nonrecursive(&mut self.state);
        self.sync_state_to_properties();
        result
    }
    
    pub fn update_search(&mut self) {
        self.sync_properties_to_state();
        let _ = FileOpsHandler::update_search(&mut self.state, &mut self.search_state);
        self.sync_state_to_properties();
    }
    
    pub fn format_selected_items(&mut self) -> io::Result<String> {
        self.sync_properties_to_state();
        ClipboardHandler::format_selected_items(&mut self.state)
    }
    
    // Other delegating methods as needed...
}
```
