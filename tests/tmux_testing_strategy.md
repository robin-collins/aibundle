Okay, this is a well-structured Rust TUI application with a clear separation of concerns. Based on the project analysis and your TUI features, here's a detailed plan of what should be tested using `tmux` scripting.

The goal is to simulate user key presses and verify the TUI's visual output and behavior in response to those actions.

**I. Core TUI Functionality Tests (using `tmux`)**

For each test, you'll typically:
1.  Start the TUI in a tmux session (`./your_tui_app <optional_start_dir>`).
2.  `sleep` briefly to allow the TUI to initialize.
3.  `tmux send-keys` to simulate user input.
4.  `sleep` briefly after each key press or action to allow the TUI to update.
5.  `tmux capture-pane -p` to get the screen content.
6.  `grep`, `awk`, or other shell tools to assert expected text, UI elements, or absence of errors.
7.  Clean up the tmux session.

**A. Startup and Basic UI Elements**

1.  **Default Startup (current directory):**
    *   Action: Launch app with no arguments.
    *   Verify:
        *   Header shows correct app version (`AIBundle v0.7.0`).
        *   Header shows current directory (e.g., `.` or the actual path).
        *   File list displays items from the current directory.
        *   Status bar shows initial item count, selected count (0), and default toggle states (e.g., `ignores [x]`, `gitignore [x]`, `format [LLM]`, `line numbers [ ]`).
        *   First item in the list is highlighted (or no item if dir is empty).

2.  **Startup with Specific Directory:**
    *   Action: Launch app with a specific directory argument (e.g., `./your_tui_app ./src`).
    *   Verify:
        *   Header shows the specified directory.
        *   File list displays items from that directory.

3.  **Startup with Config File Present:**
    *   Action: Create a `.aibundle.config.toml` with specific TUI settings (e.g., `default_format = "json"`, `default_recursive = true`, specific `source_dir`). Launch app.
    *   Verify:
        *   TUI starts in the `source_dir` specified in the config (if no CLI arg overrides it).
        *   Status bar reflects `default_format` and `default_recursive` from config.
        *   Other relevant settings from the TUI section of the config are applied.

**B. Navigation and File Listing**

1.  **List Navigation:**
    *   Action: `j` or `DownArrow` to move down.
    *   Verify: Highlighted item changes in `capture-pane` output.
    *   Action: `k` or `UpArrow` to move up.
    *   Verify: Highlighted item changes.
    *   Action: `PageDown`.
    *   Verify: Highlighted item jumps down (e.g., by 10 or page height).
    *   Action: `PageUp`.
    *   Verify: Highlighted item jumps up.
    *   Action: `Home`.
    *   Verify: First item highlighted.
    *   Action: `End`.
    *   Verify: Last item highlighted.

2.  **Directory Traversal:**
    *   Action: Navigate to a directory, press `Enter`.
    *   Verify:
        *   Header updates to the new directory path.
        *   File list updates with contents of the new directory.
        *   `../` entry appears in the file list (unless at root).
        *   Selection resets to the first item.
    *   Action: Navigate to `../`, press `Enter`.
    *   Verify:
        *   Header updates to the parent directory path.
        *   File list updates.

3.  **Folder Expansion/Collapse (Recursive Mode - `r`):**
    *   Pre-condition: Ensure `recursive` mode is active (press `r` if needed).
    *   Action: Navigate to a folder, press `Tab`.
    *   Verify:
        *   Folder contents (sub-items) appear indented below the folder in the file list.
        *   The folder icon might change (e.g., to `folder_open`).
    *   Action: Press `Tab` again on the same folder.
    *   Verify: Folder contents are hidden, icon reverts.
    *   Action: Navigate to a folder, press `Shift+Tab` (BackTab).
    *   Verify: Folder and ALL its subfolders expand.
    *   Action: Press `Shift+Tab` again.
    *   Verify: Folder and ALL its subfolders collapse.

4.  **Icons Display:**
    *   Action: Navigate to a directory with various file types (`.rs`, `.md`, `.py`, `.json`, a folder).
    *   Verify: `capture-pane` output shows the correct Unicode icons next to each file/folder name (e.g., `ðŸ¦€` for `.rs`, `ðŸ“ ` for folder).

**C. Selection Mechanics**

1.  **Single Item Selection/Deselection:**
    *   Action: Navigate to an item, press `Space`.
    *   Verify: `[X]` appears next to the item. Status bar "selected" count increments.
    *   Action: Press `Space` again.
    *   Verify: `[ ]` appears next to the item. Status bar "selected" count decrements.

