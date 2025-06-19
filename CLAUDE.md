# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Build & Run
```bash
# Build the project
cargo build

# Run in TUI mode
cargo run

# Run in CLI mode with arguments  
cargo run -- --files "*.rs" --format llm --output-console

# Run specific directory
cargo run -- /path/to/project
```

### Testing
```bash
# Unit and integration tests
cargo test

# Run specific test
cargo test test_name

# TUI integration tests (requires tmux)
./tests/scripts/run_all_tests.sh

# Individual TUI test
./tests/scripts/test_B1_list_navigation.sh
```

### Code Quality
```bash
# Format code (required before commits)
cargo fmt

# Run linter (fix warnings before commits)
cargo clippy --all-targets --all-features

# Check documentation
cargo doc --no-deps
```

### Configuration & Debugging
```bash
# Save current CLI settings to config
cargo run -- --save-config

# View config location
echo "~/.aibundle.config.toml"

# Check logs (created per run)
ls logs/
```

## Architecture Overview

AIBundle is a dual-mode (CLI/TUI) file bundling tool with the following key components:

### Core Modules
- **`cli/`**: Command-line interface with clap argument parsing
- **`tui/`**: Terminal user interface using ratatui framework  
- **`fs/`**: File system operations with async support and ignore patterns
- **`output/`**: Formatters for XML, JSON, Markdown, and LLM outputs
- **`models/`**: Shared data structures and configuration types
- **`config/`**: TOML-based configuration management
- **`clipboard/`**: Cross-platform clipboard integration

### Key Design Patterns

**Dual-Mode Architecture**: Single binary that detects CLI vs TUI mode based on provided arguments. CLI mode runs linearly, TUI mode enters event loop.

**Shared Core Logic**: Both modes use the same file traversal, filtering, and output formatting code through common interfaces.

**State Management**: TUI uses centralized `AppState` with event-driven updates. CLI merges command-line args with saved configuration.

**Performance Optimizations**: 
- Adaptive polling (60fps active, 2fps idle)
- Background async operations for file counting/clipboard
- Cached regex compilation and filesystem metadata
- Selection limits to prevent memory issues

**Output Strategy Pattern**: Polymorphic formatting via `OutputFormat` enum that dispatches to specific formatters (JSON, Markdown, XML, LLM).

### Configuration System
Uses hierarchical TOML config at `~/.aibundle.config.toml`:
- `[cli]` section for command-line defaults
- `[tui]` section for UI preferences  
- Supports ignore patterns, format preferences, selection limits
- CLI arguments override config file values

### Testing Strategy
- **Unit Tests**: Standard Rust test modules
- **CLI Tests**: Shell scripts testing various flag combinations
- **TUI Tests**: Novel tmux-based automation with screen capture verification
- **Integration**: Real terminal simulation for UI components

## Important Development Notes

### Code Style Requirements
- Follow `.cursor/rules/base-rust.mdc` guidelines strictly
- Use `cargo fmt` and address all `cargo clippy` warnings
- Prefix unused variables with underscore (`_variable`)
- Use `&Path` over `&PathBuf` where possible
- No hardcoded secrets or sensitive data in commits

### Error Handling Patterns
- Use `Result<T, E>` with `?` operator for propagation
- Custom error types using `thiserror` 
- No `.unwrap()` or `.expect()` except in tests
- Provide descriptive error messages with context

### Async Considerations
This project uses Tokio for:
- File system operations (`tokio::fs`)
- Background tasks in TUI mode
- Non-blocking clipboard operations
- Prevent UI freezing during large operations

### TUI-Specific Guidelines
- Components receive state by reference for rendering
- Use event channels for async operation results
- Background tasks should not block the UI thread
- Adaptive polling: reduce frequency when idle (2-second threshold)

### Module Dependencies
- `models/` is foundational - other modules depend on it
- `fs/` provides core file operations for both CLI and TUI
- `output/` formatters are stateless and reusable
- `tui/` modules are interdependent but well-encapsulated

### Binary Detection & Ignore Patterns
- Multi-layered ignore: `.gitignore` + defaults + custom patterns
- Binary detection via magic numbers and extensions
- Symlink loop detection prevents infinite traversal
- Permission handling for restricted directories

### Cross-Platform Considerations
- Uses `crossterm` for terminal manipulation
- Path normalization for Windows/Unix compatibility  
- Clipboard integration varies by platform (xclip, pbcopy, etc.)
- Terminal reset commands are platform-specific

This codebase demonstrates sophisticated Rust patterns while maintaining clear separation between CLI and TUI concerns. The modular architecture makes it easy to extend with new output formats or UI components.