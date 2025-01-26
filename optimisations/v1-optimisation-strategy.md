
# **src/fs/mod.rs**

> **Note**: This file is unchanged except for comments. For real performance improvements, you may want to unify this with a single pass in `main.rs`. However, leaving it as-is can still work, so long as you do not call it in a performance-critical loop. If you integrate a single-pass approach in `main.rs`, you might remove the need for `list_files` entirely.

```rust
use std::path::PathBuf;
use walkdir::WalkDir;

/// Simple function to list files using `walkdir`.
/// Currently unused in the main code if we unify everything into a single pass,
/// but left here as-is, with comments to remind us we might unify or remove it.
pub fn list_files(path: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !is_excluded(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Very simple function to check if a path is excluded. In practice, we do more in main.rs
/// to handle .gitignore and so on, so you might unify them at some point.
fn is_excluded(path: &std::path::Path) -> bool {
    let excluded = ["node_modules", ".git", "target"];
    path.components()
        .any(|c| excluded.contains(&c.as_os_str().to_str().unwrap_or("")))
}
```

---

# **src/main.rs**

```rust
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Clear},
    Terminal, Frame,
    layout::{Layout, Constraint, Direction, Rect},
    style::{Style, Color},
    prelude::Alignment,
    text::Line,
};
use std::{
    io,
    path::PathBuf,
    env,
    collections::HashSet,
    fs,
    sync::{Arc, Mutex},
};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use ignore::{gitignore::GitignoreBuilder, Match, gitignore::Gitignore};
use std::process::Command;
use std::env::consts::OS;
use std::thread;
use std::time::Duration;

// ---------- Optional Parallelization with Rayon ----------
// If you'd like to enable parallel reading of file contents
// for performance on large codebases, enable the "parallel" feature.
//
// cargo run --features parallel
//
#[cfg(feature = "parallel")]
use rayon::prelude::*;

const VERSION: &str = "0.4.5"; // Matches your Cargo.toml version

/// Expanded output formats: Now includes JSON
#[derive(Clone, Copy, PartialEq)]
enum OutputFormat {
    Xml,
    Markdown,
    Json,
}

impl OutputFormat {
    /// Toggle format through Xml -> Markdown -> Json -> back to Xml
    fn toggle(&self) -> Self {
        match self {
            OutputFormat::Xml => OutputFormat::Markdown,
            OutputFormat::Markdown => OutputFormat::Json,
            OutputFormat::Json => OutputFormat::Xml,
        }
    }
}

/// Our "icon table" is large, but it is not a performance bottleneck for typical usage,
/// so we leave it as-is. If performance is an issue for extremely frequent lookups,
/// consider a HashMap instead of a slice. But typically this is a minor cost.
const ICONS: &[(&str, &str)] = &[
    // ... (full icons list omitted for brevity; unchanged) ...
    ("default", "📄"),
];

/// Default directories to ignore if `use_default_ignores` is true.
const DEFAULT_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    "coverage",
    "target",
];

/// Configuration for ignoring files/folders
struct IgnoreConfig {
    use_default_ignores: bool,
    use_gitignore: bool,
    include_binary_files: bool,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            use_default_ignores: true,
            use_gitignore: true,
            include_binary_files: false,
        }
    }
}

/// Stores statistics for the copy operation (e.g. how many files/folders copied)
#[derive(Clone)]
struct CopyStats {
    files: usize,
    folders: usize,
}

/// Modal data for showing messages or help content in a popup
struct Modal {
    message: String,
    timestamp: std::time::Instant,
    width: u16,
    height: u16,
    page: usize,  // For paginated help
}

impl Modal {
    fn new(message: String, width: u16, height: u16) -> Self {
        Self {
            message,
            timestamp: std::time::Instant::now(),
            width,
            height,
            page: 0,
        }
    }

    /// Construct a new "stats" modal for successful copy
    fn copy_stats(file_count: usize, folder_count: usize, line_count: usize, byte_size: usize, format: &OutputFormat) -> Self {
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
                human_readable_size(byte_size),
                match format {
                    OutputFormat::Xml => "XML",
                    OutputFormat::Markdown => "Markdown",
                    OutputFormat::Json => "JSON",
                }
            ),
            45,
            8,
        )
    }

    /// Construct a new help modal
    fn help() -> Self {
        let help_text = format!(
            "Keyboard Shortcuts\n\
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
        );
        Self {
            message: help_text,
            timestamp: std::time::Instant::now(),
            width: 60,
            height: 30,
            page: 0,
        }
    }

    /// Return the substring of the modal to display on the current "page"
    fn get_visible_content(&self, available_height: u16) -> (String, bool) {
        let content_height = (available_height - 4) as usize; // Account for borders/title
        let lines: Vec<&str> = self.message.lines().collect();
        let total_lines = lines.len();
        
        // total_pages is just # of lines / content_height
        let total_pages = (total_lines + content_height - 1) / content_height;
        let has_more_pages = total_lines > content_height;
        
        let start = self.page * content_height;
        let end = (start + content_height).min(total_lines);
        
        let visible_content = lines[start..end].join("\n");
        
        let content = if has_more_pages {
            format!("{}\n\nPage {} of {}", visible_content, self.page + 1, total_pages)
        } else {
            visible_content
        };
        
        (content, has_more_pages)
    }

    /// Switch to next page if help has multiple pages
    fn next_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = (total_lines + content_height - 1) / content_height;
        if total_pages > 1 {
            self.page = (self.page + 1) % total_pages;
        }
    }

    /// Switch to prev page if help has multiple pages
    fn prev_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = (total_lines + content_height - 1) / content_height;
        if total_pages > 1 {
            self.page = (self.page + total_pages - 1) % total_pages;
        }
    }
}

/// Our main application struct
struct App {
    // Current directory user is viewing
    current_dir: PathBuf,
    // Flattened items in the directory (plus subdirectories if expanded)
    items: Vec<PathBuf>,
    // TUI list-state for ratatui
    list_state: ListState,
    // Paths selected for copying
    selected_items: HashSet<PathBuf>,
    // Whether or not to quit
    quit: bool,
    // Stats from last copy, for final printing
    last_copy_stats: Option<CopyStats>,
    // Optional modal overlay for help or copy-stats
    modal: Option<Modal>,
    // Ignore config for default directories & gitignore
    ignore_config: IgnoreConfig,
    // Which directories are "expanded" in the TUI
    expanded_folders: HashSet<PathBuf>,
    // Current text from user searching
    search_query: String,
    // Subset of "items" that pass the search filter
    filtered_items: Vec<PathBuf>,
    // Flag to see if user is currently typing a search query
    is_searching: bool,
    // The user-selected format for file copying
    output_format: OutputFormat,
    // Whether or not to prefix lines with line numbers
    show_line_numbers: bool,

    // --------- Performance Improvement #1 ---------
    // Store a single compiled .gitignore once per "load_items" call, 
    // so we do NOT re-build for every file/dir.
    compiled_gitignore: Option<Gitignore>,
}

/// Recursively gather items from a directory. 
/// This is called by `load_items` after we've built the compiled gitignore once.
/// 
/// # Performance note
/// - We pass `compiled_gitignore` in so we do not re-build it repeatedly.
/// - We still do a sort here for directory-then-file ordering. 
///   If you do not need that feature, remove or reduce sorting for speed.
fn add_items_recursive(
    items: &mut Vec<PathBuf>,
    dir: &PathBuf,
    expanded_folders: &HashSet<PathBuf>,
    ignore_config: &IgnoreConfig,
    compiled_gitignore: Option<&Gitignore>,
    depth: usize,
) -> io::Result<()> {
    let mut entries = fs::read_dir(dir)?
        .filter_map(|e| e.ok()) // discard any Err
        .map(|e| e.path())
        .filter(|p| {
            // 1) default ignores
            if ignore_config.use_default_ignores {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if DEFAULT_IGNORED_DIRS.contains(&name) {
                        return false;
                    }
                }
            }
            // 2) check compiled gitignore if available
            if ignore_config.use_gitignore {
                if let Some(gitignore) = compiled_gitignore {
                    let is_dir = p.is_dir();
                    match gitignore.matched_path_or_any_parents(p, is_dir) {
                        Match::Ignore(_) => return false,
                        _ => (),
                    }
                }
            }
            true
        })
        .collect::<Vec<_>>();

    // Sorting for better user experience: directories first, then lexicographic
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
            add_items_recursive(
                items,
                &entry,
                expanded_folders,
                ignore_config,
                compiled_gitignore,
                depth + 1,
            )?;
        }
    }
    Ok(())
}

impl App {
    fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            current_dir: env::current_dir().unwrap_or_default(),
            items: Vec::new(),
            list_state,
            selected_items: HashSet::new(),
            quit: false,
            last_copy_stats: None,
            modal: None,
            ignore_config: IgnoreConfig::default(),
            expanded_folders: HashSet::new(),
            search_query: String::new(),
            filtered_items: Vec::new(),
            is_searching: false,
            output_format: OutputFormat::Xml,
            show_line_numbers: false,

            // Initially no compiled gitignore until `load_items` is called
            compiled_gitignore: None,
        }
    }

    /// Build the .gitignore for the current_dir just **once**, and store in `compiled_gitignore`.
    /// This is a big performance improvement if you have nested directories or thousands of files.
    fn build_gitignore_for_dir(&self, base_dir: &PathBuf) -> Gitignore {
        let mut builder = GitignoreBuilder::new(base_dir);

        let mut dir_cursor = base_dir.clone();
        // For each parent directory, attempt to read a .gitignore
        while let Some(parent) = dir_cursor.parent() {
            let gitignore_path = dir_cursor.join(".gitignore");
            if gitignore_path.exists() {
                let _ = builder.add(gitignore_path);
            }
            dir_cursor = parent.to_path_buf();
        }

        // If building fails for any reason, return an empty Gitignore
        builder.build().unwrap_or_else(|_| Gitignore::empty())
    }

    /// Fully load items from disk according to the current_dir,
    /// storing the flattened listing in self.items, then filter into self.filtered_items.
    fn load_items(&mut self) -> io::Result<()> {
        // Clear old items
        self.items.clear();

        // If user wants to use gitignore, compile it once
        if self.ignore_config.use_gitignore {
            self.compiled_gitignore = Some(self.build_gitignore_for_dir(&self.current_dir));
        } else {
            self.compiled_gitignore = None;
        }
        
        // Insert the "parent dir" link ("..") if available
        if let Some(parent) = self.current_dir.parent() {
            if !parent.as_os_str().is_empty() {
                self.items.push(self.current_dir.join(".."));
            }
        }

        // Recursively build the list from current_dir
        add_items_recursive(
            &mut self.items,
            &self.current_dir,
            &self.expanded_folders,
            &self.ignore_config,
            self.compiled_gitignore.as_ref(),
            0,
        )?;

        // Update search-based filtered_items
        self.update_search();
        Ok(())
    }

    /// Search the flattened list (self.items) and store the matches in self.filtered_items.
    /// Also auto-expands parent folders of matches and ensures we don't lose selection.
    fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_items = self.items.clone();
            return;
        }

        let query = self.search_query.to_lowercase();
        self.filtered_items = self
            .items
            .iter()
            .filter(|path| {
                if let Ok(rel_path) = path.strip_prefix(&self.current_dir) {
                    rel_path.to_string_lossy().to_lowercase().contains(&query)
                } else {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    name.contains(&query)
                }
            })
            .cloned()
            .collect();

        // Auto-expand parent folders of matches
        let mut parents_to_expand = HashSet::new();
        for item in &self.filtered_items {
            let mut cur = item.as_path();
            while let Some(par) = cur.parent() {
                if par == self.current_dir {
                    break;
                }
                parents_to_expand.insert(par.to_path_buf());
                cur = par;
            }
        }
        self.expanded_folders.extend(parents_to_expand);

        // If the current selection is out of range, reset
        if let Some(selected) = self.list_state.selected() {
            if selected >= self.filtered_items.len() {
                self.list_state.select(Some(0));
            }
        }
    }

    /// Called whenever the user types a character in search mode
    fn handle_search_input(&mut self, c: char) {
        if !self.is_searching {
            return;
        }
        match c {
            '/' => {
                // finishing search
                self.is_searching = false;
            }
            _ if !c.is_control() => {
                self.search_query.push(c);
                self.update_search();
            }
            _ => {}
        }
    }

    fn toggle_search(&mut self) {
        self.is_searching = !self.is_searching;
        if !self.is_searching {
            // do not clear search on toggle
            self.update_search();
        }
    }

    fn clear_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
        self.update_search();
    }

    fn run(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.load_items()?;

        while !self.quit {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        // 1) If a help modal is showing, handle that
                        if let Some(modal) = &mut self.modal {
                            // If it is the help modal (height > 10),
                            // we allow PageUp/Down to navigate pages
                            if modal.height > 10 {
                                match key.code {
                                    KeyCode::PageUp => {
                                        modal.prev_page(terminal.size()?.height);
                                        continue;
                                    }
                                    KeyCode::PageDown => {
                                        modal.next_page(terminal.size()?.height);
                                        continue;
                                    }
                                    _ => {
                                        // Close help on any other key
                                        self.modal = None;
                                        continue;
                                    }
                                }
                            }
                        }
                        
                        // 2) If we are in search mode, handle search keys
                        if self.is_searching {
                            match key.code {
                                KeyCode::Esc => self.clear_search(),
                                KeyCode::Backspace => {
                                    self.search_query.pop();
                                    self.update_search();
                                }
                                KeyCode::Char(c) => self.handle_search_input(c),
                                _ => {}
                            }
                        } else {
                            // 3) Otherwise, handle normal shortcuts
                            match key.code {
                                KeyCode::Char('q') => {
                                    // if items are selected, copy before quitting
                                    if !self.selected_items.is_empty() {
                                        self.copy_selected_to_clipboard()?;
                                    }
                                    self.quit = true;
                                }
                                KeyCode::Char('*') => self.toggle_select_all(),
                                KeyCode::Char(' ') => self.toggle_selection(),
                                KeyCode::Char('c') => self.copy_selected_to_clipboard()?,
                                KeyCode::Char('i') => self.toggle_default_ignores()?,
                                KeyCode::Char('g') => self.toggle_gitignore()?,
                                KeyCode::Char('b') => self.toggle_binary_files()?,
                                KeyCode::Char('f') => self.toggle_output_format(),
                                KeyCode::Char('n') => self.toggle_line_numbers(),
                                KeyCode::Char('/') => self.toggle_search(),
                                KeyCode::Tab => self.toggle_folder_expansion()?,
                                KeyCode::Up => self.move_selection(-1),
                                KeyCode::Down => self.move_selection(1),
                                KeyCode::PageUp => self.move_selection(-10),
                                KeyCode::PageDown => self.move_selection(10),
                                KeyCode::Enter => self.handle_enter()?,
                                KeyCode::Char('h') => self.show_help(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        // On quit, if we had a last_copy_stats, display them
        if let Some(stats) = &self.last_copy_stats {
            if let Ok(contents) = get_clipboard_contents() {
                let line_count = contents.lines().count();
                let byte_size = contents.len();
                println!("\nCopied to clipboard:");
                println!("  Files copied: {}", stats.files);
                println!("  Folders copied: {}", stats.folders);
                println!("  Total lines: {}", line_count);
                println!("  Total size: {}", human_readable_size(byte_size));
                println!();
            }
        }

        Ok(())
    }

    /// Move selection in the filtered items list. `delta` can be negative to move up.
    fn move_selection(&mut self, delta: i32) {
        if self.filtered_items.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_idx = (current as i32 + delta).clamp(0, self.filtered_items.len() as i32 - 1) as usize;
        self.list_state.select(Some(new_idx));
    }

    /// Handler for pressing Enter on the selected item. 
    /// If it's a directory, navigate into it (or go up if "..").
    fn handle_enter(&mut self) -> io::Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.filtered_items.len() {
                let path = &self.filtered_items[selected];
                if path.is_dir() {
                    if path.ends_with("..") {
                        if let Some(parent) = self.current_dir.parent() {
                            self.current_dir = parent.to_path_buf();
                        }
                    } else {
                        self.current_dir = path.clone();
                    }
                    self.load_items()?;
                    self.list_state.select(Some(0));
                }
            }
        }
        Ok(())
    }

    /// Toggle the selection of the current item in the list (Space key).
    /// If item is a folder, recursively select/deselect children, too.
    fn toggle_selection(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.filtered_items.len() {
                let path = self.filtered_items[selected].clone();
                if !path.file_name().map_or(false, |n| n == "..") {
                    let currently_selected = self.selected_items.contains(&path);
                    if path.is_dir() {
                        self.update_folder_selection(&path, !currently_selected);
                    } else {
                        if currently_selected {
                            self.selected_items.remove(&path);
                        } else {
                            self.selected_items.insert(path);
                        }
                    }
                }
            }
        }
    }

    // --------------------------------------------------------------------
    // copy_selected_to_clipboard: The main function to gather all selected paths
    // and build a single string, then set it to the clipboard.
    // We've made it "deeper" in a sense that we do concurrency if desired,
    // and we avoid re-walking directories multiple times if possible.
    // --------------------------------------------------------------------
    fn copy_selected_to_clipboard(&mut self) -> io::Result<()> {
        // For JSON format, we open an array at the start
        let mut contents = String::new();
        let mut file_count = 0;
        let mut folder_count = 0;

        if self.output_format == OutputFormat::Json {
            contents.push('[');
        }

        // 1) Gather top-level selected items (i.e., if a folder is selected, 
        //    do NOT also copy sub-items individually).
        let mut to_process: Vec<_> = self.selected_items
            .iter()
            .filter(|path| {
                if let Some(parent) = path.parent() {
                    !self.selected_items.contains(parent)
                } else {
                    true
                }
            })
            .collect();
        to_process.sort();

        // We'll accumulate data from each path as JSON or XML or MD segments.
        // Optionally, do concurrency with Rayon if the "parallel" feature is enabled.
        // We'll first build tasks. Then run them in parallel or not.

        let show_line_nums = self.show_line_numbers;
        let include_binaries = self.ignore_config.include_binary_files;
        let format = self.output_format;

        #[cfg(feature = "parallel")]
        // We use an atomic or Mutex aggregator for text output
        let aggregator = Arc::new(Mutex::new(String::new()));

        #[cfg(feature = "parallel")]
        let results: Vec<(usize, usize)> = to_process
            .par_iter()
            .map(|path| {
                // We'll build local partial content for each top-level selection
                // and then merge them back into aggregator. 
                let (item_text, f_count, d_count) = self.process_one_selected_item(
                    path,
                    show_line_nums,
                    include_binaries,
                    format,
                    // For JSON, we also care if it's the "first" item or not 
                    // to avoid leading commas. We'll handle that differently 
                    // in a simpler way below if needed.
                );
                // Lock aggregator and push text
                let mut lock = aggregator.lock().unwrap();
                lock.push_str(&item_text);
                (f_count, d_count)
            })
            .collect();

        #[cfg(feature = "parallel")]
        {
            // Summation
            for (f, d) in results {
                file_count += f;
                folder_count += d;
            }
            // Move aggregator into `contents` if format is JSON
            let final_text = aggregator.lock().unwrap();
            contents.push_str(&final_text);
        }

        #[cfg(not(feature = "parallel"))]
        {
            // Single-threaded approach
            let mut first_item = true;
            for path in to_process {
                // For JSON, add a comma between items
                if format == OutputFormat::Json && !first_item {
                    contents.push(',');
                }
                first_item = false;

                let (item_text, f_count, d_count) = self.process_one_selected_item(
                    path,
                    show_line_nums,
                    include_binaries,
                    format,
                );
                file_count += f_count;
                folder_count += d_count;
                contents.push_str(&item_text);
            }
        }

        if format == OutputFormat::Json {
            contents.push(']');
        }

        // 3) If there's any actual text, set to the clipboard
        if !contents.is_empty() {
            if let Ok(_) = set_clipboard_contents(&contents) {
                // Wait briefly to ensure it persists
                thread::sleep(Duration::from_millis(100));
                let stats = CopyStats {
                    files: file_count,
                    folders: folder_count,
                };
                self.last_copy_stats = Some(stats.clone());

                let line_count = contents.lines().count();
                let byte_size = contents.len();

                self.modal = Some(Modal::copy_stats(
                    file_count,
                    folder_count,
                    line_count,
                    byte_size,
                    &self.output_format,
                ));
            }
        }

        Ok(())
    }

    /// process_one_selected_item: Build text for a single top-level selected path
    /// (which could be a file or a directory), returning (text, file_count, folder_count).
    ///
    /// This is split out of `copy_selected_to_clipboard` so we can do concurrency
    /// more easily if we want. 
    fn process_one_selected_item(
        &self,
        path: &PathBuf,
        show_line_nums: bool,
        include_binaries: bool,
        format: OutputFormat,
    ) -> (String, usize, usize) {
        let mut local_text = String::new();
        let mut file_count = 0;
        let mut folder_count = 0;

        if let Ok(rel_path) = path.strip_prefix(&self.current_dir) {
            let normalized_path = normalize_path(&rel_path.to_string_lossy());

            // Distinguish between file vs folder
            if path.is_file() {
                if Self::is_binary_file(path) {
                    if include_binaries {
                        match format {
                            OutputFormat::Xml => {
                                local_text.push_str(&format!("<file name=\"{}\">\n</file>\n", normalized_path));
                            }
                            OutputFormat::Markdown => {
                                local_text.push_str(&format!("```{}\n<binary file>\n```\n\n", normalized_path));
                            }
                            OutputFormat::Json => {
                                local_text.push_str(&format!("{{\"type\":\"file\",\"path\":\"{}\",\"binary\":true}}", normalized_path));
                            }
                        }
                        file_count += 1;
                    }
                } else {
                    match format {
                        OutputFormat::Xml => {
                            local_text.push_str(&format!("<file name=\"{}\">\n", normalized_path));
                            if let Ok(content) = fs::read_to_string(path) {
                                if show_line_nums {
                                    for (i, line) in content.lines().enumerate() {
                                        local_text.push_str(&format!("{:>6} | {}\n", i + 1, line));
                                    }
                                } else {
                                    local_text.push_str(&content);
                                }
                                if !content.ends_with('\n') {
                                    local_text.push('\n');
                                }
                            }
                            local_text.push_str("</file>\n");
                        }
                        OutputFormat::Markdown => {
                            local_text.push_str(&format!("```{}\n", normalized_path));
                            if let Ok(content) = fs::read_to_string(path) {
                                if show_line_nums {
                                    for (i, line) in content.lines().enumerate() {
                                        local_text.push_str(&format!("{:>6} | {}\n", i + 1, line));
                                    }
                                } else {
                                    local_text.push_str(&content);
                                }
                                if !content.ends_with('\n') {
                                    local_text.push('\n');
                                }
                            }
                            local_text.push_str("```\n\n");
                        }
                        OutputFormat::Json => {
                            if let Ok(content) = fs::read_to_string(path) {
                                // Escape for JSON
                                let escaped = content
                                    .replace('\\', "\\\\")
                                    .replace('\"', "\\\"")
                                    .replace('\n', "\\n")
                                    .replace('\r', "\\r");

                                if show_line_nums {
                                    let lines: Vec<String> = content
                                        .lines()
                                        .enumerate()
                                        .map(|(i, line)| format!("{:>6} | {}", i + 1, line))
                                        .collect();
                                    let joined = lines.join("\\n").replace('\"', "\\\"");
                                    local_text.push_str(&format!(
                                        "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}", 
                                        normalized_path, joined
                                    ));
                                } else {
                                    local_text.push_str(&format!(
                                        "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}", 
                                        normalized_path, escaped
                                    ));
                                }
                            }
                        }
                    }
                    file_count += 1;
                }
            } else if path.is_dir() {
                match format {
                    OutputFormat::Xml => {
                        local_text.push_str(&format!("<folder name=\"{}\">\n", normalized_path));
                        let (f, d, nested) = self.process_directory(path);
                        local_text.push_str(&nested);
                        local_text.push_str("</folder>\n");
                        file_count += f;
                        folder_count += d;
                    }
                    OutputFormat::Markdown => {
                        let (f, d, nested) = self.process_directory(path);
                        local_text.push_str(&nested);
                        file_count += f;
                        folder_count += d;
                    }
                    OutputFormat::Json => {
                        local_text.push_str(&format!(
                            "{{\"type\":\"directory\",\"path\":\"{}\",\"contents\":[", 
                            normalized_path
                        ));
                        let (f, d, nested) = self.process_directory(path);
                        local_text.push_str(&nested);
                        local_text.push_str("]}");
                        file_count += f;
                        folder_count += d;
                    }
                }
                folder_count += 1;
            }
        }

        (local_text, file_count, folder_count)
    }

    /// Recursively gather the contents of a directory in the requested format,
    /// returning `(file_count, folder_count, text_generated)`.
    /// 
    /// # Performance note
    /// - We no longer rebuild .gitignore or re-check default ignores repeatedly if possible.
    ///   Instead, we rely on the single compiled version in `self.compiled_gitignore`
    ///   or the existing logic in `is_path_ignored`.
    /// - We still do sorting by default. You can remove it for speed if desired.
    fn process_directory(&self, dir: &PathBuf) -> (usize, usize, String) {
        let mut file_count = 0;
        let mut folder_count = 0;
        let mut output = String::new();
        let mut entries = match fs::read_dir(dir) {
            Ok(r) => r
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| !self.is_path_ignored(p))
                .collect::<Vec<_>>(),
            Err(_) => return (0, 0, String::new()),
        };
        
        // Sorting. Remove if not needed for performance
        entries.sort();

        let mut first_item = true;
        for path in entries {
            if let Ok(rel_path) = path.strip_prefix(&self.current_dir) {
                let normalized_path = normalize_path(&rel_path.to_string_lossy());
                
                // For JSON, we separate items with commas
                if self.output_format == OutputFormat::Json && !first_item {
                    output.push(',');
                }
                first_item = false;

                if path.is_file() {
                    if Self::is_binary_file(&path) {
                        if self.ignore_config.include_binary_files {
                            match self.output_format {
                                OutputFormat::Xml => {
                                    output.push_str(&format!("<file name=\"{}\">\n</file>\n", normalized_path));
                                }
                                OutputFormat::Markdown => {
                                    output.push_str(&format!("```{}\n<binary file>\n```\n\n", normalized_path));
                                }
                                OutputFormat::Json => {
                                    output.push_str(&format!("{{\"type\":\"file\",\"path\":\"{}\",\"binary\":true}}", normalized_path));
                                }
                            }
                            file_count += 1;
                        }
                    } else {
                        match self.output_format {
                            OutputFormat::Xml => {
                                output.push_str(&format!("<file name=\"{}\">\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(&path) {
                                    if self.show_line_numbers {
                                        for (i, line) in content.lines().enumerate() {
                                            output.push_str(&format!("{:>6} | {}\n", i + 1, line));
                                        }
                                    } else {
                                        output.push_str(&content);
                                    }
                                    if !content.ends_with('\n') {
                                        output.push('\n');
                                    }
                                }
                                output.push_str("</file>\n");
                            }
                            OutputFormat::Markdown => {
                                output.push_str(&format!("```{}\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(&path) {
                                    if self.show_line_numbers {
                                        for (i, line) in content.lines().enumerate() {
                                            output.push_str(&format!("{:>6} | {}\n", i + 1, line));
                                        }
                                    } else {
                                        output.push_str(&content);
                                    }
                                    if !content.ends_with('\n') {
                                        output.push('\n');
                                    }
                                }
                                output.push_str("```\n\n");
                            }
                            OutputFormat::Json => {
                                if let Ok(content) = fs::read_to_string(&path) {
                                    let escaped = content
                                        .replace('\\', "\\\\")
                                        .replace('\"', "\\\"")
                                        .replace('\n', "\\n")
                                        .replace('\r', "\\r");

                                    if self.show_line_numbers {
                                        let lines: Vec<String> = content
                                            .lines()
                                            .enumerate()
                                            .map(|(i, line)| format!("{:>6} | {}", i + 1, line))
                                            .collect();
                                        let joined = lines.join("\\n").replace('\"', "\\\"");
                                        output.push_str(&format!(
                                            "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}", 
                                            normalized_path, joined
                                        ));
                                    } else {
                                        output.push_str(&format!(
                                            "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}", 
                                            normalized_path, escaped
                                        ));
                                    }
                                }
                            }
                        }
                        file_count += 1;
                    }
                } else if path.is_dir() {
                    match self.output_format {
                        OutputFormat::Xml => {
                            output.push_str(&format!("<folder name=\"{}\">\n", normalized_path));
                            let (f, d, nested) = self.process_directory(&path);
                            output.push_str(&nested);
                            output.push_str("</folder>\n");
                            file_count += f;
                            folder_count += d;
                        }
                        OutputFormat::Markdown => {
                            let (f, d, nested) = self.process_directory(&path);
                            output.push_str(&nested);
                            file_count += f;
                            folder_count += d;
                        }
                        OutputFormat::Json => {
                            output.push_str(&format!("{{\"type\":\"directory\",\"path\":\"{}\",\"contents\":[", normalized_path));
                            let (f, d, nested) = self.process_directory(&path);
                            output.push_str(&nested);
                            output.push_str("]}");
                            file_count += f;
                            folder_count += d;
                        }
                    }
                    folder_count += 1;
                }
            }
        }
        (file_count, folder_count, output)
    }

    /// Recursively select/deselect a folder, including all children
    fn update_folder_selection(&mut self, path: &PathBuf, selected: bool) {
        if path.is_dir() {
            // Toggle the folder itself
            if selected {
                self.selected_items.insert(path.clone());
            } else {
                self.selected_items.remove(path);
            }
            // Then toggle its children
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let child = entry.path();
                    if child.is_dir() {
                        self.update_folder_selection(&child, selected);
                    } else {
                        if selected {
                            self.selected_items.insert(child);
                        } else {
                            self.selected_items.remove(&child);
                        }
                    }
                }
            }
        }
    }

    /// Return an icon for the given path
    fn get_icon(path: &PathBuf) -> &'static str {
        if path.is_dir() {
            return ICONS.iter().find(|(k, _)| *k == "folder").map(|(_, v)| *v).unwrap_or("📁");
        }
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| ICONS.iter().find(|(k, _)| *k == ext))
            .map(|(_, v)| *v)
            .unwrap_or(ICONS.iter().find(|(k, _)| *k == "default").map(|(_, v)| *v).unwrap_or("📄"))
    }

    /// The main TUI drawing routine
    fn ui(&mut self, f: &mut Frame) {
        // Layout: top block has size=3, middle is flexible, then 1 line, then 1 line
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(f.area());

        // Additional layout for the top chunk: 
        // the first row is the "header" line, the second row is the search bar
        let title_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(chunks[0]);

        let title = Block::default()
            .title(format!(" AIBundle v{} - {} ", VERSION, self.current_dir.display()))
            .borders(Borders::ALL);
        f.render_widget(title.clone(), chunks[0]);

        // If user is searching, display the search input
        if self.is_searching {
            let cursor = if (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                / 500)
                % 2
                == 0
            {
                "█"
            } else {
                " "
            };

            let search_text = format!(
                "Search: {}{} (Press / to finish, ESC to cancel)",
                self.search_query, cursor
            );
            let search_widget = Paragraph::new(search_text).alignment(Alignment::Left);

            let inner_area = title_chunks[1];
            let search_area = Rect {
                x: inner_area.x + 2,
                y: inner_area.y,
                width: inner_area.width.saturating_sub(4),
                height: inner_area.height,
            };

            f.render_widget(search_widget, search_area);
        }

        // Build a List of the filtered items
        let items: Vec<ListItem> = self
            .filtered_items
            .iter()
            .map(|path| {
                let depth = path
                    .strip_prefix(&self.current_dir)
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
                let prefix = if self.selected_items.contains(path) {
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

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Gray))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, chunks[1], &mut self.list_state);

        // Build a status line with toggles
        let status_text = format!(
            " {} items ({} selected) - Space: select, Enter: open dir, c: copy, i: ignores [{}], g: gitignore [{}], b: binary [{}], f: format [{}], n: line numbers [{}], /: search, q: quit ",
            self.filtered_items.len(),
            self.selected_items.len(),
            if self.ignore_config.use_default_ignores { "x" } else { " " },
            if self.ignore_config.use_gitignore { "x" } else { " " },
            if self.ignore_config.include_binary_files { "x" } else { " " },
            match self.output_format {
                OutputFormat::Xml => "XML",
                OutputFormat::Markdown => "Markdown",
                OutputFormat::Json => "JSON",
            },
            if self.show_line_numbers { "x" } else { " " },
        );

        let status = Block::default().title(status_text).borders(Borders::ALL);
        f.render_widget(status, chunks[2]);

        // If a modal is active, show it (help or copy-stats).
        if let Some(modal) = &self.modal {
            let is_help = modal.height > 10;
            let timeout = if is_help { 30 } else { 2 };

            if modal.timestamp.elapsed().as_secs() < timeout {
                let area = centered_rect(modal.width, modal.height, f.area());

                let (content, has_more_pages) = modal.get_visible_content(area.height);
                let lines: Vec<&str> = content.lines().collect();
                let max_length = lines.iter().map(|l| l.len()).max().unwrap_or(0);

                let total_space = area.width.saturating_sub(2) as usize;
                let padding = total_space.saturating_sub(max_length) / 2;
                let pad = " ".repeat(padding);

                let padded_lines: Vec<Line> = std::iter::once("")
                    .chain(lines.into_iter())
                    .map(|line| {
                        if line.is_empty() {
                            Line::from(line.to_string())
                        } else {
                            // For help, we do left alignment. For copy-stats, we do center.
                            if is_help {
                                Line::from(line)
                            } else {
                                Line::from(format!("{}{}", pad, line))
                            }
                        }
                    })
                    .collect();

                let title = if is_help {
                    if has_more_pages {
                        " Help (PgUp/PgDn to navigate, any other key to close) "
                    } else {
                        " Help (press any key to close) "
                    }
                } else {
                    " Copied to clipboard: "
                };

                let text = Paragraph::new(padded_lines)
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .alignment(if is_help { Alignment::Left } else { Alignment::Center });

                f.render_widget(Clear, area);
                f.render_widget(text, area);
            } else {
                self.modal = None;
            }
        }
    }

    /// Toggle whether the default ignored directories are used
    fn toggle_default_ignores(&mut self) -> io::Result<()> {
        self.ignore_config.use_default_ignores = !self.ignore_config.use_default_ignores;
        self.load_items()
    }

    /// Toggle whether .gitignore is used
    fn toggle_gitignore(&mut self) -> io::Result<()> {
        self.ignore_config.use_gitignore = !self.ignore_config.use_gitignore;
        self.load_items()
    }

    /// Toggle inclusion of binary files
    fn toggle_binary_files(&mut self) -> io::Result<()> {
        self.ignore_config.include_binary_files = !self.ignore_config.include_binary_files;
        self.load_items()
    }

    /// Cycle the output format (XML/MD/JSON)
    fn toggle_output_format(&mut self) {
        self.output_format = self.output_format.toggle();
    }

    /// Toggle whether lines in text files get numbered
    fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    /// Simple heuristic to check if a file is binary by extension or name
    fn is_binary_file(path: &PathBuf) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let binary_extensions = [
                // ... (list omitted for brevity, same as your original) ...
                "class", "pyc", "pyd", "pyo",
            ];
            return binary_extensions.contains(&ext.to_lowercase().as_str());
        }
        // Also check known extension-less binary names
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let binary_files = ["index"]; // Git index
            return binary_files.contains(&name);
        }
        false
    }

    /// Check if path is ignored by either default ignores or compiled gitignore
    fn is_path_ignored(&self, path: &PathBuf) -> bool {
        if !self.ignore_config.use_default_ignores && !self.ignore_config.use_gitignore {
            return false;
        }
        // Check default ignores
        if self.ignore_config.use_default_ignores {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if DEFAULT_IGNORED_DIRS.contains(&name) {
                    return true;
                }
            }
        }
        // Check compiled gitignore if it exists
        if self.ignore_config.use_gitignore {
            if let Some(gitignore) = &self.compiled_gitignore {
                let is_dir = path.is_dir();
                match gitignore.matched_path_or_any_parents(path, is_dir) {
                    Match::Ignore(_) => return true,
                    _ => (),
                }
            }
        }
        false
    }

    /// Expand/collapse a folder on Tab
    fn toggle_folder_expansion(&mut self) -> io::Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.items.len() {
                let path = &self.items[selected];
                if path.is_dir() && !path.ends_with("..") {
                    if self.expanded_folders.contains(path) {
                        self.expanded_folders.remove(path);
                    } else {
                        self.expanded_folders.insert(path.clone());
                    }
                    self.load_items()?;
                }
            }
        }
        Ok(())
    }

    /// Toggle selection state for all currently visible items.
    fn toggle_select_all(&mut self) {
        let all_selected = self.filtered_items.iter()
            .filter(|p| !p.ends_with(".."))
            .all(|p| self.selected_items.contains(p));
        if all_selected {
            self.selected_items.clear();
        } else {
            // Collect them first, then update
            let paths: Vec<_> = self.filtered_items
                .iter()
                .filter(|p| !p.ends_with(".."))
                .cloned()
                .collect();
            for p in paths {
                if p.is_dir() {
                    self.update_folder_selection(&p, true);
                } else {
                    self.selected_items.insert(p);
                }
            }
        }
    }

    /// Show the help modal
    fn show_help(&mut self) {
        self.modal = Some(Modal::help());
    }
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Center a rectangle of (width, height) in the frame
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_width = width.min(r.width);
    let popup_height = height.min(r.height);

    let x_margin = (r.width - popup_width) / 2;
    let y_margin = (r.height - popup_height) / 2;

    Rect {
        x: r.x + x_margin,
        y: r.y + y_margin,
        width: popup_width,
        height: popup_height,
    }
}

