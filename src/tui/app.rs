// src/tui/app.rs
//!
//! # TUI Application Entry Point
//!
//! This module defines the `App` struct, which manages the main application state, event loop, and UI rendering for the TUI.
//! It coordinates state, event handlers, and views, and is responsible for launching and running the TUI.
//!
//! ## Usage
//! Create an `App` instance and call `run()` to start the TUI.
//!
//! ## Examples
//! ```rust
//! use crate::tui::app::App;
//! let mut app = App::new(config, start_dir, ignore_config).unwrap();
//! app.run().unwrap();
//! ```

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crate::models::AppConfig;
use crate::tui::handlers::{FileOpsHandler, KeyboardHandler};
use crate::tui::state::{AppState, SearchState, SelectionState};
use crate::tui::views::MainView;

/// Main TUI application struct. Manages state, event loop, and UI rendering.
pub struct App {
    /// Application state (files, config, etc.)
    pub state: AppState,

    /// UI state managers
    selection_state: SelectionState,
    search_state: SearchState,

    /// Event handlers
    keyboard_handler: KeyboardHandler,

    /// Views
    main_view: MainView,
    // TODO: Remove any remaining legacy compatibility fields after full migration
}

impl App {
    /// Creates a new `App` instance with the given config, start directory, and ignore config.
    pub fn new(
        config: AppConfig,
        start_dir: PathBuf,
        ignore_config: crate::models::IgnoreConfig,
    ) -> Result<Self, io::Error> {
        let app_state = AppState::new(config.clone(), start_dir, ignore_config)?;

        // Create a new instance with all components initialized
        let app = Self {
            state: app_state,
            selection_state: SelectionState::new(),
            search_state: SearchState::new(),
            keyboard_handler: KeyboardHandler::new(),
            main_view: MainView::new(),
        };

        Ok(app)
    }

    /// Runs the main TUI event loop, handling rendering and user input.
    pub fn run(&mut self) -> io::Result<()> {
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
                self.main_view.render(
                    f,
                    f.size(),
                    &self.state,
                    &mut self.selection_state,
                    &self.search_state,
                )
            })?;

            // Handle events with timeout
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    KeyboardHandler::handle_key(
                        key,
                        &mut self.state,
                        &mut self.selection_state,
                        &mut self.search_state,
                    )?;
                }
            }
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

    // TODO: Remove any remaining legacy delegating methods after full migration
}
