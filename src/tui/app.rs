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
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::collections::HashSet;

use crate::models::AppConfig;
use crate::tui::handlers::{ClipboardHandler, FileOpsHandler, KeyboardHandler};
use crate::tui::state::{AppState, MessageType, SearchState, SelectionState};
use crate::tui::views::MainView;

/// UI activity state for adaptive polling optimization
#[derive(Debug, Clone, PartialEq)]
enum ActivityState {
    /// High activity - user is actively interacting (60fps polling)
    Active,
    /// Low activity - UI is idle (slower polling to save CPU)
    Idle,
}

/// UI component dirty flags for partial redraw optimization
#[derive(Debug, Clone, Default)]
struct DirtyFlags {
    /// File list needs redrawing
    file_list: bool,
    /// Status bar needs redrawing
    status_bar: bool,
    /// Header needs redrawing
    header: bool,
    /// Modal needs redrawing
    modal: bool,
    /// Search bar needs redrawing
    search_bar: bool,
    /// Force full redraw (fallback)
    full_redraw: bool,
}

impl DirtyFlags {
    /// Mark all components as needing redraw
    fn mark_all_dirty(&mut self) {
        self.file_list = true;
        self.status_bar = true;
        self.header = true;
        self.modal = true;
        self.search_bar = true;
        self.full_redraw = true;
    }

    /// Check if any component needs redrawing
    fn needs_redraw(&self) -> bool {
        self.file_list
            || self.status_bar
            || self.header
            || self.modal
            || self.search_bar
            || self.full_redraw
    }

    /// Clear all dirty flags
    fn clear(&mut self) {
        self.file_list = false;
        self.status_bar = false;
        self.header = false;
        self.modal = false;
        self.search_bar = false;
        self.full_redraw = false;
    }
}

/// Data to be printed to the terminal after the TUI exits, if a copy occurred.
#[derive(Clone, Debug)]
pub struct ClipboardPrintData {
    pub files: usize,
    pub folders: usize,
    pub line_count: usize,
    pub byte_size: usize,
}

/// Result of App::run(), indicating what action (if any) to report to the terminal.
#[derive(Clone, Debug)]
pub enum AppRunResult {
    NoAction,
    CopyBlockedByLimit,
    Copied(ClipboardPrintData),
}

/// Main TUI application struct. Manages state, event loop, and UI rendering.
pub struct App {
    /// Application state (files, config, etc.)
    pub state: AppState,

    /// UI state managers
    selection_state: SelectionState,
    search_state: SearchState,

    /// Event handlers
    #[allow(dead_code)]
    keyboard_handler: KeyboardHandler,

    /// Views
    main_view: MainView,

    /// Performance optimizations
    /// Current UI activity state for adaptive polling
    activity_state: ActivityState,
    /// Last activity timestamp for idle detection
    last_activity: Instant,
    /// UI component dirty flags for partial redraws
    dirty_flags: DirtyFlags,
    /// Channel for async clipboard operations
    clipboard_tx: Option<mpsc::UnboundedSender<Result<(), String>>>,
    clipboard_rx: Option<mpsc::UnboundedReceiver<Result<(), String>>>,
    /// Add event channel fields to App struct
    #[allow(dead_code)]
    pub event_tx: Sender<AppEvent>,
    event_rx: Receiver<AppEvent>,
}

impl App {
    /// Creates a new `App` instance with the given config, start directory, and ignore config.
    pub fn new(
        config: AppConfig,
        start_dir: PathBuf,
        ignore_config: crate::models::IgnoreConfig,
    ) -> Result<Self, io::Error> {
        let (event_tx, event_rx) = channel::<AppEvent>();

        let app_state = AppState::new(config.clone(), start_dir, ignore_config.clone(), event_tx.clone())?;

        let (clipboard_tx, clipboard_rx) = tokio::sync::mpsc::unbounded_channel::<Result<(), String>>();

        let app = Self {
            state: app_state,
            selection_state: SelectionState::new(),
            search_state: SearchState::new(),
            keyboard_handler: KeyboardHandler::new(),
            main_view: MainView::new(),
            activity_state: ActivityState::Active,
            last_activity: Instant::now(),
            dirty_flags: DirtyFlags::default(),
            clipboard_tx: Some(clipboard_tx),
            clipboard_rx: Some(clipboard_rx),
            event_tx,
            event_rx,
        };

        Ok(app)
    }

