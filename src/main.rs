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

const VERSION: &str = "0.4.5"; // This should always be the same as the version in the Cargo.toml file

#[derive(Clone, Copy, PartialEq)]
enum OutputFormat {
    Xml,
    Markdown,
    Json,
}

impl OutputFormat {
    fn toggle(&self) -> Self {
        match self {
            OutputFormat::Xml => OutputFormat::Markdown,
            OutputFormat::Markdown => OutputFormat::Json,
            OutputFormat::Json => OutputFormat::Xml,
        }
    }
}

const ICONS: &[(&str, &str)] = &[
// Folders
("folder", "📁"),
("folder_open", "📂"),
("archive_folder", "🗄️"), // For archive or special folders

// Development
("rs", "🦀"), // Rust
("ts", "💠"), // TypeScript
("js", "📜"), // JavaScript
("py", "🐍"), // Python
("java", "☕"), // Java
("class", "☕"), // Java Class Files
("cpp", "⚡"), // C++
("c", "🔌"), // C
("h", "📐"), // C Header
("hpp", "📐"), // C++ Header
("go", "🐹"), // Go
("rb", "💎"), // Ruby
("php", "🐘"), // PHP
("scala", "💫"), // Scala
("swift", "🕊️"), // Swift
("kotlin", "🎯"), // Kotlin
("dart", "🦋"), // Dart
("lua", "🌙"), // Lua
("sh", "🐚"), // Shell Script
("pl", "🔮"), // Perl
("r", "📊"), // R
("erl", "🐘"), // Erlang
("hs", "☯️"), // Haskell
("sql", "🗃️"), // SQL
("m", "🔧"), // Objective-C / MATLAB (context-dependent)
("adb", "📱"), // Android Debug Bridge Scripts
("gradle", "🛠️"), // Gradle Build Scripts
("pom", "📦"), // Maven POM Files
("sbt", "📚"), // Scala Build Tool
("makefile", "🛠️"), // Makefile
("Dockerfile", "🐳"), // Dockerfile
("Vagrantfile", "🔧"), // Vagrant Configuration
("gulpfile", "🛠️"), // Gulp Build Scripts
("webpack.config", "🔗"), // Webpack Configuration

// Web
("html", "🌐"),
("htm", "🌐"),
("css", "🎨"),
("scss", "🎨"),
("sass", "🎨"),
("less", "🎨"),
("jsx", "⚛️"),
("tsx", "⚛️"),
("vue", "💚"),
("svelte", "🔥"),
("twig", "🧵"), // Twig Templates
("asp", "🖥️"),
("aspx", "🖥️"),
("jsp", "🖥️"),
("erb", "🌸"), // Embedded Ruby
("handlebars", "🪡"), // Handlebars Templates
("ejs", "🖥️"), // Embedded JavaScript
("phphtml", "🐘"), // PHP Embedded in HTML
("jade", "🌿"), // Jade/Pug Templates

// Data
("json", "📋"),
("yaml", "⚙️"),
("yml", "⚙️"),
("xml", "📰"),
("csv", "📊"),
("sql", "🗃️"),
("db", "🗃️"),
("sqlite", "🗃️"),
("dbml", "🗂️"), // Database Markup Language
("geojson", "🗺️"),
("parquet", "📦"),
("pickle", "🥒"),
("protobuf", "🔗"), // Protocol Buffers
("avsc", "🗃️"), // Avro Schema
("ndjson", "📄"), // Newline Delimited JSON
("hdf5", "📁"), // Hierarchical Data Format

// Config
("toml", "⚙️"),
("ini", "⚙️"),
("conf", "⚙️"),
("config", "⚙️"),
("env", "🔒"),
("dockerignore", "🐳"),
("gitignore", "🔰"),
("eslint", "🔍"),
("prettierrc", "✨"),
("babelrc", "🔄"),
("tsconfig", "💠"),
("webpack.config", "🔗"),
("babel.config", "🔄"),
("package.json", "📦"),
("composer.json", "📦"),
("requirements.txt", "📄"),
("Pipfile", "📦"),
("Cargo.toml", "🦀"),
("Makefile", "🛠️"),

// Documents
("md", "📝"),
("txt", "📄"),
("pdf", "📕"),
("doc", "📘"),
("docx", "📘"),
("rtf", "📝"),
("odt", "📝"),
("tex", "📜"),
("rst", "📑"),
("asciidoc", "📖"),
("mmd", "📜"), // MultiMarkdown
("epub", "📚"),
("djvu", "📖"),

    // Spreadsheets
("xls", "📗"),
("xlsx", "📗"),
("ods", "📗"),
("csv", "📊"),
("tsv", "📈"),
("numbers", "📊"), // Apple Numbers
("gsheet", "📊"), // Google Sheets

// Presentations
("ppt", "📙"),
("pptx", "📙"),
("odp", "📙"),
("key", "🔑"), // Apple Keynote
("mdx", "📝"), // Markdown with JSX
("sldx", "📊"), // Slide files
("gslides", "📊"), // Google Slides

// Images
("png", "🖼️"),
("jpg", "🖼️"),
("jpeg", "🖼️"),
("gif", "🎥"),
("svg", "🎨"),
("ico", "🎴"),
("bmp", "🖼️"),
("tiff", "🖼️"),
("webp", "🖼️"),
("heic", "📷"),
("raw", "📸"),
("psd", "🖌️"), // Photoshop
("ai", "🖍️"), // Adobe Illustrator
("indd", "📄"), // Adobe InDesign
("xcf", "🎨"), // GIMP
("eps", "📐"), // Encapsulated PostScript
("drw", "🖊️"), // Generic Drawing
("dxf", "📏"), // Drawing Exchange Format

// Audio
("mp3", "🎵"),
("wav", "🎵"),
("ogg", "🎵"),
("flac", "🎼"),
("m4a", "🎵"),
("aac", "🎶"),
("wma", "🎧"),
("alac", "🎹"),
("opus", "🎧"),
("mid", "🎹"), // MIDI Files
("midi", "🎹"),
("aiff", "🎤"),

    // Video
("mp4", "🎬"),
("avi", "🎬"),
("mkv", "🎬"),
("mov", "🎬"),
("wmv", "🎬"),
("flv", "📹"),
("webm", "🌐"),
("m4v", "📺"),
("3gp", "📱"),
("mpeg", "📽️"),
("mpg", "📽️"),
("rmvb", "🗂️"), // RealMedia Variable Bitrate
("vob", "📀"), // DVD Video Object
("m2ts", "🎞️"),

    // Archives
("zip", "📦"),
("rar", "📦"),
("7z", "📦"),
("tar", "📦"),
("gz", "📦"),
("iso", "💿"),
("bz2", "📦"),
("xz", "📦"),
("dmg", "🖥️"),
("tgz", "📦"),
("lzma", "📦"),
("cab", "📦"),
("arj", "📦"),
("ace", "📦"),

    // Git
("git", "🔰"),
("gitignore", "🔰"),
("gitattributes", "🔰"),
("gitmodules", "🔰"),
("gitconfig", "🔧"),
("gitlab-ci.yml", "🤖"), // GitLab CI Configuration
("circleci", "🔄"), // CircleCI Config
("travis.yml", "📈"), // Travis CI Config

    // Logs
("log", "📋"),
("trace", "🔍"),
("out", "📤"),
("err", "❌"),
("debug", "🐞"),
("access", "🔓"),
("server", "🌐"),
("audit", "🔒"),

    // Shell
("sh", "🐚"),
("bash", "🐚"),
("zsh", "🐚"),
("fish", "🐟"),
("ps1", "💻"),
("bat", "🖥️"), // Old Windows DOS icon
("cmd", "🖥️"), // Old Windows CMD icon
("ksh", "🐚"),
("csh", "🐚"),
("tcsh", "🐚"),

    // Lock files
("lock", "🔒"),
("pem", "🔑"),
("key", "🔑"),
("crt", "🔒"),
("p12", "🔐"), // PKCS#12 Certificate
("pfx", "🔐"),

    // Executables
("exe", "💻"),
("bin", "📦"),
("app", "📱"),
("msi", "📥"),
("apk", "📱"),
("deb", "📦"),
("rpm", "📦"),
("run", "🔄"),
("sh", "🐚"),
("out", "📤"),
("dll", "🗄️"), // Dynamic Link Library
("so", "🗄️"), // Shared Object
("dylib", "🗄️"), // macOS Dynamic Library

    // Scripts
("ps1", "💻"),
("lua", "🌙"),
("vbs", "💾"),
("coffee", "☕"),
("groovy", "🌿"),
("rb", "💎"),
("rake", "🔨"),
("perl", "🔮"),
("tcl", "🔧"),
("awk", "✂️"),
("sed", "📝"),

    // Fonts
("ttf", "🔤"),
("otf", "🔤"),
("woff", "🔡"),
("woff2", "🔡"),
("eot", "🔡"),
("sfnt", "🔤"),
("fon", "🔠"),
("pfm", "🔠"),
("afm", "🔠"),

    // Miscellaneous
("ipynb", "📓"), // Jupyter Notebook
("vuepress", "📚"),
("env.example", "🔒"),
("LICENSE", "📜"),
("README", "📖"),
("CHANGELOG", "🗒️"),
("TODO", "📝"),
("CONTRIBUTING", "🤝"),
("CODE_OF_CONDUCT", "📜"),
("SECURITY", "🛡️"),
("Gulpfile", "🛠️"),
("Gruntfile", "🛠️"),
("LICENSE.md", "📜"),
("README.md", "📖"),
("Dockerfile", "🐳"),
("Makefile", "🛠️"),
("Procfile", "📄"), // Heroku Procfile
("yarn.lock", "🔒"),
("package-lock.json", "🔒"),
("Pipfile.lock", "🔒"),

    // Fonts
("ttf", "🔤"),
("otf", "🔤"),
("woff", "🔡"),
("woff2", "🔡"),
("eot", "🔡"),

    // Default
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
    width: u16,
    height: u16,
    page: usize,  // Current page number
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

    fn get_visible_content(&self, available_height: u16) -> (String, bool) {
        let content_height = (available_height - 4) as usize; // Account for borders and title
        let lines: Vec<&str> = self.message.lines().collect();
        let total_lines = lines.len();
        
        // Calculate total pages based on actual content vs available height
        let total_pages = (total_lines + content_height - 1) / content_height;
        let has_more_pages = total_lines > content_height;
        
        // Get lines for current page
        let start = self.page * content_height;
        let end = (start + content_height).min(total_lines);
        
        let visible_content = lines[start..end].join("\n");
        
        // Add page indicator if there are multiple pages
        let content = if has_more_pages {
            format!("{}\n\nPage {} of {}", visible_content, self.page + 1, total_pages)
        } else {
            visible_content
        };
        
        (content, has_more_pages)
    }

    fn next_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = (total_lines + content_height - 1) / content_height;
        if total_pages > 1 {  // Only change page if we have multiple pages
            self.page = (self.page + 1) % total_pages;
        }
    }

    fn prev_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = (total_lines + content_height - 1) / content_height;
        if total_pages > 1 {  // Only change page if we have multiple pages
            self.page = (self.page + total_pages - 1) % total_pages;
        }
    }
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
    show_line_numbers: bool,
}

