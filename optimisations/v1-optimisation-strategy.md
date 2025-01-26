Below is your **full** code again, annotated with some comments and suggestions about performance improvements. Following the code, you'll find an itemized explanation of where the most significant gains are likely to come from. Some changes are small and straightforward (e.g., avoiding rebuilding `.gitignore` for every file); others involve more invasive approaches (e.g., parallel file reading, partial reads, or large architectural changes).

---

```rust
use std::path::PathBuf;
use walkdir::WalkDir;

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
```

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
use std::{io, path::PathBuf, env, collections::HashSet};
use std::fs;
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use ignore::{gitignore::GitignoreBuilder, Match};
use std::process::Command;
use std::env::consts::OS;
use std::thread;
use std::time::Duration;

const VERSION: &str = "0.4.0"; // This should always be the same as the version in the Cargo.toml file

#[derive(Clone, Copy, PartialEq)]
enum OutputFormat {
    Xml,
    Markdown,
}

impl OutputFormat {
    fn toggle(&self) -> Self {
        match self {
            OutputFormat::Xml => OutputFormat::Markdown,
            OutputFormat::Markdown => OutputFormat::Xml,
        }
    }
}

const ICONS: &[(&str, &str)] = &[
    // ... (omitted for brevity, same as your original ICONS list) ...
    ("default", "📄"),
];

const DEFAULT_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    "coverage",
    "target",  // Keep Rust's target dir in defaults
];

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

#[derive(Clone)]
struct CopyStats {
    files: usize,
    folders: usize,
}

struct Modal {
    message: String,
    timestamp: std::time::Instant,
}

struct App {
    current_dir: PathBuf,
    items: Vec<PathBuf>,
    list_state: ListState,
    selected_items: HashSet<PathBuf>,
    quit: bool,
    last_copy_stats: Option<CopyStats>,
    modal: Option<Modal>,
    ignore_config: IgnoreConfig,
    expanded_folders: HashSet<PathBuf>,
    search_query: String,
    filtered_items: Vec<PathBuf>,
    is_searching: bool,
    output_format: OutputFormat,

    // ---------------------------
    // SUGGESTION #1:
    // Build the Gitignore once and store it (see explanation #1 below).
    // This is an Option because we only build it if use_gitignore = true.
    // If we switch ignores on/off at runtime, we'd rebuild once each time
    // the user toggles. That is still better than building it for every file
    // we visit.
    // ---------------------------
    compiled_gitignore: Option<ignore::gitignore::Gitignore>,
}