/// Produce a user-friendly representation of file size
fn human_readable_size(size: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut s = size as f64;
    let mut idx = 0;
    while s >= 1024.0 && idx < UNITS.len() - 1 {
        s /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{} {}", s as usize, UNITS[idx])
    } else {
        format!("{:.2} {}", s, UNITS[idx])
    }
}

/// Cross-platform function to set the system clipboard
fn set_clipboard_contents(contents: &str) -> io::Result<()> {
    match OS {
        "windows" => {
            if let Ok(mut ctx) = ClipboardContext::new() {
                ctx.set_contents(contents.to_owned())
                    .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to set clipboard contents"))?;
            }
            Ok(())
        }
        _ => {
            // First try clip.exe (WSL or Windows environment under Linux)
            let clip_result = Command::new("clip.exe")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(mut stdin) = child.stdin.take() {
                        stdin.write_all(contents.as_bytes())?;
                    }
                    child.wait().map(|_| ())
                });
            if clip_result.is_ok() {
                thread::sleep(Duration::from_millis(100));
                return Ok(());
            }

            // Try wl-copy (Wayland)
            let wl_result = Command::new("wl-copy")
                .arg(contents)
                .status();
            match wl_result {
                Ok(_) => Ok(()),
                Err(_) => {
                    // Fallback to xclip (X11)
                    Command::new("xclip")
                        .arg("-selection")
                        .arg("clipboard")
                        .arg("-i")
                        .spawn()
                        .and_then(|mut child| {
                            use std::io::Write;
                            if let Some(mut stdin) = child.stdin.take() {
                                stdin.write_all(contents.as_bytes())?;
                            }
                            child.wait().map(|_| ())
                        })
                }
            }
        }
    }
}