2.  **Select All/Deselect All:**
    *   Action: Press `a`.
    *   Verify: All visible items (excluding `../`) get `[X]`. Status bar "selected" count updates.
    *   Action: Press `a` again.
    *   Verify: All visible items get `[ ]`. Status bar "selected" count becomes 0.

3.  **Folder Selection (and recursive selection):**
    *   Action: Navigate to a folder, press `Space`.
    *   Verify:
        *   The folder itself gets `[X]`.
        *   If recursive selection for folders is implied by `update_folder_selection`, then all items *within* that folder (and subfolders if TUI logic does that automatically for selection) should also be considered selected internally, even if not visually marked `[X]` without expansion. This is harder to test via `capture-pane` directly for unexpanded children but the *total selected count* in status bar should reflect this.
        *   If counting is asynchronous, verify "Counting..." message or modal appears. Then, after a `sleep`, verify the selection and count update.

4.  **Selection Limit:**
    *   Action: Set a low `selection_limit` in config or via a (hypothetical) test setup. Try to select items exceeding this limit (e.g., using `a` on a large directory).
    *   Verify:
        *   A modal/message appears: "Cannot select: would exceed limit...".
        *   The number of selected items does not exceed the limit.

**D. Search Functionality**

1.  **Enter/Exit Search Mode:**
    *   Action: Press `/`.
    *   Verify: Header shows "Search: " and a blinking cursor.
    *   Action: Type a query (e.g., "main").
    *   Verify: Header shows "Search: main_". File list filters to items containing "main".
    *   Action: Press `/` again (or `Enter` as per `keyboard.rs`).
    *   Verify: Search input disappears from header. File list remains filtered. Highlight is on the first match.
    *   Action: Press `/`, then `Esc`.
    *   Verify: Search input disappears. File list reverts to non-filtered view.

2.  **Search Input:**
    *   Action: Press `/`, type "mod", press `Backspace`.
    *   Verify: Header shows "Search: mo_". File list updates.

3.  **Search with No Results:**
    *   Action: Press `/`, type "nonexistentqueryxyz", press `/`.
    *   Verify: File list is empty (or shows "No results"). Status bar item count becomes 0.

4.  **Search in Recursive vs. Non-Recursive Mode:**
    *   Test searching with `recursive` (`r`) toggled on and off to see how `filtered_items` changes.
        *   Non-recursive: Only items in the current view matching.
        *   Recursive: Items from subdirectories matching, and parent folders of matches might become expanded.

**E. Toggling Options & Filters (Verify Status Bar and File List Changes)**

For each toggle, press the key, capture pane, check status bar, press again, capture, check.
1.  **Toggle Default Ignores (`i`):**
    *   Verify: `ignores [x]` changes to `ignores [ ]` in status bar. File list updates (e.g., `.git` might appear/disappear).
2.  **Toggle .gitignore (`g`):**
    *   Verify: `gitignore [x]` changes to `gitignore [ ]`. File list updates if a `.gitignore` file is affecting the view.
3.  **Toggle Binary Files (`b`):**
    *   Verify: `binary [x]` changes to `binary [ ]`. Affects clipboard output more than list view, but status bar should update.
4.  **Toggle Output Format (`f`):**
    *   Verify: `format [LLM]` -> `format [XML]` -> `format [Markdown]` -> `format [JSON]` -> `format [LLM]` in status bar.
5.  **Toggle Line Numbers (`n`):**
    *   Verify: `line numbers [x]` changes to `line numbers [ ]`. Primarily affects clipboard output, but status bar updates. (Should not change if format is JSON).
6.  **Toggle Recursive Mode (`r`):**
    *   Verify: Status bar might indicate recursive mode (if it does). File list loads items recursively or non-recursively. Expanded folders state might change.

**F. Clipboard and Output Operations**

1.  **Copy to Clipboard (`c`):**
    *   Action: Select some files/folders, press `c`.
    *   Verify:
        *   A modal appears: "Copied to clipboard (FORMAT)... Files: X, Folders: Y, Lines: Z, Size: SSS". (Assert the text and rough numbers).
        *   The content of the clipboard (if testable in CI, otherwise manual check or a mock clipboard) contains the formatted output.
        *   Test with each output format (`f` key).
        *   Test with line numbers (`n` key) on/off for relevant formats (XML, MD, LLM).

**G. Help and Modals**

1.  **Show/Hide Help (`h`, `?`, `F1`):**
    *   Action: Press `h`.
    *   Verify: Help modal appears with "Keyboard Controls", keybindings list.
    *   Action: Press `PageDown` in help modal.
    *   Verify: Help content scrolls (if long enough). Text "Page X of Y" updates.
    *   Action: Press `PageUp` in help modal.
    *   Verify: Help content scrolls.
    *   Action: Press `Esc` (or any other key as per help text).
    *   Verify: Help modal disappears.

