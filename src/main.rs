use clap::Parser;
use cli_clipboard::{ClipboardContext, ClipboardProvider};
/// File: src/main.rs
/// Encoding: UTF-8
///
/// This version modifies `count_selection_items` so that it bails out early
/// once the total count exceeds the SELECTION_LIMIT, potentially saving time
/// and resources in large directories. The rest of the file remains the same
/// as your previous version.
///
/// Version incremented to 0.5.9 to reflect this update.
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ignore::{gitignore::GitignoreBuilder, Match};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Alignment,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::env::consts::OS;
use std::process::Command;
use std::time::Duration;
use std::{collections::HashSet, fs, io, path::Path, path::PathBuf};

const VERSION: &str = "0.6.3"; 
const DEFAULT_SELECTION_LIMIT: usize = 400;

#[derive(Serialize, Deserialize, Debug, Default)]
struct AppConfig {
    default_format: Option<String>,
    default_gitignore: Option<bool>,
    default_ignore: Option<Vec<String>>,
    default_line_numbers: Option<bool>,
    default_recursive: Option<bool>,
}

// Re-add the new types to support separate CLI and TUI configuration
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
struct ModeConfig {
    files: Option<String>,
    format: Option<String>,
    out: Option<String>,
    gitignore: Option<bool>,
    ignore: Option<Vec<String>>,
    line_numbers: Option<bool>,
    recursive: Option<bool>,
    source_dir: Option<String>,
    selection_limit: Option<usize>, // New field for selection limit override
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct FullConfig {
    cli: Option<ModeConfig>,
    tui: Option<ModeConfig>,
}

/// Command-line options parsed via clap.
#[derive(Parser, Debug)]
#[command(name = "aibundle", version = VERSION)]
#[command(about = "AIBUNDLE: A CLI & TUI file aggregator and formatter")]
#[command(long_about = "\
A powerful tool for aggregating and formatting files with both CLI and TUI modes.

EXAMPLES:
    aibundle                                                # TUI mode (default, using current folder)
    aibundle d:\\projects                                   # TUI mode, starting in d:\\projects (Windows)
    aibundle /mnt/d/projects                                # TUI mode, starting in /mnt/d/projects (POSIX)
    aibundle --files \"*.rs\"                              # CLI mode, current folder, files that match \"*.rs\"
    aibundle --files \"*.rs\" /mnt/d/projects/rust_aiformat  # CLI mode, starting in specified directory")]
struct CliOptions {
    /// Optional positional source directory.
    #[arg(value_name = "SOURCE_DIR", index = 1)]
    source_dir_pos: Option<String>,

    /// Write output to file instead of clipboard
    #[arg(short = 'o', long)]
    output_file: Option<String>,

    /// Write output to console instead of clipboard
    #[arg(short = 'p', long)]
    output_console: bool,

    /// File pattern to match (e.g., "*.rs" or "*.{rs,toml}")
    #[arg(short = 'f', long)]
    files: Option<String>,

    /// Search pattern to match file contents (e.g., "test" to match files containing 'test')
    #[arg(short = 's', long)]
    search: Option<String>,

    /// Output format to use [possible values: markdown, xml, json]
    #[arg(short = 'm', long, value_parser = ["markdown", "xml", "json"], default_value = "xml")]
    format: String,

    /// Source directory to start from (overridden by the positional SOURCE_DIR if supplied)
    #[arg(short = 'd', long, default_value = ".")]
    source_dir: String,

    /// Include subfolders in search
    #[arg(short = 'r', long, default_value = "true")]
    recursive: bool,

    /// Show line numbers in output (ignored in JSON format)
    #[arg(short = 'n', long)]
    line_numbers: bool,

    /// Use .gitignore files for filtering
    #[arg(short = 'g', long, default_value = "true")]
    gitignore: bool,