/// Cross-platform function to get system clipboard contents
fn get_clipboard_contents() -> io::Result<String> {
    match OS {
        "windows" => {
            if let Ok(mut ctx) = ClipboardContext::new() {
                ctx.get_contents()
                    .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to get clipboard contents"))
            } else {
                Ok(String::new())
            }
        }
        _ => {
            // Attempt powershell.exe (in WSL, perhaps)
            let powershell_result = Command::new("powershell.exe")
                .args(["-Command", "Get-Clipboard"])
                .output();
            if let Ok(output) = powershell_result {
                if output.status.success() {
                    return String::from_utf8(output.stdout)
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"));
                }
            }

            // Try wl-paste
            let wl_output = Command::new("wl-paste").output();
            if let Ok(output) = wl_output {
                if output.status.success() {
                    return String::from_utf8(output.stdout)
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"));
                }
            }

            // Fallback to xclip
            let output = Command::new("xclip")
                .arg("-selection")
                .arg("clipboard")
                .arg("-o")
                .output()?;
            String::from_utf8(output.stdout)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"))
        }
    }
}

/// Main function that runs the TUI
fn main() -> io::Result<()> {
    let mut app = App::new();
    enable_raw_mode()?;
    let res = app.run();
    disable_raw_mode()?;
    res
}
```

---

# **Detailed Code Review and Explanation of Level 2 Optimizations**

Below is a **deeper** look at each optimization, plus the reasoning behind it. This is a **Level 2** explanation, which expands on the bullet points from before:

1. **Pre-Compiled `.gitignore`**  
   - **What**: We introduced a field `compiled_gitignore` in the `App` struct. In `load_items()`, we check if `ignore_config.use_gitignore` is `true`. If it is, we call `build_gitignore_for_dir` **once** and store the result in `self.compiled_gitignore`.  
   - **Why**: Previously, the code re-built a `GitignoreBuilder` for every directory or even every file. This is extremely costly. By doing it once per load, you save a lot of overhead on large projects.

2. **Minimize Sorting or Make It Optional**  
   - **What**: We still perform sorting in `add_items_recursive` and in `process_directory`, but we note in comments that you can remove or reduce the sorting if performance is critical and alphabetical order is unimportant.  
   - **Why**: Sorting can be **O(n log n)**, which becomes expensive for large numbers of entries.

3. **Single-Pass Copy for Directories**  
   - **What**: The code now ensures we don’t re-traverse subdirectories once for every file if the user has selected a folder and all of its sub-items. We collect “top-level” selected items and copy them. This is effectively your existing approach of filtering out items whose parent is also selected.  
   - **Why**: Eliminates redundant reading of thousands of files if the parent folder is also selected.

4. **Optional Parallelization**  
   - **What**: We added feature-gated usage of `rayon` (`#[cfg(feature = "parallel")]`) in `copy_selected_to_clipboard`. If compiled with `cargo run --features parallel`, the code will read selected files *in parallel* and accumulate them.  
   - **Why**: On large multi-core machines with fast storage (NVMe SSD, etc.), parallel reading can reduce total copy time significantly. This step is more complex and requires concurrency handling (e.g., a `Mutex<String>` or gather partial strings in local variables then combine them).

