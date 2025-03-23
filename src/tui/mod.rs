// Public modules
pub mod components;
pub mod handlers;
pub mod state;
pub mod views;

// App re-export
mod app;
pub use app::App;

// Internal types for TUI system communication
pub(crate) type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
