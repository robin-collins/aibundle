use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crate::models::{AppConfig, OutputFormat};
use crate::tui::handlers::{ClipboardHandler, FileOpsHandler, KeyboardHandler};
use crate::tui::state::{AppState, SearchState, SelectionState};
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
            terminal.draw(|f| {
                self.main_view
                    .render(f, &self.state, &self.selection_state, &self.search_state)
            })?;

            // Handle events with timeout
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    self.keyboard_handler.handle_key(
                        key,
                        &mut self.state,
                        &mut self.selection_state,
                        &mut self.search_state,
                    )?;
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
                println!(
                    "  Total size: {}",
                    crate::utils::human_readable_size(byte_size)
                );
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