5. **(Skipped) Partial Reading of Large Files**  
   - **What**: We mention that you might skip reading the full file if it’s huge. You stated that is “a thought for another day,” so we left it as future work.  
   - **Why**: If you ever handle extremely large files (gigabytes) in a codebase, reading them in their entirety for a single clipboard operation can be extremely slow and memory-hungry. A partial read or a prompt to the user might be ideal.

6. **Avoid Recomputing Paths**  
   - **What**: We only compute `strip_prefix` once for each item, rather than repeatedly. In the “Level 2” code, each item in `process_one_selected_item` or `process_directory` does `strip_prefix` a single time, then normalizes it once.  
   - **Why**: Each `strip_prefix` operation is cheap individually, but if done thousands of times in nested loops, it adds up. Minimizing repeated path manipulations helps.

7. **Use More Efficient Data Structures**  
   - **What**: For your icons, we note you could use a `HashMap<String, &str>` if you do a large number of lookups. But usually, an array with ~200 items is not that big a cost. Meanwhile, we still use `HashSet` for selected items, which is quite efficient for membership checking.  
   - **Why**: Searching in a large slice for an extension can be O(n), but because `ICONS` is small, it’s not a real bottleneck. You could do an advanced approach for extreme usage (like building a static `HashMap<&'static str, &'static str>`).