    /// Runs the main TUI event loop, handling rendering and user input.
    pub fn run(&mut self) -> io::Result<AppRunResult> {
        // Setup terminal
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Load initial items and mark UI as needing full redraw
        FileOpsHandler::load_items(&mut self.state)?;
        self.dirty_flags.mark_all_dirty();

        // Main event loop with performance optimizations
        while !self.state.quit {
            // Check for completed async clipboard operations
            self.check_clipboard_results();

            // Update activity state based on time since last activity
            self.update_activity_state();

            // Process internal AppEvents from the channel
            while let Ok(app_event) = self.event_rx.try_recv() {
                // TODO: Implement comprehensive AppEvent handling here
                // For now, just a placeholder. This is where SelectAllScanComplete would be handled.
                match app_event {
                    AppEvent::SelectAllScanComplete {
                        total_potential_item_count,
                        final_selection_set,
                        initial_optimistic_set,
                    } => {
                        // TODO: Consider if this logic should be in AppState or a handler
                        self.state.is_counting = false; // Always reset the counting flag

                        if initial_optimistic_set.len() > self.state.selection_limit && final_selection_set.is_empty() {
                             self.state.set_message(
                                format!(
                                    "Selection limit ({}) exceeded. {} items were attempted. Selection reverted to initially visible items.",
                                    self.state.selection_limit, total_potential_item_count
                                ),
                                MessageType::Warning,
                            );
                        } else {
                            self.state.selected_items = final_selection_set;
                            self.state.selection_is_over_limit = false;
                        }
                        self.dirty_flags.file_list = true; // Mark for redraw
                        self.dirty_flags.status_bar = true;
                    },
                    AppEvent::SelectionCountComplete(path, count, opt_folder, opt_children) => {
                        // Delegate to a handler or process directly
                        // This is effectively what FileOpsHandler::check_pending_selection was doing with app_state.pending_count
                        // If we centralize event handling, check_pending_selection might change or be removed.
                        // For now, this event might be redundant if check_pending_selection is still active.
                        // However, if toggle_selection now sends this event via app_state.tx, this is the place to handle it.
                        FileOpsHandler::finalize_single_selection(&mut self.state, path, count, opt_folder, opt_children);
                        self.dirty_flags.file_list = true;
                        self.dirty_flags.status_bar = true;
                    },
                }
            }

            // Only render if UI components need updating (partial redraw optimization)
            if self.dirty_flags.needs_redraw() {
                terminal.draw(|f| {
                    self.main_view.render(
                        f,
                        f.area(),
                        &self.state,
                        &mut self.selection_state,
                        &self.search_state,
                    )
                })?;
                // Clear dirty flags after successful render
                self.dirty_flags.clear();
            }

            // Adaptive polling based on activity state
            let poll_timeout = self.get_adaptive_poll_timeout();

            // Handle events with adaptive timeout
            if event::poll(poll_timeout)? {
                if let Event::Key(key) = event::read()? {
                    // Mark activity and update UI state
                    self.mark_activity();
                    self.mark_ui_dirty_for_input(&key);

                    // Gate copy operations
                    let is_copy_key = (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('q'))
                                      && !self.state.is_searching;

                    if is_copy_key && self.state.selection_is_over_limit {
                        self.state.set_message(
                            format!("Selection over limit ({}) items. Deselect items to copy.", self.state.selected_items.len()),
                            MessageType::Error
                        );
                        if key.code == KeyCode::Char('q') { // Still allow quit for 'q', but no copy
                             self.state.quit = true;
                        }
                    } else if key.code == KeyCode::Char('c') && !self.state.is_searching {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.copy_selected_to_clipboard_async()?;
                            self.state.quit = true;
                        } else {
                            self.copy_selected_to_clipboard_async()?;
                        }
                    } else if key.code == KeyCode::Char('q') && !self.state.is_searching {
                        self.copy_selected_to_clipboard_async()?;
                        self.state.quit = true;
                    } else {
                        // Handle other keys with standard keyboard handler
                        KeyboardHandler::handle_key(
                            key,
                            &mut self.state,
                            &mut self.selection_state,
                            &mut self.search_state,
                        )?;
                    }
                }
            }
        }

        // Restore terminal
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        disable_raw_mode()?; // Explicitly disable raw mode here, immediately after leaving alternate screen