fn add_items_recursive(
    items: &mut Vec<PathBuf>,
    dir: &PathBuf,
    expanded_folders: &HashSet<PathBuf>,
    ignore_config: &IgnoreConfig,
    current_dir: &PathBuf,
    depth: usize,
) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            if !ignore_config.use_default_ignores && !ignore_config.use_gitignore {
                return true;
            }

            if ignore_config.use_default_ignores {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if DEFAULT_IGNORED_DIRS.contains(&name) {
                        return false;
                    }
                }
            }

            if ignore_config.use_gitignore {
                let mut builder = GitignoreBuilder::new(current_dir);
                let mut dir = current_dir.clone();
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
                    let is_dir = p.is_dir();
                    match gitignore.matched_path_or_any_parents(p, is_dir) {
                        Match::Ignore(_) => return false,
                        _ => (),
                    }
                }
            }

            true
        })
        .collect();

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
        }
    }

    fn load_items(&mut self) -> io::Result<()> {
        self.items.clear();
        
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
        )?;

        // Update filtered items based on search
        self.update_search();
        Ok(())
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
                        if let Some(modal) = &mut self.modal {
                            if modal.height > 10 { // If help modal is showing
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
                                        self.modal = None;
                                        continue;
                                    }
                                }
                            }
                        }
                        
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
        
        // For JSON format, we need to start the root array
        if self.output_format == OutputFormat::Json {
            contents.push_str("[");
        }
        
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
        
        let mut first_item = true;
        for path in to_process {
            if let Some(rel_path) = path.strip_prefix(&self.current_dir).ok() {
                let normalized_path = normalize_path(&rel_path.to_string_lossy());
                
                // For JSON format, add comma between items
                if self.output_format == OutputFormat::Json && !first_item {
                    contents.push(',');
                }
                first_item = false;
                
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
                                OutputFormat::Json => {
                                    contents.push_str(&format!("{{\"type\":\"file\",\"path\":\"{}\",\"binary\":true}}", normalized_path));
                                }
                            }
                            file_count += 1;
                        }
                    } else {
                        match self.output_format {
                            OutputFormat::Xml => {
                                contents.push_str(&format!("<file name=\"{}\">\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(path) {
                                    if self.show_line_numbers {
                                        for (i, line) in content.lines().enumerate() {
                                            contents.push_str(&format!("{:>6} | {}\n", i + 1, line));
                                        }
                                    } else {
                                        contents.push_str(&content);
                                    }
                                    if !content.ends_with('\n') {
                                        contents.push('\n');
                                    }
                                }
                                contents.push_str("</file>\n");
                            }
                            OutputFormat::Markdown => {
                                contents.push_str(&format!("```{}\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(path) {
                                    if self.show_line_numbers {
                                        for (i, line) in content.lines().enumerate() {
                                            contents.push_str(&format!("{:>6} | {}\n", i + 1, line));
                                        }
                                    } else {
                                        contents.push_str(&content);
                                    }
                                    if !content.ends_with('\n') {
                                        contents.push('\n');
                                    }
                                }
                                contents.push_str("```\n\n");
                            }
                            OutputFormat::Json => {
                                if let Ok(content) = fs::read_to_string(path) {
                                    let escaped_content = content
                                        .replace('\\', "\\\\")
                                        .replace('\"', "\\\"")
                                        .replace('\n', "\\n")  // Always escape newlines for JSON
                                        .replace('\r', "\\r"); 
                                    if self.show_line_numbers {
                                        let lines: Vec<String> = content.lines()
                                            .enumerate()
                                            .map(|(i, line)| format!("{:>6} | {}", i + 1, line))
                                            .collect();
                                        let numbered_content = lines.join("\\n");  // Use escaped newline for join
                                        contents.push_str(&format!("{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}", 
                                            normalized_path, numbered_content.replace('\"', "\\\"")));
                                    } else {
                                        contents.push_str(&format!("{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}", 
                                            normalized_path, escaped_content));
                                    }
                                }
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
                        OutputFormat::Json => {
                            let mut dir_contents = String::new();
                            contents.push_str(&format!("{{\"type\":\"directory\",\"path\":\"{}\",\"contents\":[", normalized_path));
                            if let Ok((files, folders)) = self.process_directory(path, &mut dir_contents, &self.current_dir) {
                                contents.push_str(&dir_contents);
                                file_count += files;
                                folder_count += folders;
                            }
                            contents.push(']');
                            contents.push('}');
                        }
                    }
                    folder_count += 1;
                }
            }
        }

        // Close the root array for JSON format
        if self.output_format == OutputFormat::Json {
            contents.push(']');
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
                
                self.modal = Some(Modal::copy_stats(
                    file_count,
                    folder_count,
                    line_count,
                    byte_size,
                    &self.output_format
                ));
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
        
        entries.sort();

        let mut first_item = true;
        for path in entries {
            if let Some(rel_path) = path.strip_prefix(base_path).ok() {
                let normalized_path = normalize_path(&rel_path.to_string_lossy());
                
                // For JSON format, add comma between items
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
                                    let escaped_content = content
                                        .replace('\\', "\\\\")
                                        .replace('\"', "\\\"")
                                        .replace('\n', "\\n")  // Always escape newlines for JSON
                                        .replace('\r', "\\r"); 
                                    if self.show_line_numbers {
                                        let lines: Vec<String> = content.lines()
                                            .enumerate()
                                            .map(|(i, line)| format!("{:>6} | {}", i + 1, line))
                                            .collect();
                                        let numbered_content = lines.join("\\n");  // Use escaped newline for join
                                        output.push_str(&format!("{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}", 
                                            normalized_path, numbered_content.replace('\"', "\\\"")));
                                    } else {
                                        output.push_str(&format!("{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}", 
                                            normalized_path, escaped_content));
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
                        OutputFormat::Json => {
                            let mut dir_contents = String::new();
                            output.push_str(&format!("{{\"type\":\"directory\",\"path\":\"{}\",\"contents\":[", normalized_path));
                            if let Ok((files, folders)) = self.process_directory(&path, &mut dir_contents, base_path) {
                                output.push_str(&dir_contents);
                                file_count += files;
                                folder_count += folders;
                            }
                            output.push(']');
                            output.push('}');
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

        // Create a layout for the title block to split it into header and search areas
        let title_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // For the header
                Constraint::Length(1),  // For the search
            ])
            .split(chunks[0]);

        // Render the header
        let title = Block::default()
            .title(format!(" AIBundle v{} - {} ", VERSION, self.current_dir.display()))
            .borders(Borders::ALL);
        f.render_widget(title.clone(), chunks[0]);

        // Render the search area inside the title block if searching
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
            
            // Render the search text inside the title block, with a 2-character margin
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

        let status = Block::default()
            .title(status_text)
            .borders(Borders::ALL);
        f.render_widget(status, chunks[2]);

        // Draw modal if exists and recent
        if let Some(modal) = &self.modal {
            let is_help = modal.height > 10; // Help modal is taller than copy modal
            let timeout = if is_help { 30 } else { 2 }; // Help modal stays longer
            
            if modal.timestamp.elapsed().as_secs() < timeout {
                let area = centered_rect(modal.width, modal.height, f.area());
                
                let (content, has_more_pages) = modal.get_visible_content(area.height);
                let lines: Vec<&str> = content.lines().collect();
                let max_length = lines.iter()
                    .map(|line| line.len())
                    .max()
                    .unwrap_or(0);
                
                let total_space = area.width as usize - 2;
                let padding = (total_space - max_length) / 2;
                let pad = " ".repeat(padding);
                
                let padded_lines: Vec<Line> = std::iter::once("")
                    .chain(lines.into_iter())
                    .map(|line| {
                        if line.is_empty() {
                            Line::from(line.to_string())
                        } else {
                            Line::from(format!("{}{}", if is_help { "" } else { &pad }, line))
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
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(title))
                    .alignment(if is_help { Alignment::Left } else { Alignment::Center });

                f.render_widget(Clear, area);
                f.render_widget(text, area);
            } else {
                self.modal = None;
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

    fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
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
        
        // Also check for files without extensions that are known to be binary
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

        // Check .gitignore if enabled
        if self.ignore_config.use_gitignore {
            let mut builder = GitignoreBuilder::new(&self.current_dir);
            
            // Try to load .gitignore from current and parent directories
            let mut dir = self.current_dir.clone();
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
                match gitignore.matched_path_or_any_parents(path, is_dir) {
                    Match::Ignore(_) => return true,
                    _ => (),
                }
            }
        }

        false
    }

    fn toggle_folder_expansion(&mut self) -> io::Result<()> {
        if let Some(selected) = self.list_state.selected() {
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
        Ok(())
    }

    fn toggle_select_all(&mut self) {
        // Check if all visible items are already selected
        let all_selected = self.filtered_items.iter()
            .filter(|path| !path.ends_with("..")) // Exclude parent directory
            .all(|path| self.selected_items.contains(path));

        if all_selected {
            // Unselect all items
            self.selected_items.clear();
        } else {
            // Collect paths first to avoid borrow checker issues
            let paths_to_select: Vec<_> = self.filtered_items.iter()
                .filter(|path| !path.ends_with(".."))
                .cloned()
                .collect();

            // Then process them
            for path in paths_to_select {
                if path.is_dir() {
                    self.update_folder_selection(&path, true);
                } else {
                    self.selected_items.insert(path);
                }
            }
        }
    }

    fn show_help(&mut self) {
        self.modal = Some(Modal::help());
    }
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    // width and height are now in characters/lines instead of percentages
    
    // Ensure the requested size doesn't exceed the terminal
    let popup_width = width.min(r.width);
    let popup_height = height.min(r.height);

    // Calculate margins to center the rect
    let x_margin = (r.width - popup_width) / 2;
    let y_margin = (r.height - popup_height) / 2;

    Rect {
        x: r.x + x_margin,
        y: r.y + y_margin,
        width: popup_width,
        height: popup_height,
    }
}

// Add this helper function for human-readable byte sizes
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

// Add this function to handle cross-platform clipboard operations
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
            // First try PowerShell clip.exe (WSL2 -> Windows clipboard)
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
                // Give clip.exe time to process
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
                    // Fall back to xclip (X11)
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

// Add this function to get clipboard contents
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
            // First try PowerShell Get-Clipboard (WSL2 -> Windows clipboard)
            let powershell_result = Command::new("powershell.exe")
                .args(["-Command", "Get-Clipboard"])
                .output();

            if let Ok(output) = powershell_result {
                if output.status.success() {
                    return String::from_utf8(output.stdout)
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"));
                }
            }

            // Try wl-paste (Wayland)
            let wl_output = Command::new("wl-paste")
                .output();

            match wl_output {
                Ok(output) if output.status.success() => {
                    String::from_utf8(output.stdout)
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"))
                }
                _ => {
                    // Fall back to xclip (X11)
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