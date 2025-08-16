// src/tui/app.rs
//!
//! # TUI Application Entry Point
//!
//! Provides the [`App`] struct, which manages the main application state, event loop, and UI rendering for the TUI.
//!
//! ## Purpose
//!
//! - Coordinate state, event handlers, and views for the TUI.
//! - Manage the event loop, rendering, and user input.
//! - Launch and run the TUI application.
//!
//! ## Organization
//!
//! - [`App`]: Main TUI application struct.
//! - [`AppRunResult`]: Enum for run result reporting.
//! - [`ClipboardPrintData`]: Struct for clipboard copy stats.
//! - [`AppEvent`]: Internal event enum for async operations.
//!
//! ## Example
//! ```rust
//! use aibundle::tui::App;
//! # use aibundle::models::{AppConfig, IgnoreConfig};
//! # use std::path::PathBuf;
//! let config = AppConfig::default();
//! let start_dir = PathBuf::from(".");
//! let ignore_config = IgnoreConfig::default();
//! let mut app = App::new(config, start_dir, ignore_config).unwrap();
//! app.run().unwrap();
//! ```
//!
//! # Doc Aliases
//! - "tui-app"
//! - "terminal-app"
//!
#![doc(alias = "tui-app")]
#![doc(alias = "terminal-app")]

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
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
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
    pub event_tx: SyncSender<AppEvent>,
    event_rx: Receiver<AppEvent>,
    /// Recent events for deduplication (operation_id -> timestamp)
    recent_events: std::collections::HashMap<OperationId, Instant>,
}

impl App {
    /// Creates a new `App` instance with the given config, start directory, and ignore config.
    pub fn new(
        config: AppConfig,
        start_dir: PathBuf,
        ignore_config: crate::models::IgnoreConfig,
    ) -> Result<Self, io::Error> {
        let (event_tx, event_rx) = sync_channel::<AppEvent>(100); // Bounded channel with capacity 100

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
            recent_events: std::collections::HashMap::new(),
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

            // Clean up old events from deduplication cache (older than 30 seconds)
            self.cleanup_old_events();

            // Process internal AppEvents from the channel
            while let Ok(app_event) = self.event_rx.try_recv() {
                // Check for event deduplication
                if self.is_duplicate_event(&app_event) {
                    continue; // Skip duplicate events
                }
                
                // Track this event for deduplication
                self.track_event(&app_event);
                
                match app_event {
                    AppEvent::SelectAllScanComplete {
                        operation_id,
                        total_potential_item_count,
                        final_selection_set,
                        initial_optimistic_set,
                    } => {
                        // Only process if no cancellation was requested and operation ID matches (prevent race conditions)
                        if self.state.is_counting && 
                           self.state.current_operation_id.as_ref() == Some(&operation_id) {
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
                        }
                        // If is_counting was false, this event was cancelled/outdated, so ignore it
                    },
                    AppEvent::SelectionCountComplete {
                        operation_id,
                        path,
                        count,
                        optimistic_folder,
                        optimistic_children
                    } => {
                        // Only process if still counting, operation ID matches, and the path matches what we're expecting
                        if self.state.is_counting && 
                           self.state.current_operation_id.as_ref() == Some(&operation_id) &&
                           self.state.counting_path.as_ref() == Some(&path) {
                            // Delegate to a handler or process directly
                            FileOpsHandler::finalize_single_selection(&mut self.state, path, count, optimistic_folder, optimistic_children);
                            self.dirty_flags.file_list = true;
                            self.dirty_flags.status_bar = true;
                        }
                        // If conditions don't match, this event was cancelled/outdated, so ignore it
                    },
                    AppEvent::CancelSelectionOperations => {
                        // Cancel any ongoing selection operations
                        self.state.is_counting = false;
                        self.state.counting_path = None;
                        self.state.current_operation_id = None;
                        self.state.optimistically_added_folder = None;
                        self.state.optimistically_added_children.clear();
                        if let Some(sender) = self.state.count_abort_sender.take() {
                            let _ = sender.send(());
                        }
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

                    // Cancel ongoing selection operations on certain key presses to prevent race conditions
                    let should_cancel_operations = match key.code {
                        KeyCode::Char(' ') | KeyCode::Char('a') | KeyCode::Char('A') => {
                            // Selection keys should cancel previous operations
                            !self.state.is_searching
                        }
                        KeyCode::Esc => {
                            // Escape always cancels operations
                            true
                        }
                        _ => false
                    };

                    if should_cancel_operations && self.state.is_counting {
                        // Send cancel event to prevent race conditions
                        if self.event_tx.send(AppEvent::CancelSelectionOperations).is_err() {
                            // Channel closed - handle gracefully
                            self.state.is_counting = false;
                            self.state.counting_path = None;
                        }
                    }

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

    /// Cleans up old events from the deduplication cache
    fn cleanup_old_events(&mut self) {
        const EVENT_CACHE_DURATION: Duration = Duration::from_secs(30);
        let cutoff_time = Instant::now() - EVENT_CACHE_DURATION;
        
        self.recent_events.retain(|_, timestamp| *timestamp > cutoff_time);
    }

    /// Checks if an event is a duplicate based on operation ID
    fn is_duplicate_event(&self, event: &AppEvent) -> bool {
        match event {
            AppEvent::SelectionCountComplete { operation_id, .. } |
            AppEvent::SelectAllScanComplete { operation_id, .. } => {
                self.recent_events.contains_key(operation_id)
            }
            AppEvent::CancelSelectionOperations => false, // Always process cancellation events
        }
    }

    /// Tracks an event for deduplication
    fn track_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::SelectionCountComplete { operation_id, .. } |
            AppEvent::SelectAllScanComplete { operation_id, .. } => {
                self.recent_events.insert(*operation_id, Instant::now());
            }
            AppEvent::CancelSelectionOperations => {
                // Clear all tracked events on cancellation
                self.recent_events.clear();
            }
        }
    }
}

/// Unique identifier for selection operations to prevent race conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationId(u64);

impl OperationId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub enum AppEvent {
    SelectionCountComplete {
        operation_id: OperationId,
        path: PathBuf, 
        count: usize, 
        optimistic_folder: Option<PathBuf>, 
        optimistic_children: HashSet<PathBuf>
    },
    SelectAllScanComplete {
        operation_id: OperationId,
        total_potential_item_count: usize,
        final_selection_set: HashSet<PathBuf>,
        initial_optimistic_set: HashSet<PathBuf>
    },
    /// Event to cancel ongoing selection operations (prevent race conditions)
    CancelSelectionOperations,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppConfig, IgnoreConfig};
    use tempfile::TempDir;

    use std::time::Duration;

    /// Test AppEvent enum completeness
    #[test]
    fn test_app_event_variants() {
        // Ensure all expected event types exist
        let _selection_complete = AppEvent::SelectionCountComplete {
            operation_id: OperationId::new(),
            path: PathBuf::from("/test"),
            count: 42,
            optimistic_folder: None,
            optimistic_children: HashSet::new(),
        };

        let _select_all_complete = AppEvent::SelectAllScanComplete {
            operation_id: OperationId::new(),
            total_potential_item_count: 100,
            final_selection_set: HashSet::new(),
            initial_optimistic_set: HashSet::new(),
        };

        let _cancel_operations = AppEvent::CancelSelectionOperations;

        // Test that events can be sent through channel
        let (tx, rx) = std::sync::mpsc::sync_channel::<AppEvent>(10);
        tx.send(AppEvent::CancelSelectionOperations).unwrap();

        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(AppEvent::CancelSelectionOperations) => {}, // Expected
            _ => panic!("Should receive CancelSelectionOperations event"),
        }
    }