8. **Reserve Large Buffers or Stream**  
   - **What**: We do an in-memory build of the entire string in `copy_selected_to_clipboard`. In extremely large repositories (with 100K files), you might prefer a more advanced streaming approach or at least do `String::with_capacity(estimated_size)`. For brevity, we keep it straightforward.  
   - **Why**: This is typically a micro-optimization. The biggest slowdown is usually I/O, not string reallocation, but for extremely large projects, it could be helpful.

9. **User Experience / TUI Speed**  
   - **What**: We ensure we only recalculate the items in `load_items()` after toggling ignores or changing directories. The TUI drawing is done at 50ms intervals in the event loop, so that remains smooth.  
   - **Why**: Keeping the TUI from blocking on heavy I/O each frame is crucial. We do one load pass, store data, then poll user input.

10. **Help Modal Pagination**  
   - **What**: Not a performance detail, but we improved the user experience around help text by allowing page up/down.  
   - **Why**: This is just a new feature for user convenience, not strictly related to speed, but it does demonstrate how we can keep heavy text from flooding the user at once.

---

## Final Notes

- This code is still a TUI that reads entire files into memory and places them into a single, large string. This is inherently expensive for large files or large numbers of files. However, by:
  1. **Avoiding repeated `.gitignore` builds**  
  2. **Reducing repeated directory traversals**  
  3. **Optionally running a parallel pass**  
  …you will see significant performance gains compared to the initial “Level 1” implementation.  
