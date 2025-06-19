.
├── rules
│  └── linting-errors.md
├── src
│  ├── cli
│  │  ├── mod.rs
│  │  └── options.rs
│  ├── clipboard
│  │  └── mod.rs
│  ├── config
│  │  └── mod.rs
│  ├── fs
│  │  └── mod.rs
│  ├── models
│  │  ├── app_config.rs
│  │  ├── constants.rs
│  │  ├── enums.rs
│  │  └── mod.rs
│  ├── output
│  │  ├── format.rs
│  │  ├── json.rs
│  │  ├── llm.rs
│  │  ├── markdown.rs
│  │  ├── mod.rs
│  │  └── xml.rs
│  ├── tui
│  │  ├── components
│  │  │  ├── file_list.rs
│  │  │  ├── header.rs
│  │  │  ├── modal.rs
│  │  │  ├── status_bar.rs
│  │  │  └── mod.rs
│  │  ├── handlers
│  │  │  ├── clipboard.rs
│  │  │  ├── file_ops.rs
│  │  │  ├── keyboard.rs
│  │  │  ├── mod.rs
│  │  │  └── search.rs
│  │  ├── state
│  │  │  ├── app_state.rs
│  │  │  ├── mod.rs
│  │  │  ├── search.rs
│  │  │  └── selection.rs
│  │  ├── views
│  │  │  ├── help_view.rs
│  │  │  ├── main_view.rs
│  │  │  ├── message_view.rs
│  │  │  └── mod.rs
│  │  ├── app.rs
│  │  └── mod.rs
│  ├── utils
│  │  └── mod.rs
│  └── main.rs
├── Cargo.lock
├── Cargo.toml
├── CLAUDE-RULES.md
├── CLAUDE.md
├── FILETREE.MD
├── FILETREE.md
└── last-hurrah.md

File List:
src/cli/mod.rs
src/cli/options.rs
src/clipboard/mod.rs
src/config/mod.rs
src/fs/mod.rs
src/main.rs
src/models/app_config.rs
src/models/constants.rs
src/models/enums.rs
src/models/mod.rs
src/output/format.rs
src/output/json.rs
src/output/llm.rs
src/output/markdown.rs
src/output/mod.rs
src/output/xml.rs
src/tui/app.rs
src/tui/components/file_list.rs
src/tui/components/header.rs
src/tui/components/modal.rs
src/tui/components/status_bar.rs
src/tui/components/mod.rs
src/tui/handlers/clipboard.rs
src/tui/handlers/file_ops.rs
src/tui/handlers/keyboard.rs
src/tui/handlers/mod.rs
src/tui/handlers/search.rs
src/tui/mod.rs
src/tui/state/app_state.rs
src/tui/state/mod.rs
src/tui/state/search.rs
src/tui/state/selection.rs
src/tui/views/help_view.rs
src/tui/views/main_view.rs
src/tui/views/message_view.rs
src/tui/views/mod.rs
src/utils/mod.rs

36 Files
13 Directories

# Project File Tree (updated)

- src/
  - main.rs    # Now includes comprehensive crate-level and public item Rustdoc per rustdoc.mdc
  - lib.rs     # Now includes comprehensive crate-level and public item Rustdoc per rustdoc.mdc
  ...

## Notes
- `src/main.rs` and `src/lib.rs` now serve as documentation exemplars, strictly following the `rustdoc.mdc` style guide with advanced formatting, doctests, and discoverability improvements.

- All `mod.rs` and `options.rs` files now feature comprehensive, professional, rustdoc.mdc-compliant documentation:
  - Expanded module-level documentation for each core module (CLI, config, fs, clipboard, models, tui, utils)
  - All public structs, fields, and functions are documented
  - Doc aliases added for discoverability
  - Consistent markdown formatting and compiling usage examples throughout

# Documentation Update

All major Rust source modules in `src/output/` and `src/models/` have received a comprehensive documentation overhaul. The following files now strictly adhere to the `rustdoc.mdc` style guide, with improved module-level and item-level documentation, field descriptions, markdown formatting, doc aliases, and realistic compiling examples:

- `src/output/format.rs`
- `src/output/json.rs`
- `src/output/llm.rs`
- `src/output/markdown.rs`
- `src/output/mod.rs`
- `src/output/xml.rs`
- `src/models/app_config.rs`
- `src/models/constants.rs`
- `src/models/enums.rs`
- `src/models/mod.rs`

This update significantly improves documentation quality and discoverability for all core data models and output formatting modules.

> All TUI modules are now documented to the highest standards, following the `rustdoc.mdc` style guide for clarity, discoverability, and maintainability.

- All TUI handler and state modules (`src/tui/handlers/clipboard.rs`, `file_ops.rs`, `keyboard.rs`, `search.rs`, `mod.rs`; `src/tui/state/app_state.rs`, `search.rs`, `selection.rs`, `mod.rs`) now include comprehensive Rustdoc documentation:
  - Module-level documentation describing purpose, organization, and usage.
  - All public structs, fields, and functions are documented with detailed descriptions, argument/return/panic/error/safety sections, and realistic, compiling code examples.
  - Documentation follows the `rustdoc.mdc` style guide for clarity, consistency, and discoverability.

# FILETREE

## Documentation Updates
- The following files have received comprehensive Rustdoc documentation improvements, adhering to the advanced documentation style guide:
  - src/tui/views/help_view.rs
  - src/tui/views/main_view.rs
  - src/tui/views/message_view.rs
  - src/tui/views/mod.rs
  - src/utils/mod.rs

## File Modifications (No Structural Changes)
- `src/tui/state/app_state.rs`: Modified to derive `Default` trait for `Trie` struct.
- `src/tui/state/search.rs`: Modified to derive `Default` trait for `SearchState` struct.