fn add_items_recursive(
    items: &mut Vec<PathBuf>,
    dir: &PathBuf,
    expanded_folders: &HashSet<PathBuf>,
    ignore_config: &IgnoreConfig,
    current_dir: &PathBuf,
    depth: usize,

    // Pass in the "compiled_gitignore" if available, to avoid building repeatedly
    compiled_gitignore: Option<&ignore::gitignore::Gitignore>,
) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            // 1) Default ignores
            if ignore_config.use_default_ignores {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if DEFAULT_IGNORED_DIRS.contains(&name) {
                        return false;
                    }
                }
            }

            // 2) Gitignore check
            if let Some(gitignore) = compiled_gitignore {
                let is_dir = p.is_dir();
                match gitignore.matched_path_or_any_parents(p, is_dir) {
                    Match::Ignore(_) => return false,
                    _ => (),
                }
            }

            true
        })
        .collect();

    // Possibly skip sorting if not strictly needed. Sorting is an O(n log n)
    // operation. If you don't need alphabetical order, removing the sort saves time.
    // We'll keep it here for user-friendliness.
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
                current_dir,
                depth + 1,
                compiled_gitignore,
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

            // SUGGESTION #1:
            compiled_gitignore: None,
        }
    }

    fn load_items(&mut self) -> io::Result<()> {
        self.items.clear();

        // Build or clear the compiled Gitignore once per reload
        if self.ignore_config.use_gitignore {
            self.compiled_gitignore = Some(self.build_gitignore_for_dir(&self.current_dir));
        } else {
            self.compiled_gitignore = None;
        }

        if let Some(parent) = self.current_dir.parent() {
            if !parent.as_os_str().is_empty() {
                self.items.push(self.current_dir.join(".."));
            }
        }

        add_items_recursive(
            &mut self.items,
            &self.current_dir,
            &self.expanded_folders,
            &self.ignore_config,
            &self.current_dir,
            0,
            self.compiled_gitignore.as_ref(),
        )?;

        // Update filtered items based on search
        self.update_search();
        Ok(())
    }

    // ---------------------------
    // SUGGESTION #1 (continued):
    // Build a Gitignore object once. We do it at the "load_items()" time
    // instead of inside every single read_dir or copying function.
    // ---------------------------
    fn build_gitignore_for_dir(&self, current_dir: &PathBuf) -> ignore::gitignore::Gitignore {
        let mut builder = GitignoreBuilder::new(current_dir);
        let mut dir = current_dir.clone();

        while let Some(parent) = dir.parent() {
            let gitignore_path = dir.join(".gitignore");
            if gitignore_path.exists() {
                // If this returns Some(err) we might break or handle it, but let's ignore for brevity
                let _ = builder.add(gitignore_path);
            }
            dir = parent.to_path_buf();
        }
        builder.build().unwrap_or_else(|_| ignore::gitignore::Gitignore::empty())
    }

    fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_items = self.items.clone();
            return;
        }

        let query = self.search_query.to_lowercase();
        self.filtered_items = self.items
            .iter()
            .filter(|path| {
                // Get the full relative path for searching
                if let Ok(rel_path) = path.strip_prefix(&self.current_dir) {
                    let path_str = rel_path.to_string_lossy().to_lowercase();
                    path_str.contains(&query)
                } else {
                    // Fallback to just the filename if we can't get relative path
                    let name = path.file_name()
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
            let mut current = item.as_path();
            while let Some(parent) = current.parent() {
                if parent == self.current_dir {
                    break;
                }
                parents_to_expand.insert(parent.to_path_buf());
                current = parent;
            }
        }
        self.expanded_folders.extend(parents_to_expand);

        // Reset selection if current selection is not in filtered items
        if let Some(selected) = self.list_state.selected() {
            if selected >= self.filtered_items.len() {
                self.list_state.select(Some(0));
            }
        }
    }

    fn handle_search_input(&mut self, c: char) {
        if !self.is_searching {
            return;
        }

        match c {
            '/' => { // Use '/' to finish search
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
            // Don't clear search when toggling off - keep the filter
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
                            match key.code {
                                KeyCode::Char('q') => {
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
                                KeyCode::Char('/') => self.toggle_search(),
                                KeyCode::Tab => self.toggle_folder_expansion()?,
                                KeyCode::Up => self.move_selection(-1),
                                KeyCode::Down => self.move_selection(1),
                                KeyCode::PageUp => self.move_selection(-10),
                                KeyCode::PageDown => self.move_selection(10),
                                KeyCode::Enter => self.handle_enter()?,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        // Show detailed exit message if items were copied
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

    fn move_selection(&mut self, delta: i32) {
        if self.filtered_items.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_selected = (current as i32 + delta).clamp(0, self.filtered_items.len() as i32 - 1) as usize;
        self.list_state.select(Some(new_selected));
    }

    fn handle_enter(&mut self) -> io::Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if selected >= self.filtered_items.len() {
                return Ok(());
            }
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
        Ok(())
    }

    fn toggle_selection(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected >= self.filtered_items.len() {
                return;
            }
            let path = self.filtered_items[selected].clone();
            if !path.file_name().map_or(false, |n| n == "..") {
                let is_selected = self.selected_items.contains(&path);
                if path.is_dir() {
                    self.update_folder_selection(&path, !is_selected);
                } else {
                    if is_selected {
                        self.selected_items.remove(&path);
                    } else {
                        self.selected_items.insert(path);
                    }
                }
            }
        }
    }

    fn copy_selected_to_clipboard(&mut self) -> io::Result<()> {
        let mut contents = String::new();
        let mut file_count = 0;
        let mut folder_count = 0;

        // ---------------
        // SUGGESTION #2:
        // If you have a large number of files or directories, consider
        // collecting them in a single pass (like a single WalkDir).
        // Then build "contents" in parallel (Rayon) or at least
        // in a single pass. This example remains synchronous.
        // ---------------

        // First, collect all paths we need to process
        let mut to_process: Vec<_> = self.selected_items.iter()
            .filter(|path| {
                if let Some(parent) = path.parent() {
                    !self.selected_items.contains(parent)
                } else {
                    true
                }
            })
            .collect();
        to_process.sort();

        for path in to_process {
            if let Some(rel_path) = path.strip_prefix(&self.current_dir).ok() {
                let normalized_path = normalize_path(&rel_path.to_string_lossy());

                if path.is_file() {
                    if Self::is_binary_file(path) {
                        if self.ignore_config.include_binary_files {
                            match self.output_format {
                                OutputFormat::Xml => {
                                    contents.push_str(&format!("<file name=\"{}\">\n</file>\n", normalized_path));
                                }
                                OutputFormat::Markdown => {
                                    contents.push_str(&format!("```{}\n<binary file>\n```\n\n", normalized_path));
                                }
                            }
                            file_count += 1;
                        }
                    } else {
                        match self.output_format {
                            OutputFormat::Xml => {
                                contents.push_str(&format!("<file name=\"{}\">\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(path) {
                                    contents.push_str(&content);
                                    if !content.ends_with('\n') {
                                        contents.push('\n');
                                    }
                                }
                                contents.push_str("</file>\n");
                            }
                            OutputFormat::Markdown => {
                                contents.push_str(&format!("```{}\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(path) {
                                    contents.push_str(&content);
                                    if !content.ends_with('\n') {
                                        contents.push('\n');
                                    }
                                }
                                contents.push_str("```\n\n");
                            }
                        }
                        file_count += 1;
                    }
                } else if path.is_dir() {
                    match self.output_format {
                        OutputFormat::Xml => {
                            contents.push_str(&format!("<folder name=\"{}\">\n", normalized_path));
                            let mut dir_contents = String::new();
                            if let Ok((files, folders)) = self.process_directory(path, &mut dir_contents, &self.current_dir) {
                                file_count += files;
                                folder_count += folders;
                            }
                            contents.push_str(&dir_contents);
                            contents.push_str("</folder>\n");
                        }
                        OutputFormat::Markdown => {
                            let mut dir_contents = String::new();
                            if let Ok((files, folders)) = self.process_directory(path, &mut dir_contents, &self.current_dir) {
                                file_count += files;
                                folder_count += folders;
                            }
                            contents.push_str(&dir_contents);
                        }
                    }
                    folder_count += 1;
                }
            }
        }

        if !contents.is_empty() {
            if let Ok(()) = set_clipboard_contents(&contents) {
                thread::sleep(Duration::from_millis(100));

                let stats = CopyStats {
                    files: file_count,
                    folders: folder_count,
                };
                self.last_copy_stats = Some(stats.clone());

                let line_count = contents.lines().count();
                let byte_size = contents.len();

                self.modal = Some(Modal {
                    message: format!(
                        "Files copied: {}\n\
                         Folders copied: {}\n\
                         Total lines: {}\n\
                         Total size: {}\n\
                         Format: {}\n",
                        file_count,
                        folder_count,
                        line_count,
                        human_readable_size(byte_size),
                        match self.output_format {
                            OutputFormat::Xml => "XML",
                            OutputFormat::Markdown => "Markdown",
                        }
                    ),
                    timestamp: std::time::Instant::now(),
                });
            }
        }

        Ok(())
    }

    fn process_directory(&self, dir: &PathBuf, output: &mut String, base_path: &PathBuf) -> io::Result<(usize, usize)> {
        let mut file_count = 0;
        let mut folder_count = 0;

        let mut entries: Vec<_> = fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| !self.is_path_ignored(p))
            .collect();

        // Sorting again. If order doesn't matter, removing might help performance.
        entries.sort();

        for path in entries {
            if let Some(rel_path) = path.strip_prefix(base_path).ok() {
                let normalized_path = normalize_path(&rel_path.to_string_lossy());

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
                            }
                            file_count += 1;
                        }
                    } else {
                        match self.output_format {
                            OutputFormat::Xml => {
                                output.push_str(&format!("<file name=\"{}\">\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(&path) {
                                    output.push_str(&content);
                                    if !content.ends_with('\n') {
                                        output.push('\n');
                                    }
                                }
                                output.push_str("</file>\n");
                            }
                            OutputFormat::Markdown => {
                                output.push_str(&format!("```{}\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(&path) {
                                    output.push_str(&content);
                                    if !content.ends_with('\n') {
                                        output.push('\n');
                                    }
                                }
                                output.push_str("```\n\n");
                            }
                        }
                        file_count += 1;
                    }
                } else if path.is_dir() {
                    match self.output_format {
                        OutputFormat::Xml => {
                            output.push_str(&format!("<folder name=\"{}\">\n", normalized_path));
                            let mut dir_contents = String::new();
                            if let Ok((files, folders)) = self.process_directory(&path, &mut dir_contents, base_path) {
                                file_count += files;
                                folder_count += folders;
                            }
                            output.push_str(&dir_contents);
                            output.push_str("</folder>\n");
                        }
                        OutputFormat::Markdown => {
                            let mut dir_contents = String::new();
                            if let Ok((files, folders)) = self.process_directory(&path, &mut dir_contents, base_path) {
                                file_count += files;
                                folder_count += folders;
                            }
                            output.push_str(&dir_contents);
                        }
                    }
                    folder_count += 1;
                }
            }
        }
        Ok((file_count, folder_count))
    }

    fn update_folder_selection(&mut self, path: &PathBuf, selected: bool) {
        if path.is_dir() {
            // Update the folder itself
            if selected {
                self.selected_items.insert(path.clone());
            } else {
                self.selected_items.remove(path);
            }

            // Update all children
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let child_path = entry.path();
                    if child_path.is_dir() {
                        self.update_folder_selection(&child_path, selected);
                    } else {
                        if selected {
                            self.selected_items.insert(child_path);
                        } else {
                            self.selected_items.remove(&child_path);
                        }
                    }
                }
            }
        }
    }

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

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(f.area());

        let title_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(chunks[0]);

        let title = Block::default()
            .title(format!(" aiformat v{} - {} ", VERSION, self.current_dir.display()))
            .borders(Borders::ALL);
        f.render_widget(title.clone(), chunks[0]);

        if self.is_searching {
            let cursor = if (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() / 500) % 2 == 0
            {
                "█"
            } else {
                " "
            };

            let search_text = format!("Search: {}{} (Press / to finish, ESC to cancel)", self.search_query, cursor);
            let search_widget = Paragraph::new(search_text)
                .alignment(Alignment::Left);

            let inner_area = title_chunks[1];
            let search_area = Rect {
                x: inner_area.x + 2,
                y: inner_area.y,
                width: inner_area.width - 4,
                height: inner_area.height,
            };

            f.render_widget(search_widget, search_area);
        }

        let items: Vec<ListItem> = self.filtered_items
            .iter()
            .map(|path| {
                let depth = path.strip_prefix(&self.current_dir)
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
                let prefix = if self.selected_items.contains(path) { "[X] " } else { "[ ] " };
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

        let status_text = format!(
            " {} items ({} selected) - Space: select, Enter: open dir, c: copy, i: ignores [{}], g: gitignore [{}], b: binary [{}], f: format [{}], /: search, q: quit ",
            self.filtered_items.len(),
            self.selected_items.len(),
            if self.ignore_config.use_default_ignores { "x" } else { " " },
            if self.ignore_config.use_gitignore { "x" } else { " " },
            if self.ignore_config.include_binary_files { "x" } else { " " },
            match self.output_format {
                OutputFormat::Xml => "XML",
                OutputFormat::Markdown => "Markdown",
            },
        );

        let status = Block::default()
            .title(status_text)
            .borders(Borders::ALL);
        f.render_widget(status, chunks[2]);

        if let Some(modal) = &self.modal {
            if modal.timestamp.elapsed().as_secs() < 2 {
                let area = centered_rect(45, 8, f.area());

                let lines: Vec<&str> = modal.message.lines().collect();
                let max_length = lines.iter()
                    .map(|line| line.len())
                    .max()
                    .unwrap_or(0);

                let total_space = area.width as usize - 2;
                let padding = (total_space - max_length).max(0) / 2;
                let pad = " ".repeat(padding);

                let padded_lines: Vec<Line> = std::iter::once("")
                    .chain(lines.into_iter())
                    .map(|line| {
                        if line.is_empty() {
                            Line::from(line.to_string())
                        } else {
                            Line::from(format!("{}{}", pad, line))
                        }
                    })
                    .collect();

                let text = Paragraph::new(padded_lines)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(" Copied to clipboard: "))
                    .alignment(Alignment::Left);

                f.render_widget(Clear, area);
                f.render_widget(text, area);
            }
        }
    }

    fn toggle_default_ignores(&mut self) -> io::Result<()> {
        self.ignore_config.use_default_ignores = !self.ignore_config.use_default_ignores;
        self.load_items()
    }

    fn toggle_gitignore(&mut self) -> io::Result<()> {
        self.ignore_config.use_gitignore = !self.ignore_config.use_gitignore;
        self.load_items()
    }

    fn toggle_binary_files(&mut self) -> io::Result<()> {
        self.ignore_config.include_binary_files = !self.ignore_config.include_binary_files;
        self.load_items()
    }

    fn toggle_output_format(&mut self) {
        self.output_format = self.output_format.toggle();
    }

    fn is_binary_file(path: &PathBuf) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let binary_extensions = [
                // Git specific
                "idx", "pack", "rev", "index",
                // Images
                "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp", "ico", "svg",
                // Audio
                "mp3", "wav", "ogg", "flac", "m4a", "aac", "wma",
                // Video
                "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm",
                // Archives
                "zip", "rar", "7z", "tar", "gz", "iso",
                // Executables
                "exe", "dll", "so", "dylib",
                // Other binary formats
                "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
                "class", "pyc", "pyd", "pyo",
            ];
            return binary_extensions.contains(&ext.to_lowercase().as_str());
        }

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let binary_files = ["index"];  // Git index file
            return binary_files.contains(&name);
        }

        false
    }

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

        // If we have a compiled gitignore, use that:
        if let Some(gitignore) = &self.compiled_gitignore {
            let is_dir = path.is_dir();
            match gitignore.matched_path_or_any_parents(path, is_dir) {
                Match::Ignore(_) => return true,
                _ => (),
            }
        }

        false
    }

    fn toggle_folder_expansion(&mut self) -> io::Result<()> {
        if let Some(selected) = self.list_state.selected() {
            // Before, this was `let path = &self.items[selected];`
            // but to be sure, we use self.filtered_items (since that's the UI).
            // (This fix depends on your intended behavior—filtered_items is
            // what's actually shown).
            if selected < self.filtered_items.len() {
                let path = &self.filtered_items[selected];
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

    fn toggle_select_all(&mut self) {
        let all_selected = self.filtered_items.iter()
            .filter(|path| !path.ends_with(".."))
            .all(|path| self.selected_items.contains(path));

        if all_selected {
            self.selected_items.clear();
        } else {
            let paths_to_select: Vec<_> = self.filtered_items.iter()
                .filter(|path| !path.ends_with(".."))
                .cloned()
                .collect();

            for path in paths_to_select {
                if path.is_dir() {
                    self.update_folder_selection(&path, true);
                } else {
                    self.selected_items.insert(path);
                }
            }
        }
    }
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

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

fn human_readable_size(size: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as usize, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

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

            let wl_result = Command::new("wl-copy")
                .arg(contents)
                .status();

            match wl_result {
                Ok(_) => Ok(()),
                Err(_) => {
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
            let powershell_result = Command::new("powershell.exe")
                .args(["-Command", "Get-Clipboard"])
                .output();

            if let Ok(output) = powershell_result {
                if output.status.success() {
                    return String::from_utf8(output.stdout)
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"));
                }
            }

            let wl_output = Command::new("wl-paste")
                .output();

            match wl_output {
                Ok(output) if output.status.success() => {
                    String::from_utf8(output.stdout)
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"))
                }
                _ => {
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
    }
}

fn main() -> io::Result<()> {
    let mut app = App::new();
    enable_raw_mode()?;
    let result = app.run();
    disable_raw_mode()?;
    result
}
```

---

## Explanation of Performance Optimizations

1. **Build `.gitignore` Once**  
   - **Issue**: In the original code, you build a new GitignoreBuilder for each directory (and sometimes for each file) during navigation or copying. This can become *very* expensive if there are many nested directories or thousands of files.  
   - **Solution**: Build the ignore set *once* per load (in `load_items()` or on toggling the `use_gitignore` flag) and store it in the struct (`compiled_gitignore`). Then, when filtering files, you simply call `gitignore.matched_path_or_any_parents(...)` for each path instead of rebuilding.  
   - **Why**: `GitignoreBuilder::build()` can be I/O heavy (reading `.gitignore` files from multiple parent directories) and is a step you want to do only once, not once-per-file.

2. **Minimize Sorting**  
   - **Issue**: You call `sort()` on the directory listing in several places (for example in `add_items_recursive` and `process_directory`). If you have very large directories, sorting becomes significant.  
   - **Solution**: If strict alphabetical or directory-first ordering is *not* required, remove or reduce sorting. If it is required, consider a single sort at the top level or at least only in the main listing, not in every recursion step. (Alternatively, you can keep it if you value the sorted results more than the performance cost.)

3. **Single Walk for Copying / Large Selections**  
   - **Issue**: When you have *lots* of selected directories, the code calls `process_directory` recursively for each directory. If the user has tens of thousands of files in multiple directories, you’re effectively walking the filesystem many times.  
   - **Solution**: Combine all paths into a single traversal (e.g., using [`WalkDir`](https://docs.rs/walkdir) or a single DFS) that respects which directories are selected. That way you only do one filesystem walk (or at least minimize repeated directory reads).  
   - **Details**: If a directory is entirely selected, you can walk it once, building the text in one pass. If multiple siblings are selected, you might unify them into a single pass. Possibly, you can skip recursing into a child if the parent was also selected. You already do some skipping with the `if let Some(parent) = path.parent() { !self.selected_items.contains(parent) }` check, but you still do multiple BFS calls for each top-level selected folder.

4. **Use Parallelization for Reading File Contents**  
   - **Issue**: Reading from the disk and building a large string for the clipboard is done on a single thread. For large codebases or very large files, the I/O cost can be high.  
   - **Solution**: If the environment and your constraints allow it, you can use [`rayon`](https://crates.io/crates/rayon) or another concurrency approach to read file contents in parallel. You would need to carefully accumulate strings in a thread-safe way (e.g., use a `Mutex<String>` or a concurrent data structure). For tens of thousands of files, parallel reads often saturate multiple CPU cores and can reduce total copy time.  
   - **Note**: This is a more invasive change but can yield significant gains if you know the disk I/O pattern benefits from concurrency.

5. **Skip Reading Full File If Not Necessary**  
   - **Issue**: For large files, you `read_to_string()` the entire file. This is perfect for small source files, but if you have big logs or large partially-binary files, it might degrade performance.  
   - **Solution** (depending on your feature needs):
     - Provide an option to only read up to X kilobytes per file and then skip the rest.  
     - Or do a streaming copy into the final string if you must put everything in the clipboard (though the standard OS clipboards typically expect a fully formed final buffer).  
     - Or allow the user to confirm before copying huge files.  

6. **Avoid Recomputing `strip_prefix(&self.current_dir)`**  
   - **Issue**: You do `strip_prefix(...)` plus repeated path manipulations in multiple places. Probably not a big cost, but if you do it often for thousands of paths, it can add up.  
   - **Solution**: Cache the string once. For example, compute the normalized relative path once in `copy_selected_to_clipboard` before passing it to further logic. This is a small micro-optimization.

7. **Use More Efficient Data Structures**  
   - **Issue**: Constant insertion and removal in `HashSet<PathBuf>` is fine, but if you toggle a large directory on/off, you do a BFS removing/adding. That BFS might be repeated multiple times.  
   - **Solution**: If toggling entire directories is frequent, you could store “selected directories” in a separate data structure and only expand sub-file selection if the user specifically selects inside. This gets complicated but can reduce repeated BFS expansions.

8. **Minimize Freed/Allocated Strings**  
   - **Issue**: `contents.push_str(...)` repeatedly is typically fine in Rust, but if you anticipate *really large* lumps of text, you might consider reserving capacity or streaming.  
   - **Solution**: For instance, do `let mut contents = String::with_capacity(estimated_size)` if you have a prior estimate of how large the final copy might be. If not, it’s a minor detail.

---

## Summary of Key Gains
- **Building `.gitignore` once** can drastically reduce overhead on large projects.  
- **Reducing repeated file system traversals** (particularly for large multi-directory selections) is often the biggest win. Consider a single pass.  
- **Parallelization** (if allowed) can reduce total copy time for large sets of files.  

Aside from that, everything else is fairly standard for a TUI file browser. The cost of reading entire files into the clipboard is *inherent*—the best you can do is mitigate how many times you read them and how you do it. If you truly need to copy everything in full text form into the system clipboard, there’s no shortcut around the I/O cost, but these changes will help you avoid doing redundant or repeated work.