    /// Ignore patterns (comma-separated list)
    #[arg(
        short = 'i',
        long,
        use_value_delimiter = true,
        default_value = "default"
    )]
    ignore: Vec<String>,

    /// Save current settings to .aibundle.config
    #[arg(short = 'S', long)]
    save_config: bool,
}

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
    ("rs", "🦀"),             // Rust
    ("ts", "💠"),             // TypeScript
    ("js", "📜"),             // JavaScript
    ("py", "🐍"),             // Python
    ("java", "☕"),           // Java
    ("class", "☕"),          // Java Class Files
    ("cpp", "⚡"),            // C++
    ("c", "🔌"),              // C
    ("h", "📐"),              // C Header
    ("hpp", "📐"),            // C++ Header
    ("go", "🐹"),             // Go
    ("rb", "💎"),             // Ruby
    ("php", "🐘"),            // PHP
    ("scala", "💫"),          // Scala
    ("swift", "🕊️"),          // Swift
    ("kotlin", "🎯"),         // Kotlin
    ("dart", "🦋"),           // Dart
    ("lua", "🌙"),            // Lua
    ("sh", "🐚"),             // Shell Script
    ("pl", "🔮"),             // Perl
    ("r", "📊"),              // R
    ("erl", "🐘"),            // Erlang
    ("hs", "☯️"),             // Haskell
    ("sql", "🗃️"),            // SQL
    ("m", "🔧"),              // Objective-C / MATLAB (context-dependent)
    ("adb", "📱"),            // Android Debug Bridge Scripts
    ("gradle", "🛠️"),         // Gradle Build Scripts
    ("pom", "📦"),            // Maven POM Files
    ("sbt", "📚"),            // Scala Build Tool
    ("makefile", "🛠️"),       // Makefile
    ("Dockerfile", "🐳"),     // Dockerfile
    ("Vagrantfile", "🔧"),    // Vagrant Configuration
    ("gulpfile", "🛠️"),       // Gulp Build Scripts
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
    ("erb", "🌸"),        // Embedded Ruby
    ("handlebars", "🪡"), // Handlebars Templates
    ("ejs", "🖥️"),        // Embedded JavaScript
    ("phphtml", "🐘"),    // PHP Embedded in HTML
    ("jade", "🌿"),       // Jade/Pug Templates
    // Data
    ("json", "📋"),
    ("yaml", "⚙️"),
    ("yml", "⚙️"),
    ("xml", "📰"),
    ("csv", "📊"),
    ("db", "🗃️"),
    ("sqlite", "🗃️"),
    ("dbml", "🗂️"), // Database Markup Language
    ("geojson", "🗺️"),
    ("parquet", "📦"),
    ("pickle", "🥒"),
    ("protobuf", "🔗"), // Protocol Buffers
    ("avsc", "🗃️"),     // Avro Schema
    ("ndjson", "📄"),   // Newline Delimited JSON
    ("hdf5", "📁"),     // Hierarchical Data Format
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
    ("gsheet", "📊"),  // Google Sheets
    // Presentations
    ("ppt", "📙"),
    ("pptx", "📙"),
    ("odp", "📙"),
    ("key", "🔑"),     // Apple Keynote
    ("mdx", "📝"),     // Markdown with JSX
    ("sldx", "📊"),    // Slide files
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
    ("psd", "🖌️"),  // Photoshop
    ("ai", "🖍️"),   // Adobe Illustrator
    ("indd", "📄"), // Adobe InDesign
    ("xcf", "🎨"),  // GIMP
    ("eps", "📐"),  // Encapsulated PostScript
    ("drw", "🖊️"),  // Generic Drawing
    ("dxf", "📏"),  // Drawing Exchange Format
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
    ("vob", "📀"),  // DVD Video Object
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
    ("circleci", "🔄"),      // CircleCI Config
    ("travis.yml", "📈"),    // Travis CI Config
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
    ("dll", "🗄️"),   // Dynamic Link Library
    ("so", "🗄️"),    // Shared Object
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
    // Fonts (duplicates omitted; left once if encountered)
    // Default
    ("default", "📄"),
];

const DEFAULT_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    "coverage",
    "target", // Keep Rust's target dir in defaults
];

#[derive(Clone)]
struct IgnoreConfig {
    use_default_ignores: bool,
    use_gitignore: bool,
    include_binary_files: bool,
    extra_ignore_patterns: Vec<String>,
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
struct CopyStats {
    files: usize,
    folders: usize,
}

struct Modal {
    message: String,
    timestamp: std::time::Instant,
    width: u16,
    height: u16,
    page: usize,
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

    fn copy_stats(
        file_count: usize,
        folder_count: usize,
        line_count: usize,
        byte_size: usize,
        format: &OutputFormat,
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
            timestamp: std::time::Instant::now(),
            width: 60,
            height: 30,
            page: 0,
        }
    }

    fn get_visible_content(&self, available_height: u16) -> (String, bool) {
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

    fn next_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = total_lines.div_ceil(content_height);
        if total_pages > 1 {
            self.page = (self.page + 1) % total_pages;
        }
    }