        // Determine what to report back to main for printing
        if self.state.selection_is_over_limit && self.state.last_copy_stats.is_some() {
            Ok(AppRunResult::CopyBlockedByLimit)
        } else if let Some(stats) = &self.state.last_copy_stats {
            // Attempt to get clipboard contents to calculate line_count and byte_size
            // If this fails, we might not have anything to report or report with 0 lines/bytes.
            // For now, let's assume if last_copy_stats is Some, we want to report something.
            match crate::clipboard::get_clipboard_contents() {
                Ok(contents) => {
                    let line_count = contents.lines().count();
                    let byte_size = contents.len();
                    Ok(AppRunResult::Copied(ClipboardPrintData {
                        files: stats.files,
                        folders: stats.folders,
                        line_count,
                        byte_size,
                    }))
                }
                Err(_) => {
                    // If we can't get clipboard contents, but stats exist,
                    // report with 0 lines/bytes. Or decide if this case means NoAction.
                    // For now, reporting with what we have from CopyStats.
                    Ok(AppRunResult::Copied(ClipboardPrintData {
                        files: stats.files,
                        folders: stats.folders,
                        line_count: 0, // Or handle error differently
                        byte_size: 0,  // Or handle error differently
                    }))
                }
            }
        } else {
            Ok(AppRunResult::NoAction)
        }
    }

    /// Updates activity state based on time since last user interaction
    fn update_activity_state(&mut self) {
        const IDLE_THRESHOLD_MS: u64 = 2000; // 2 seconds of inactivity = idle

        let elapsed = self.last_activity.elapsed();
        let new_state = if elapsed.as_millis() > IDLE_THRESHOLD_MS as u128 {
            ActivityState::Idle
        } else {
            ActivityState::Active
        };

        // Only update if state changed to avoid unnecessary work
        if new_state != self.activity_state {
            self.activity_state = new_state;
        }
    }

    /// Returns adaptive poll timeout based on current activity state
    fn get_adaptive_poll_timeout(&self) -> Duration {
        match self.activity_state {
            // Active state: 60fps polling (16ms) for responsive UI
            ActivityState::Active => Duration::from_millis(16),
            // Idle state: 2fps polling (500ms) to save CPU
            ActivityState::Idle => Duration::from_millis(500),
        }
    }

    /// Marks user activity and updates timestamp
    fn mark_activity(&mut self) {
        self.last_activity = Instant::now();
        if self.activity_state != ActivityState::Active {
            self.activity_state = ActivityState::Active;
        }
    }

    /// Marks UI components as dirty based on input type for partial redraws
    fn mark_ui_dirty_for_input(&mut self, key: &crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        match key.code {
            // Navigation keys affect file list and status bar
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End => {
                self.dirty_flags.file_list = true;
                self.dirty_flags.status_bar = true;
            }
            // Selection keys affect file list and status bar
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.dirty_flags.file_list = true;
                self.dirty_flags.status_bar = true;
            }
            // Search keys affect search bar and file list
            KeyCode::Char('/') | KeyCode::Esc => {
                self.dirty_flags.search_bar = true;
                self.dirty_flags.file_list = true;
                self.dirty_flags.status_bar = true;
            }
            // Typing in search affects search bar and file list
            KeyCode::Char(_) | KeyCode::Backspace if self.state.is_searching => {
                self.dirty_flags.search_bar = true;
                self.dirty_flags.file_list = true;
            }
            // Format toggle affects status bar
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.dirty_flags.status_bar = true;
            }
            // Copy operations may show modal
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.dirty_flags.modal = true;
                self.dirty_flags.status_bar = true;
            }
            // Help toggle affects entire UI
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.dirty_flags.mark_all_dirty();
            }
            // Default: mark minimal components dirty
            _ => {
                self.dirty_flags.status_bar = true;
            }
        }
    }

    /// Checks for completed async clipboard operations
    fn check_clipboard_results(&mut self) {
        if let Some(rx) = &mut self.clipboard_rx {
            // Non-blocking check for clipboard operation results
            while let Ok(result) = rx.try_recv() {
                match result {
                    Ok(()) => {
                        // Clipboard operation succeeded - mark modal dirty to show success
                        self.dirty_flags.modal = true;
                        self.dirty_flags.status_bar = true;
                    }
                    Err(error_msg) => {
                        // Clipboard operation failed - show error message
                        self.state.set_message(
                            format!("Clipboard operation failed: {}", error_msg),
                            MessageType::Warning,
                        );
                        self.dirty_flags.modal = true;
                        self.dirty_flags.status_bar = true;
                    }
                }
            }
        }
    }

    /// Initiates async clipboard copy operation to prevent UI blocking
    pub fn copy_selected_to_clipboard_async(&mut self) -> io::Result<()> {
        // Selection state is already managed in app_state.selected_items by the selection handlers
        // No synchronization needed as the selection logic updates app_state directly

        // Get formatted output for clipboard
        let (output, stats) = ClipboardHandler::format_selected_items(&self.state)?;

        // Update UI state immediately with stats
        let line_count = output.lines().count();
        let byte_size = output.len();
        self.state.modal = Some(crate::tui::components::Modal::copy_stats(
            stats.files,
            stats.folders,
            line_count,
            byte_size,
            &self.state.output_format,
        ));
        self.state.last_copy_stats = Some(stats);

        // Mark UI components as dirty
        self.dirty_flags.modal = true;
        self.dirty_flags.status_bar = true;

        // Send clipboard operation to background thread if channel is available
        if let Some(tx) = &self.clipboard_tx {
            let tx_clone = tx.clone();
            let output_clone = output.clone();
            tokio::spawn(async move {
                let result = match crate::clipboard::copy_to_clipboard(&output_clone) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(format!("Failed to copy to clipboard: {}", e)),
                };
                let _ = tx_clone.send(result);
            });
        } else {
            // Fallback to synchronous operation if async channel not available
            return crate::clipboard::copy_to_clipboard(&output);
        }

        Ok(())
    }
}

pub enum AppEvent {
    SelectionCountComplete(PathBuf, usize, Option<PathBuf>, HashSet<PathBuf>),
    SelectAllScanComplete {
        total_potential_item_count: usize,
        final_selection_set: HashSet<PathBuf>,
        initial_optimistic_set: HashSet<PathBuf>
    },
}
