# AIBundle Testing and Feature Checklist

This document provides a comprehensive checklist of features and functions implemented in AIBundle along with steps to test and confirm that each works as expected. The checklist is organized into sections for TUI testing, CLI testing, and common utility functions.

---

## TUI Testing

### User Interface & Navigation
- [ ] **Launch TUI Mode:**  
  - Run `aibundle` (without CLI flags) and verify that the TUI interface loads with the current directory listing.
- [ ] **Title Bar Check:**  
  - Ensure the title bar shows the version (e.g., "v0.6.7") and the current working directory.
- [ ] **List Navigation:**  
  - **Arrow Keys:** Use Up/Down keys to move the selection highlight.  
  - **Page Navigation:** Use PageUp/PageDown keys to jump 10 items at a time.
  - **Enter Key:** Select a directory (or the ".." parent directory) and verify that navigation updates the view accordingly.
  - **Tab Key:** Toggle folder expansion and check that subdirectories are correctly included or hidden.

### Search Functionality
- [ ] **Enter Search Mode:**  
  - Press `/` to enter search mode and see a search prompt with a blinking cursor.
- [ ] **Typing a Query:**  
  - Type in a search query and confirm that the list updates to show only matching records.
- [ ] **Cancel Search:**  
  - Press **Esc** to exit search mode and validate that the original full list is restored.

### Selection & Copy Features
- [ ] **Basic Selection:**  
  - Press **Space** on an item to toggle its selection (visually changing from `[ ]` to `[X]` and vice versa).
- [ ] **Select/Deselect All:**  
  - Press `*` to select or deselect all items (excluding parent directory entries).
- [ ] **Selection Limit:**  
  - Test selecting a directory (or multiple items) that would exceed the selection limit (default 400). Verify that a modal pops up with an error message when the limit is exceeded.
- [ ] **Copy to Clipboard:**  
  - After selecting items, press `c` and ensure:
    - A modal appears showing copy statistics (number of files/folders, total lines, total size).
    - The clipboard contains the correctly formatted content based on the current output format.

### Toggling Options & Settings
- [ ] **Output Format Toggle:**  
  - Press `f` to cycle through output formats (XML, Markdown, JSON). Confirm that the status bar is updated to reflect the new format.
- [ ] **Line Numbers Toggle:**  
  - Press `n` to toggle line numbers (ensure this option is disabled for JSON output).
- [ ] **Ignore Options:**  
  - **Default Ignores:** Press `i` to toggle default ignore patterns.
  - **.gitignore Toggle:** Press `g` to toggle using .gitignore.
  - **Binary Files Inclusion:** Press `b` to enable or disable inclusion of binary files.
- [ ] **Help Modal:**  
  - Press `h` to display the help modal. Verify that it shows usage instructions and that navigation (PageUp/PageDown) operates correctly.  
  - Confirm that any key (outside page navigation) closes the modal.
- [ ] **Save Configuration:**  
  - Press `s` to trigger saving of the configuration. When prompted, confirm or cancel and check that the configuration file (`.aibundle.config`) is created or updated.
- [ ] **Exit TUI:**  
  - Press `q` to quit TUI mode. Verify that if items are selected, the copy function is triggered before quitting.

### Additional TUI Considerations
- [ ] **Directory Navigation:**  
  - Confirm that selecting the parent directory ("..") navigates to the correct parent folder.
- [ ] **Modal Pagination:**  
  - For long modals (like the help view), use PageUp/PageDown keys to cycle through the modal content.

---

## CLI Testing

### Command-Line Options & Output
- [ ] **File Pattern Filtering:**  
  - Run `aibundle --files "*.rs"` and verify that only files matching the pattern are processed.
- [ ] **Search Filtering:**  
  - Run `aibundle --search "test"` to check that the file or content filtering is applied.
- [ ] **Output Format Selection:**  
  - Use `--format` flag with values `xml`, `markdown`, and `json` to verify that output follows the selected format.
- [ ] **Output Destination:**  
  - **Output to File:** Use `--output-file <filename>` and confirm that output is correctly written to the file.
  - **Output to Console:** Use `--output-console` and ensure that content is printed on the terminal.
  - **Clipboard Copy:** When neither output file nor console flag is provided, test that the output is correctly copied to the clipboard.

### Recursive & Non-Recursive Processing
- [ ] **Recursive Mode:**  
  - Ensure that with the default or `--recursive true` flag, the tool processes directories recursively (complete directory tree expanded).
- [ ] **Non-Recursive Mode:**  
  - If available or via configuration, run in non-recursive mode and confirm that only the current directory's items are processed.

### Configuration File Handling
- [ ] **Load Configuration:**  
  - Verify that running the tool with an existing config (e.g., from `~/.aibundle.config.toml`) applies settings for CLI mode.
- [ ] **Save Configuration:**  
  - Run `aibundle --save-config` and confirm that the CLI configuration (including ignore patterns, output format, and recursion settings) is saved correctly.

---

## Common & Utility Function Testing

### File & Directory Processing
- [ ] **Count Selection Items:**  
  - **Synchronous Count:** Test `count_selection_items` with both files and directories. Validate that it returns 1 for files and correctly counts directories.
  - **Early Bail-Out:** For directories with many items, verify that counting bails out early when it exceeds the selection limit.
- [ ] **Asynchronous Count:**  
  - Validate that `count_selection_items_async` returns similar results and respects the provided selection limit without blocking the UI.
- [ ] **Update Folder Selection:**  
  - Test the `update_folder_selection` function to ensure that selecting/deselecting a folder cascades the selection change to all its children.
- [ ] **Output Formatting:**  
  - Confirm that `format_selected_items` generates correct output in XML, Markdown, and JSON. Verify that line numbers are included when toggled on (except in JSON).

### File System Helper Functions (in `src/fs/mod.rs`)
- [ ] **Confirm Overwrite:**  
  - Create a temporary file and test `confirm_overwrite`. Provide both "y" and other responses to ensure it returns the correct boolean value.
- [ ] **List Files:**  
  - Use `list_files` on a directory structure containing folders like `node_modules`, `.git`, and `target`; verify that these are excluded from the results.

---

## Additional Testing Considerations

- [ ] **Performance Testing:**  
  - For directories with a very large number of items, check that the early bail-out in the counting functions works to keep the UI responsive.
- [ ] **Error Handling:**  
  - Simulate error conditions (e.g., permission errors, nonexistent directories) and verify that error messages are displayed appropriately without crashing.
- [ ] **Platform Specific Tests:**  
  - Validate clipboard interactions and file system operations under different operating systems (Windows, macOS, Linux).
- [ ] **User Guidance:**  
  - Ensure that all help messages, modals, and tooltips provide clear and accurate information.

---

This checklist is meant to guide testing efforts to ensure that every feature function provided by AIBundle is thoroughly validated. 