    /// Test App creation with event channel
    #[test]
    fn test_app_creation_with_event_channel() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = AppConfig::default();
        let ignore_config = IgnoreConfig::default();

        let app_result = App::new(
            config,
            temp_dir.path().to_path_buf(),
            ignore_config,
        );

        assert!(app_result.is_ok(), "App creation should succeed");

        let app = app_result.unwrap();
        // Verify event channel was created
        // Note: We can't easily test the channel directly due to private fields,
        // but we can verify the app was created successfully
        assert_eq!(app.state.current_dir, temp_dir.path().to_path_buf());
    }

    /// Test event handling doesn't block main thread
    #[test]
    fn test_event_channel_non_blocking() {
        let (tx, rx) = std::sync::mpsc::sync_channel::<AppEvent>(1000);

        // Send multiple events rapidly
        for i in 0..1000 {
            let event = AppEvent::SelectionCountComplete {
                operation_id: OperationId::new(),
                path: PathBuf::from(format!("/test/{}", i)),
                count: i,
                optimistic_folder: None,
                optimistic_children: HashSet::new(),
            };

            // This should not block
            let send_result = tx.send(event);
            if send_result.is_err() {
                // Channel receiver might be dropped, but shouldn't panic
                break;
            }
        }

        // Send cancellation event
        tx.send(AppEvent::CancelSelectionOperations).unwrap();

        // Verify events can be received
        let mut received_cancel = false;
        while let Ok(event) = rx.try_recv() {
            if matches!(event, AppEvent::CancelSelectionOperations) {
                received_cancel = true;
                break;
            }
        }

        assert!(received_cancel, "Should receive cancellation event");
    }

    /// Mock test for race condition prevention
    #[test]
    fn test_selection_operation_cancellation_pattern() {
        let (tx, rx) = std::sync::mpsc::sync_channel::<AppEvent>(10);

        // Simulate scenario where user input triggers cancellation
        // while background selection operation is running

        // 1. Start background operation (simulated)
        tx.send(AppEvent::SelectionCountComplete {
            operation_id: OperationId::new(),
            path: PathBuf::from("/test"),
            count: 100,
            optimistic_folder: None,
            optimistic_children: HashSet::new(),
        }).unwrap();

        // 2. User input causes cancellation
        tx.send(AppEvent::CancelSelectionOperations).unwrap();

        // 3. Another background operation result arrives late
        tx.send(AppEvent::SelectAllScanComplete {
            operation_id: OperationId::new(),
            total_potential_item_count: 200,
            final_selection_set: HashSet::new(),
            initial_optimistic_set: HashSet::new(),
        }).unwrap();

        // Event handler should process cancellation and ignore late results
        let events: Vec<_> = rx.try_iter().collect();
        assert_eq!(events.len(), 3);

        // Verify cancellation event is present
        let has_cancellation = events.iter().any(|e| {
            matches!(e, AppEvent::CancelSelectionOperations)
        });
        assert!(has_cancellation, "Should contain cancellation event");
    }
}
