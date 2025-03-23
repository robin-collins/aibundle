# Error & Warning Analysis Report

Here's a breakdown of the files that need updating to resolve the errors and warnings from the Clippy output:

## 1. `src/tui/state/selection.rs`
- **Error**: Import conflict between `std::fs` and `crate::fs` (E0252)
- **Error**: Cannot find `count_selection_items_async` function in the `fs` module (E0425)
- **Warnings**: Unused imports for `HashSet`, `Path`, `crate::fs`, and `IgnoreConfig`

## 2. `src/cli/options.rs`
- **Error**: Unresolved import `crate::VERSION` (E0432) 
- **Warnings**: Unused imports `AppConfig`, `FullConfig`, `IgnoreConfig`, and `Path`

## 3. `src/tui/views/help_view.rs`, `src/tui/views/main_view.rs`, `src/tui/views/message_view.rs`
- **Error**: Unresolved import `ratatui::text::Spans` - this type doesn't exist in the module (E0432)

## 4. `src/tui/views/message_view.rs`
- **Error**: Failed to resolve `MessageType` in `state` module (E0433) - referenced multiple times
- **Error**: No field `message` on `AppState` (E0609)

## 5. `src/tui/handlers/clipboard.rs`
- **Error**: No field `modal` on `AppState` (E0609)
- **Error**: No method `is_path_ignored` on `AppState` (E0599)
- **Error**: Function call mismatches - wrong argument types (E0061) for multiple functions
- **Error**: Borrow of moved value `name` (E0382)
- **Warning**: Unused variables and imports

## 6. `src/tui/handlers/file_ops.rs`
- **Error**: Cannot find types `SearchState` and `SelectionState` (E0412)
- **Warnings**: Multiple unused variables (app_state parameters) and imports

## 7. `src/tui/handlers/keyboard.rs`
- **Error**: Match arms have incompatible types - one returns `()` and another `Result<(), Error>` (E0308)
- **Error**: No function `toggle_folder_expansion` found in `FileOpsHandler` (E0599)

## 8. `src/tui/views/main_view.rs`
- **Error**: Wrong number of arguments for `FileList::new()` (E0061)
- **Error**: Wrong number of arguments for `render` methods (E0061)
- **Error**: No fields `is_regex`, `query`, `show_help`, `show_message`, or `modal` on `AppState` (E0609)
- **Error**: Private module `constants` (E0603)
- **Warnings**: Unused imports

## 9. `src/tui/app.rs`
- **Error**: No `clone` method for `AppConfig` - missing `Clone` implementation (E0599)
- **Error**: Wrong number of arguments for `render` method (E0061)
- **Warnings**: Unused imports

## 10. `src/config/mod.rs`
- **Warning**: Collapsible if statement

## 11. `src/output/format.rs` and `src/output/json.rs`
- **Warnings**: Collapsible else-if blocks and unused imports

## 12. `src/models/app_config.rs`
- **Warnings**: Unused imports `HashSet` and `PathBuf`

## 13. `src/output/mod.rs`
- **Warning**: Unused import `format::*`

## 14. `src/tui/components/status_bar.rs`
- **Warnings**: Unused imports `Paragraph`, `Alignment`, and `Style`

## 15. `src/main.rs`
- **Warnings**: Unused imports related to terminal functionality

Most critical issues to fix:
1. The import conflicts in `selection.rs`
2. Missing `MessageType` in the state module
3. Missing fields/methods on `AppState` in various views
4. Wrong function call signatures in `clipboard.rs`
5. Missing `Clone` implementation for `AppConfig`
6. Incompatible type returns in `keyboard.rs`

Would you like me to focus on fixing any specific file or error first?