2.  **Message Display (e.g., after selection limit):**
    *   Action: Trigger an action that shows a message (e.g., try to select too many items).
    *   Verify: Message modal appears with correct text and style (e.g., "ERROR: Selection Limit Reached...").

**H. Configuration Saving (TUI initiated)**

1.  **Save Configuration (`S`):**
    *   Action: Make some changes to toggles (e.g., format, line numbers). Press `S`.
    *   Verify:
        *   If `.aibundle.config.toml` does *not* exist:
            *   Message modal: "Configuration saved successfully to /path/to/.aibundle.config.toml".
            *   File is created with the current TUI settings.
        *   If `.aibundle.config.toml` *does* exist:
            *   Message modal (Warning): "Configuration file already exists...".
            *   File is *not* overwritten (as per `FileOpsHandler` logic).

**I. Quit**

1.  **Quit (`q`):**
    *   Action: Press `q`.
    *   Verify:
        *   If items are selected, a "Copied to clipboard..." modal briefly appears (or stats are printed to stdout after exit as per `App::run`).
        *   TUI exits, terminal returns to normal.
2.  **Quit (Ctrl+C - standard terminal interrupt):**
    *   Action: Send `Ctrl+C`.
    *   Verify: TUI exits cleanly. (This might be harder to distinguish from a crash via tmux scripts alone unless specific exit messages are printed).

**II. CLI Mode Functionality Tests (using `assert_cmd` or similar, not `tmux`)**

While not `tmux`-based, these are crucial for full coverage:
1.  **Basic CLI Operations:**
    *   `./your_tui_app --files "*.rs" --output-console`: Verify Rust files from current dir are printed to stdout in default (LLM) format.
    *   `./your_tui_app --files "*.md" --format markdown --output-file test.md`: Verify `test.md` is created with Markdown content.
    *   `./your_tui_app --files "*.json" --format json --output-console`: Verify JSON output.
    *   `./your_tui_app --files "*.xml" --format xml --output-console`: Verify XML output.
2.  **CLI Option Combinations:**
    *   Test `--recursive`, `--line-numbers`, `--gitignore` (with a test .gitignore file), `--ignore "pattern"`.
    *   Test `--source-dir <path>` and positional `<SOURCE_DIR>`.
    *   Test `--output-file`, `--output-console`, and default clipboard behavior.
3.  **CLI Config Saving (`--save-config`):**
    *   `./your_tui_app --save-config`: Verify `.aibundle.config.toml` is created/updated with default CLI and TUI sections.
    *   `./your_tui_app --save-config --format json --files "*.txt"`: Verify config is saved and then CLI operation *still proceeds* if other CLI flags are present.
4.  **CLI with Existing Config:**
    *   Create a config file with specific CLI defaults (e.g., `format = "xml"`).
    *   Run `./your_tui_app --files "*.log"` (no format specified).
    *   Verify output is XML, respecting the config default.
    *   Run `./your_tui_app --files "*.log" --format markdown`.
    *   Verify output is Markdown, as CLI flag overrides config.
5.  **CLI Search (`--search` - if distinct from `--files`):**
    *   The code shows `files` option used for pattern matching in CLI. If `--search` is intended for content search or a different type of filtering in CLI, that needs tests. Currently, `CliOptions` has `search` but `run_cli_mode` doesn't seem to use it directly for filtering, it uses `files_pattern`. The TUI search is distinct. If CLI search is not implemented, this can be skipped.

**Key Considerations for Tmux Tests:**

*   **Test Data:** Have a dedicated directory with a varied set of files and subdirectories (including a `.gitignore` file) to run tests against.
*   **Timing (`sleep`):** Crucial. Tmux commands are fast, the TUI needs time to process and re-render. `sleep 0.1` to `sleep 1` might be needed.
*   **Screen Size:** Ensure tmux pane size is consistent and large enough to display relevant info. You can set this with `tmux resize-pane`.
*   **Color Codes:** `capture-pane` can include ANSI color codes. You might need to strip them (`sed`) or use tools that can handle them if asserting specific colors. Asserting text content is usually more robust.
*   **Idempotency:** Ensure tests clean up after themselves (tmux sessions, created files like `output.txt` or `.aibundle.config.toml`).
*   **Error Handling:** Test invalid inputs, non-existent directories. The TUI should handle these gracefully (e.g., show an error message in a modal or status bar).
*   **CI Environment:** These tests are excellent for CI. Ensure `tmux` is available in the CI runner.
*   **Logging:** The `log_event` utility is great. Check these log files for more detailed insight if a test fails.

This plan provides a comprehensive set of scenarios. Start with the most critical features and gradually expand coverage. Good luck!