    fn prev_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = total_lines.div_ceil(content_height);
        if total_pages > 1 {
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
    pending_count: Option<std::sync::mpsc::Receiver<io::Result<usize>>>,
    counting_path: Option<PathBuf>,
    is_counting: bool,
    config: AppConfig,
    selection_limit: usize, // New dynamic selection limit field
}

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            current_dir: std::env::current_dir().unwrap_or_default(),
            items: Vec::new(),
            list_state,
            selected_items: std::collections::HashSet::new(),
            quit: false,
            last_copy_stats: None,
            modal: None,
            ignore_config: IgnoreConfig::default(),
            expanded_folders: std::collections::HashSet::new(),
            search_query: String::new(),
            filtered_items: Vec::new(),
            is_searching: false,
            output_format: OutputFormat::Xml,
            show_line_numbers: false,
            pending_count: None,
            counting_path: None,
            is_counting: false,
            config: AppConfig::default(),
            selection_limit: DEFAULT_SELECTION_LIMIT, // Set default limit
        }
    }

    fn load_items(&mut self) -> io::Result<()> {
        self.items.clear();

        if let Some(parent) = self.current_dir.parent() {
            if !parent.as_os_str().is_empty() {
                self.items.push(self.current_dir.join(".."));
            }
        }

        add_items_iterative(
            &mut self.items,
            &self.current_dir,
            &self.expanded_folders,
            &self.ignore_config,
            &self.current_dir,
        )?;

        self.update_search();
        Ok(())
    }

    fn load_items_nonrecursive(&mut self) -> io::Result<()> {
        self.items.clear();
        self.filtered_items.clear();

        // Add parent directory entry if applicable
        if let Some(parent) = self.current_dir.parent() {
            if !parent.as_os_str().is_empty() {
                self.items.push(self.current_dir.join(".."));
            }
        }

        // Read only the current directory, no recursion
        let entries = fs::read_dir(&self.current_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| !is_path_ignored_for_iterative(p, &self.current_dir, &self.ignore_config))
            .collect::<Vec<_>>();

        // Sort entries (directories first, then files)
        let mut sorted_entries = entries;
        sorted_entries.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        self.items.extend(sorted_entries);
        self.filtered_items = self.items.clone();
        Ok(())
    }

    fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_items = self.items.clone();
            return;
        }

        let query = self.search_query.to_lowercase();
        let use_glob = query.contains('*') || query.contains('?');

        if use_glob {
            // Use glob pattern matching for file names.
            if let Ok(pattern) = glob::Pattern::new(&query) {
                self.filtered_items = self
                    .items
                    .iter()
                    .filter(|path| {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            pattern.matches(&name.to_lowercase())
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();
            } else {
                // If glob pattern fails to compile, fallback to substring matching.
                self.filtered_items = self
                    .items
                    .iter()
                    .filter(|path| {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            name.to_lowercase().contains(&query)
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();
            }
        } else {
            self.filtered_items = self
                .items
                .iter()
                .filter(|path| {
                    if let Ok(rel_path) = path.strip_prefix(&self.current_dir) {
                        let path_str = rel_path.to_string_lossy().to_lowercase();
                        path_str.contains(&query)
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
        }

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
            '/' => {
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
            // Check for pending selection count results
            if self.is_counting {
                if let Some(rx) = &self.pending_count {
                    if let Ok(Ok(count)) = rx.try_recv() {
                        if count <= self.selection_limit {
                            if let Some(path) = self.counting_path.take() {
                                if path.is_dir() {
                                    self.update_folder_selection(&path, true);
                                } else {
                                    self.selected_items.insert(path);
                                }
                            }
                        } else {
                            self.modal = Some(Modal::new(
                                format!(
                                    "Cannot select: would exceed limit of {} items\nTried to add {} items",
                                    self.selection_limit, count
                                ),
                                50,
                                4,
                            ));
                        }
                        self.is_counting = false;
                        self.pending_count = None;
                    }
                }
            }

            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        // If a modal is open, handle modal key events first
                        if let Some(modal) = &mut self.modal {
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
                                        self.modal = None;
                                        continue;
                                    }
                                }
                            }
                        }

                        // If search mode is active, handle search input
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
                            // Normal mode key handling
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
                                KeyCode::Char('s') => self.save_config()?,
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
        let new_selected =
            (current as i32 + delta).clamp(0, self.filtered_items.len() as i32 - 1) as usize;
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
        if let Some(selected_index) = self.list_state.selected() {
            if selected_index >= self.filtered_items.len() {
                return;
            }
            let path = self.filtered_items[selected_index].clone();
            if path.file_name().map_or(false, |n| n == "..") {
                return;
            }

            let is_selected = self.selected_items.contains(&path);
            // If it's already selected, unselect it immediately (no counting needed)
            if is_selected {
                if path.is_dir() {
                    self.update_folder_selection(&path, false);
                } else {
                    self.selected_items.remove(&path);
                }
                return;
            }

            // If not selected, start an async count to see if we can add it without exceeding limit
            if !self.is_counting {
                let (tx, rx) = std::sync::mpsc::channel();
                let base_path = self.current_dir.clone();
                let ignore_config = self.ignore_config.clone();
                let path_clone = path.clone();
                let selection_limit = self.selection_limit; // Capture the current limit
                std::thread::spawn(move || {
                    let result = count_selection_items_async(
                        &path_clone,
                        &base_path,
                        &ignore_config,
                        selection_limit,
                    );
                    let _ = tx.send(result);
                });

                self.pending_count = Some(rx);
                self.counting_path = Some(path);
                self.is_counting = true;
            }
        }
    }

    fn toggle_select_all(&mut self) {
        let all_selected = self
            .filtered_items
            .iter()
            .filter(|path| !path.ends_with(".."))
            .all(|path| self.selected_items.contains(path));

        if all_selected {
            self.selected_items.clear();
        } else {
            let mut total_new_items = 0usize;
            let mut would_select = Vec::new();

            for path in &self.filtered_items {
                if path.ends_with("..") {
                    continue;
                }
                if !self.selected_items.contains(path) {
                    if path.is_dir() {
                        if let Ok(sub_count) = self.count_selection_items(path) {
                            total_new_items += sub_count;
                            would_select.push(path.clone());
                        }
                    } else {
                        total_new_items += 1;
                        would_select.push(path.clone());
                    }
                }
            }

            let total_would_be = self.selected_items.len() + total_new_items;
            if total_would_be > self.selection_limit {
                let msg = format!(
                    "Selection aborted.\n\
                     Selecting {} additional items would exceed the limit of {}.\n\
                     Currently selected: {}",
                    total_new_items,
                    self.selection_limit,
                    self.selected_items.len()
                );
                self.modal = Some(Modal::new(msg, 50, 6));
                return;
            }

            for path in would_select {
                if path.is_dir() {
                    self.update_folder_selection(&path, true);
                } else {
                    self.selected_items.insert(path);
                }
            }
        }
    }

    fn copy_selected_to_clipboard(&mut self) -> io::Result<()> {
        let contents = self.format_selected_items()?;
        let mut ctx = ClipboardContext::new().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create clipboard context: {}", e),
            )
        })?;
        ctx.set_contents(contents.to_owned()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to set clipboard contents: {}", e),
            )
        })?;

        // Update copy stats
        let line_count = contents.lines().count();
        let byte_size = contents.len();
        let (file_count, folder_count) = self.count_selected_items();
        self.last_copy_stats = Some(CopyStats {
            files: file_count,
            folders: folder_count,
        });
        self.modal = Some(Modal::copy_stats(
            file_count,
            folder_count,
            line_count,
            byte_size,
            &self.output_format,
        ));

        Ok(())
    }

    fn count_selected_items(&self) -> (usize, usize) {
        let mut file_count = 0;
        let mut folder_count = 0;

        for path in &self.selected_items {
            if path.is_file() {
                file_count += 1;
            } else if path.is_dir() {
                folder_count += 1;
            }
        }

        (file_count, folder_count)
    }

    fn process_directory(
        &self,
        path: &PathBuf,
        output: &mut String,
        base_path: &PathBuf,
    ) -> io::Result<(usize, usize)> {
        let mut file_count = 0;
        let mut folder_count = 0;

        let mut entries: Vec<PathBuf> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        entries.retain(|p| !is_path_ignored_for_iterative(p, base_path, &self.ignore_config));
        entries.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in entries.iter() {
            let entry_clone = entry.clone();
            if let Ok(rel_path) = entry_clone.strip_prefix(base_path) {
                let normalized_path = normalize_path(&rel_path.to_string_lossy());

                if entry_clone.is_file() {
                    if Self::is_binary_file(&entry_clone) {
                        if self.ignore_config.include_binary_files {
                            match self.output_format {
                                OutputFormat::Xml => {
                                    output.push_str(&format!(
                                        "<file name=\"{}\">\n</file>\n",
                                        normalized_path
                                    ));
                                }
                                OutputFormat::Markdown => {
                                    output.push_str(&format!(
                                        "```{}\n<binary file>\n```\n\n",
                                        normalized_path
                                    ));
                                }
                                OutputFormat::Json => {
                                    output.push_str(&format!(
                                        "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":true}}",
                                        normalized_path
                                    ));
                                }
                            }
                            file_count += 1;
                        }
                    } else {
                        match self.output_format {
                            OutputFormat::Xml => {
                                output.push_str(&format!("<file name=\"{}\">\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(&entry_clone) {
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
                                if let Ok(content) = fs::read_to_string(&entry_clone) {
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
                                if let Ok(content) = fs::read_to_string(&entry_clone) {
                                    let escaped_content = content
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
                                        let numbered_content = lines.join("\\n");
                                        output.push_str(&format!(
                                            "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}",
                                            normalized_path,
                                            numbered_content.replace('\"', "\\\"")
                                        ));
                                    } else {
                                        output.push_str(&format!(
                                            "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}",
                                            normalized_path,
                                            escaped_content
                                        ));
                                    }
                                }
                            }
                        }
                        file_count += 1;
                    }
                } else if entry_clone.is_dir() {
                    match self.output_format {
                        OutputFormat::Xml => {
                            output.push_str(&format!("<folder name=\"{}\">\n", normalized_path));
                            let mut dir_contents = String::new();
                            if let Ok((files, folders)) =
                                self.process_directory(&entry_clone, &mut dir_contents, base_path)
                            {
                                file_count += files;
                                folder_count += folders;
                            }
                            output.push_str(&dir_contents);
                            output.push_str("</folder>\n");
                        }
                        OutputFormat::Markdown => {
                            let mut dir_contents = String::new();
                            if let Ok((files, folders)) =
                                self.process_directory(&entry_clone, &mut dir_contents, base_path)
                            {
                                file_count += files;
                                folder_count += folders;
                            }
                            output.push_str(&dir_contents);
                        }
                        OutputFormat::Json => {
                            let mut dir_contents = String::new();
                            output.push_str(&format!(
                                "{{\"type\":\"directory\",\"path\":\"{}\",\"contents\":[",
                                normalized_path
                            ));
                            if let Ok((files, folders)) =
                                self.process_directory(&entry_clone, &mut dir_contents, base_path)
                            {
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

        if self.output_format == OutputFormat::Json {
            output.push(']');
        }

        Ok((file_count, folder_count))
    }

    /// Updated method that **bails early** when partial count exceeds SELECTION_LIMIT.
    fn count_selection_items(&self, path: &PathBuf) -> io::Result<usize> {
        if path.is_file() {
            return Ok(1);
        }
        if path.is_dir() {
            let mut count = 0;
            let mut stack = vec![path.clone()];

            while let Some(current) = stack.pop() {
                if self.is_path_ignored(&current) {
                    continue;
                }
                if current.is_file() {
                    count += 1;
                } else if current.is_dir() {
                    count += 1;
                    if count > DEFAULT_SELECTION_LIMIT {
                        // Bail as soon as limit is exceeded
                        return Ok(count);
                    }
                    let entries = fs::read_dir(&current)?
                        .filter_map(|e| e.ok())
                        .map(|e| e.path());
                    for entry_path in entries {
                        stack.push(entry_path);
                    }
                }
                if count > DEFAULT_SELECTION_LIMIT {
                    return Ok(count);
                }
            }
            return Ok(count);
        }
        Ok(0)
    }

    fn update_folder_selection(&mut self, path: &PathBuf, selected: bool) {
        if path.is_dir() {
            if selected {
                self.selected_items.insert(path.clone());
            } else {
                self.selected_items.remove(path);
            }

            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let child_path = entry.path();
                    if child_path.is_dir() {
                        self.update_folder_selection(&child_path, selected);
                    } else if selected {
                        self.selected_items.insert(child_path);
                    } else {
                        self.selected_items.remove(&child_path);
                    }
                }
            }
        } else if selected {
            self.selected_items.insert(path.clone());
        } else {
            self.selected_items.remove(path);
        }
    }

    fn get_icon(path: &PathBuf) -> &'static str {
        if path.is_dir() {
            return ICONS
                .iter()
                .find(|(k, _)| *k == "folder")
                .map(|(_, v)| *v)
                .unwrap_or("📁");
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
                    .unwrap_or("📄"),
            )
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
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(chunks[0]);

        let title = Block::default()
            .title(format!(
                " AIBundle v{} - {} ",
                VERSION,
                self.current_dir.display()
            ))
            .borders(Borders::ALL);
        f.render_widget(title, chunks[0]);

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

        if let Some(modal) = &self.modal {
            let is_help = modal.height > 10;
            let timeout = if is_help { 30 } else { 2 };

            if modal.timestamp.elapsed().as_secs() < timeout {
                let area = centered_rect(modal.width, modal.height, f.area());
                let (content, has_more_pages) = modal.get_visible_content(area.height);
                let lines: Vec<&str> = content.lines().collect();
                let max_length = lines.iter().map(|line| line.len()).max().unwrap_or(0);
                let total_space = area.width.saturating_sub(2) as usize;
                let padding = total_space.saturating_sub(max_length) / 2;
                let pad = " ".repeat(padding);

                let padded_lines: Vec<Line> = std::iter::once("")
                    .chain(lines)
                    .map(|line| {
                        if line.is_empty() {
                            Line::from(line.to_string())
                        } else if is_help {
                            Line::from(line.to_string())
                        } else {
                            Line::from(format!("{}{}", pad, line))
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
                    " Message: "
                };

                let text = Paragraph::new(padded_lines)
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .alignment(if is_help {
                        Alignment::Left
                    } else {
                        Alignment::Center
                    });
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
        // Don't toggle line numbers if we're in JSON mode
        if self.output_format != OutputFormat::Json {
            self.show_line_numbers = !self.show_line_numbers;
        }
    }

    fn show_help(&mut self) {
        self.modal = Some(Modal::help());
    }

    fn is_binary_file(path: &PathBuf) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let binary_extensions = [
                "idx", "pack", "rev", "index", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp",
                "ico", "svg", "mp3", "wav", "ogg", "flac", "m4a", "aac", "wma", "mp4", "avi",
                "mkv", "mov", "wmv", "flv", "webm", "zip", "rar", "7z", "tar", "gz", "iso", "exe",
                "dll", "so", "dylib", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "class",
                "pyc", "pyd", "pyo",
            ];
            if binary_extensions.contains(&ext.to_lowercase().as_str()) {
                return true;
            }
        }

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let binary_files = ["index"];
            return binary_files.contains(&name);
        }
        false
    }

    fn is_path_ignored(&self, path: &Path) -> bool {
        if !self.ignore_config.use_default_ignores && !self.ignore_config.use_gitignore {
            return false;
        }
        if self.ignore_config.use_default_ignores {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if DEFAULT_IGNORED_DIRS.contains(&name) {
                    return true;
                }
            }
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if self
                .ignore_config
                .extra_ignore_patterns
                .contains(&name.to_string())
            {
                return true;
            }
        }
        if self.ignore_config.use_gitignore {
            let mut builder = GitignoreBuilder::new(&self.current_dir);
            let mut dir = self.current_dir.clone();
            while let Some(parent) = dir.parent() {
                let gitignore = dir.join(".gitignore");
                if gitignore.exists() && builder.add(gitignore).is_some() {
                    break;
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

    fn toggle_folder_expansion(&mut self) -> io::Result<()> {
        if let Some(selected) = self.list_state.selected() {
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

    fn save_config(&self) -> io::Result<()> {
        let config_path = ".aibundle.config";
        if Path::new(config_path).exists() {
            println!("Configuration file already exists. Overwrite? (y/n): ");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted saving configuration.");
                return Ok(());
            }
        }
        let config = AppConfig {
            default_format: Some(match self.output_format {
                OutputFormat::Xml => "xml".to_string(),
                OutputFormat::Markdown => "markdown".to_string(),
                OutputFormat::Json => "json".to_string(),
            }),
            default_gitignore: Some(self.ignore_config.use_gitignore),
            default_ignore: self.config.default_ignore.clone(),
            default_line_numbers: Some(self.show_line_numbers),
            default_recursive: Some(true), // Always true in TUI mode
        };
        let toml_str = toml::to_string_pretty(&config).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("TOML serialize error: {e}"))
        })?;
        fs::write(config_path, toml_str)?;
        println!("Configuration saved successfully.");
        Ok(())
    }

    fn format_selected_items(&mut self) -> io::Result<String> {
        let mut output = String::new();
        let mut stats = CopyStats {
            files: 0,
            folders: 0,
        };

        if self.output_format == OutputFormat::Json {
            output.push('[');
        }

        // Process only items whose parent is not also selected (avoid duplication)
        let mut to_process: Vec<_> = self
            .selected_items
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

        let mut first_item = true;
        for path in to_process {
            if let Ok(rel_path) = path.strip_prefix(&self.current_dir) {
                let normalized_path = normalize_path(&rel_path.to_string_lossy());

                if self.output_format == OutputFormat::Json && !first_item {
                    output.push(',');
                }
                first_item = false;

                if path.is_file() {
                    if Self::is_binary_file(path) {
                        if self.ignore_config.include_binary_files {
                            match self.output_format {
                                OutputFormat::Xml => {
                                    output.push_str(&format!(
                                        "<file name=\"{}\">\n</file>\n",
                                        normalized_path
                                    ));
                                }
                                OutputFormat::Markdown => {
                                    output.push_str(&format!(
                                        "```{}\n<binary file>\n```\n\n",
                                        normalized_path
                                    ));
                                }
                                OutputFormat::Json => {
                                    output.push_str(&format!(
                                        "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":true}}",
                                        normalized_path
                                    ));
                                }
                            }
                            stats.files += 1;
                        }
                    } else {
                        match self.output_format {
                            OutputFormat::Xml => {
                                output.push_str(&format!("<file name=\"{}\">\n", normalized_path));
                                if let Ok(content) = fs::read_to_string(path) {
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
                                if let Ok(content) = fs::read_to_string(path) {
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
                                if let Ok(content) = fs::read_to_string(path) {
                                    let escaped_content = content
                                        .replace('\\', "\\\\")
                                        .replace('\"', "\\\"")
                                        .replace('\n', "\\n")
                                        .replace('\r', "\\r");
                                    output.push_str(&format!(
                                        "{{\"type\":\"file\",\"path\":\"{}\",\"binary\":false,\"content\":\"{}\"}}",
                                        normalized_path,
                                        escaped_content
                                    ));
                                }
                            }
                        }
                        stats.files += 1;
                    }
                } else if path.is_dir() {
                    match self.output_format {
                        OutputFormat::Xml => {
                            output.push_str(&format!("<folder name=\"{}\">\n", normalized_path));
                            let mut dir_contents = String::new();
                            if let Ok((files, folders)) =
                                self.process_directory(path, &mut dir_contents, &self.current_dir)
                            {
                                stats.files += files;
                                stats.folders += folders;
                            }
                            output.push_str(&dir_contents);
                            output.push_str("</folder>\n");
                        }
                        OutputFormat::Markdown => {
                            let mut dir_contents = String::new();
                            if let Ok((files, folders)) =
                                self.process_directory(path, &mut dir_contents, &self.current_dir)
                            {
                                stats.files += files;
                                stats.folders += folders;
                            }
                            output.push_str(&dir_contents);
                        }
                        OutputFormat::Json => {
                            let mut dir_contents = String::new();
                            output.push_str(&format!(
                                "{{\"type\":\"directory\",\"path\":\"{}\",\"contents\":[",
                                normalized_path
                            ));
                            if let Ok((files, folders)) =
                                self.process_directory(path, &mut dir_contents, &self.current_dir)
                            {
                                output.push_str(&dir_contents);
                                stats.files += files;
                                stats.folders += folders;
                            }
                            output.push(']');
                            output.push('}');
                        }
                    }
                    stats.folders += 1;
                }
            }
        }

        if self.output_format == OutputFormat::Json {
            output.push(']');
        }

        self.last_copy_stats = Some(stats);
        Ok(output)
    }
}

fn add_items_iterative(
    items: &mut Vec<PathBuf>,
    root: &PathBuf,
    expanded_folders: &HashSet<PathBuf>,
    ignore_config: &IgnoreConfig,
    base_dir: &PathBuf,
) -> io::Result<()> {
    // Only add ".." for the root directory (not for expanded subdirectories)
    if items.is_empty() && root == base_dir {
        if let Some(parent) = root.parent() {
            if !parent.as_os_str().is_empty() {
                items.push(root.join(".."));
            }
        }
    }

    // Process current directory
    let mut entries: Vec<PathBuf> = fs::read_dir(root)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    entries.retain(|p| !is_path_ignored_for_iterative(p, base_dir, ignore_config));
    entries.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    // Add entries and recursively process expanded folders
    for entry in entries {
        items.push(entry.clone());
        if entry.is_dir() && expanded_folders.contains(&entry) {
            add_items_iterative(items, &entry, expanded_folders, ignore_config, base_dir)?;
        }
    }

    Ok(())
}

fn is_path_ignored_for_iterative(
    path: &PathBuf,
    base_dir: &PathBuf,
    ignore_config: &IgnoreConfig,
) -> bool {
    if !ignore_config.use_default_ignores && !ignore_config.use_gitignore {
        return false;
    }
    if ignore_config.use_default_ignores {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if DEFAULT_IGNORED_DIRS.contains(&name) {
                return true;
            }
        }
    }
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if ignore_config
            .extra_ignore_patterns
            .contains(&name.to_string())
        {
            return true;
        }
    }
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

fn collect_all_subdirs(
    base_dir: &Path,
    ignore_config: &IgnoreConfig,
) -> io::Result<HashSet<PathBuf>> {
    let base_dir_buf = base_dir.to_path_buf();
    let mut dirs = HashSet::new();
    let mut stack = vec![base_dir_buf.clone()];
    while let Some(current) = stack.pop() {
        if current.is_dir() {
            dirs.insert(current.clone());
            for entry in fs::read_dir(&current)? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir()
                        && !is_path_ignored_for_iterative(&path, &base_dir_buf, ignore_config)
                    {
                        stack.push(path);
                    }
                }
            }
        }
    }
    Ok(dirs)
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_width = width.min(r.width);
    let popup_height = height.min(r.height);

    let x_margin = (r.width.saturating_sub(popup_width)) / 2;
    let y_margin = (r.height.saturating_sub(popup_height)) / 2;

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

fn get_clipboard_contents() -> io::Result<String> {
    match OS {
        "windows" => {
            if let Ok(mut ctx) = ClipboardContext::new() {
                ctx.get_contents().map_err(|_| {
                    io::Error::new(io::ErrorKind::Other, "Failed to get clipboard contents")
                })
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
                    return String::from_utf8(output.stdout).map_err(|_| {
                        io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard")
                    });
                }
            }
            let wl_output = Command::new("wl-paste").output();
            if let Ok(output) = wl_output {
                if output.status.success() {
                    return String::from_utf8(output.stdout).map_err(|_| {
                        io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard")
                    });
                }
            }
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

/// Asynchronous helper replicating `count_selection_items` logic without blocking the UI.
/// Now accepts a `selection_limit` parameter.
fn count_selection_items_async(
    path: &PathBuf,
    base_dir: &PathBuf,
    ignore_config: &IgnoreConfig,
    selection_limit: usize,
) -> io::Result<usize> {
    if path.is_file() {
        return Ok(1);
    }
    if path.is_dir() {
        let mut count = 0;
        let mut stack = vec![path.clone()];

        while let Some(current) = stack.pop() {
            if is_path_ignored_for_iterative(&current, base_dir, ignore_config) {
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

/// Returns the config file path in the user's home directory.
fn config_file_path() -> io::Result<PathBuf> {
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").map(PathBuf::from)
    } else {
        std::env::var("HOME").map(PathBuf::from)
    }
    .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
    Ok(home.join(".aibundle.config.toml"))
}

/// Loads a config file from the user's home directory if present.
fn load_config() -> io::Result<FullConfig> {
    let config_path = config_file_path()?;
    if config_path.exists() {
        let contents = fs::read_to_string(&config_path)?;
        let parsed: FullConfig = toml::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("TOML parse error: {e}")))?;
        Ok(parsed)
    } else {
        Ok(FullConfig::default())
    }
}

/// This function runs the tool in CLI mode, bypassing the TUI entirely.
fn run_cli_mode(
    files_pattern: Option<&str>,
    search_pattern: Option<&str>,
    source_dir: &str,
    format: &str,
    gitignore: bool,
    line_numbers: bool,
    recursive: bool,
    ignore_list: &[String],
    output_file: Option<&str>,
    output_console: bool,
) -> io::Result<()> {
    let mut app = App::new();
    app.current_dir = PathBuf::from(source_dir);
    app.ignore_config.use_gitignore = gitignore;
    app.output_format = match format.to_lowercase().as_str() {
        "markdown" => OutputFormat::Markdown,
        "json" => OutputFormat::Json,
        _ => OutputFormat::Xml,
    };
    app.show_line_numbers = line_numbers && app.output_format != OutputFormat::Json;

    // Set up ignore patterns from the CLI flag (--ignore)
    app.config.default_ignore = Some(ignore_list.to_vec());
    app.ignore_config.extra_ignore_patterns = ignore_list.to_vec();
    // Override selection limit from CLI config if provided.
    if let Some(cli_conf) = load_config()?.cli {
        if let Some(limit) = cli_conf.selection_limit {
            app.selection_limit = limit;
        }
    }

    // Load items based on patterns and recursion setting
    if recursive {
        app.expanded_folders = collect_all_subdirs(&app.current_dir, &app.ignore_config)?;
        app.load_items()?;
    } else {
        app.load_items_nonrecursive()?;
    }

    // Apply file pattern and search filters
    if let Some(pattern) = files_pattern {
        app.search_query = pattern.to_string();
        app.update_search();
    }
    if let Some(pattern) = search_pattern {
        if !app.search_query.is_empty() {
            app.search_query.push(' ');
        }
        app.search_query.push_str(pattern);
        app.update_search();
    }

    // Select all filtered items
    app.selected_items
        .extend(app.filtered_items.iter().cloned());

    // Generate output
    let output = app.format_selected_items()?;

    // Handle output
    if let Some(file_path) = output_file {
        fs::write(file_path, output)?;
        println!("Output written to file: {file_path}");
    } else if output_console {
        println!("{output}");
    } else {
        // Copy to clipboard
        let mut ctx = ClipboardContext::new().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create clipboard context: {}", e),
            )
        })?;
        ctx.set_contents(output).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to set clipboard contents: {}", e),
            )
        })?;
        println!("Output copied to clipboard");
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let cli_args = CliOptions::parse();

    // Use the positional SOURCE_DIR if supplied; otherwise, fall back to --source-dir.
    let effective_source_dir = cli_args
        .source_dir_pos
        .unwrap_or(cli_args.source_dir.clone());

    // Load existing config from the user's home directory
    let full_config = load_config()?;

    // Determine CLI mode if any of these flags are provided.
    let use_cli = cli_args.files.is_some()
        || cli_args.search.is_some()
        || cli_args.output_file.is_some()
        || cli_args.output_console
        || cli_args.save_config;

    if use_cli {
        // If saving config is requested, save a default config file (with both [cli] and [tui] sections)
        if cli_args.save_config {
            // Define our default ignored directories as a Vec<String>
            let default_ignore: Vec<String> =
                DEFAULT_IGNORED_DIRS.iter().map(|s| s.to_string()).collect();

            let cli_config = ModeConfig {
                files: cli_args.files.clone().or(Some("*".to_string())),
                format: Some(cli_args.format.clone()),
                out: cli_args.output_file.clone().or(Some("".to_string())),
                gitignore: Some(cli_args.gitignore),
                ignore: if cli_args.ignore.len() == 1 && cli_args.ignore[0] == "default" {
                    Some(default_ignore.clone())
                } else {
                    Some(cli_args.ignore.clone())
                },
                line_numbers: Some(cli_args.line_numbers),
                recursive: Some(cli_args.recursive),
                source_dir: Some(cli_args.source_dir.clone()),
                selection_limit: Some(DEFAULT_SELECTION_LIMIT),
            };
            // For TUI, use the same defaults as CLI
            let tui_config = cli_config.clone();
            let config_to_save = FullConfig {
                cli: Some(cli_config),
                tui: Some(tui_config),
            };
            let toml_str = toml::to_string_pretty(&config_to_save).map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("TOML serialize error: {e}"))
            })?;
            let config_path = config_file_path()?;
            fs::write(&config_path, toml_str)?;
            println!(
                "Configuration saved successfully to {}.",
                config_path.display()
            );

            // If --save-config was provided without any other CLI options, exit early.
            if cli_args.files.is_none()
                && cli_args.search.is_none()
                && cli_args.output_file.is_none()
                && !cli_args.output_console
            {
                return Ok(());
            }
        }

        // Merge command-line values with the [cli] defaults (command-line wins)
        let cli_conf = full_config.cli.unwrap_or_default();
        let files = cli_args.files.or(cli_conf.files);
        let format = if !cli_args.format.is_empty() {
            cli_args.format.clone()
        } else {
            cli_conf.format.unwrap_or_else(|| "xml".to_string())
        };
        let source_dir = effective_source_dir.clone();
        let gitignore = cli_args.gitignore || cli_conf.gitignore.unwrap_or(true);
        let line_numbers = cli_args.line_numbers || cli_conf.line_numbers.unwrap_or(false);
        let recursive = cli_args.recursive || cli_conf.recursive.unwrap_or(true);
        let ignore = if !cli_args.ignore.is_empty() {
            cli_args.ignore.clone()
        } else {
            cli_conf.ignore.unwrap_or_default()
        };

        run_cli_mode(
            files.as_deref(),
            cli_args.search.as_deref(),
            &source_dir,
            &format,
            gitignore,
            line_numbers,
            recursive,
            &ignore,
            cli_args.output_file.as_deref(),
            cli_args.output_console,
        )
    } else {
        // Run in TUI mode: start in the effective source directory.
        let mut app = App::new();
        app.current_dir = PathBuf::from(effective_source_dir);

        if let Some(tui_conf) = full_config.tui {
            if let Some(format) = tui_conf.format {
                app.output_format = match format.to_lowercase().as_str() {
                    "markdown" => OutputFormat::Markdown,
                    "json" => OutputFormat::Json,
                    _ => OutputFormat::Xml,
                };
            }
            if let Some(git) = tui_conf.gitignore {
                app.ignore_config.use_gitignore = git;
            }
            if let Some(ignore) = tui_conf.ignore {
                app.config.default_ignore = Some(ignore.clone());
                app.ignore_config.extra_ignore_patterns = ignore;
            }
            if let Some(ln) = tui_conf.line_numbers {
                app.show_line_numbers = ln;
            }
            if let Some(src) = tui_conf.source_dir {
                app.current_dir = PathBuf::from(src);
            }
            if let Some(limit) = tui_conf.selection_limit {
                app.selection_limit = limit;
            }
        }

        enable_raw_mode()?;
        let result = app.run();
        disable_raw_mode()?;
        result
    }
}
