Okay, let's analyze the provided Rust codebase for the `aibundle` application and generate a comprehensive feature list.

**Core Functionality:**

1.  **File Aggregation:** Collects content from multiple files and directories based on user specifications.
2.  **Content Formatting:** Formats the aggregated content into various structured output formats.
3.  **Cross-Platform Support:** Designed to run on Windows, macOS, and Linux (including Wayland/X11 and WSL environments).

**Operating Modes:**

1.  **Command-Line Interface (CLI) Mode:**
    *   For non-interactive, batch processing.
    *   Triggered by specific CLI arguments (`--files`, `--output-file`, `--output-console`, `--save-config`).
    *   Parses arguments using `clap`.
2.  **Text-Based User Interface (TUI) Mode:**
    *   Default mode when no specific CLI action arguments are provided.
    *   Provides an interactive file browser and selection interface.
    *   Built using `ratatui` and `crossterm`.

**Output & Formatting:**

1.  **Multiple Output Formats:** Supports formatting aggregated content as:
    *   **XML:** Structured output with `<folder>` and `<file>` tags.
    *   **Markdown:** Code blocks for files (```), headers for folders (`##`).
    *   **JSON:** Structured JSON output representing files, folders, and content.
    *   **LLM (Large Language Model) Format:** A specialized Markdown format designed for AI assistants, including:
        *   Project overview (path, counts, languages).
        *   File tree structure visualization.
        *   Basic dependency analysis (internal/external) based on common import/include patterns for various languages (Python, C/C++, JS/TS, Java, Go, Ruby, PHP, Rust, Swift, Shell, Makefile).
        *   File contents presented with syntax highlighting hints and dependency summaries.
2.  **Output Destinations:**
    *   **Clipboard:** Copies the formatted output directly to the system clipboard (CLI default if no file/console output, TUI via 'c' key).
    *   **File:** Saves the formatted output to a specified file (`--output-file` in CLI).
    *   **Console:** Prints the formatted output directly to the standard output (`--output-console` in CLI).
3.  **Line Numbering:** Option to include line numbers in the formatted output (XML, Markdown, LLM formats). (`--line-numbers` CLI flag, 'n' key in TUI).
4.  **Binary File Handling:**
    *   Detects binary files based on common extensions or names.
    *   Excludes binary files by default.
    *   Option to include placeholders or basic info for binary files (`--include-binary` implied logic, 'b' key in TUI).

**File Handling & Traversal:**

1.  **Directory Traversal:** Can navigate and list files/folders within specified directories.
2.  **Recursive Mode:** Option to traverse directories recursively (`--recursive` CLI flag, 'r' key toggle in TUI).
3.  **Ignore Mechanisms:** Filters files and directories based on:
    *   **.gitignore Rules:** Respects rules found in `.gitignore` files (`--gitignore` CLI flag, 'g' key in TUI).
    *   **Default Patterns:** Ignores common directories like `.git`, `node_modules`, `target`, etc. ('d' key in TUI).
    *   **Custom Patterns:** Allows specifying additional ignore patterns (`--ignore` CLI flag).
4.  **File Filtering:** Allows specifying glob patterns to include only specific files (`--files` CLI flag).
5.  **Search:** Interactive file/folder name search within the TUI ('/' key). Supports basic substring and glob patterns.
6.  **Symlink Loop Detection:** Basic protection against infinite loops caused by symbolic links during traversal.
7.  **Async Operations:** Utilizes asynchronous file operations (`tokio`) for potentially improved performance in some I/O tasks (listing, reading).

**Configuration:**

1.  **TOML Configuration File:** Loads and saves configuration from `.aibundle.config.toml` in the user's home directory.
2.  **Mode-Specific Defaults:** Stores separate default settings for CLI and TUI modes.
3.  **Configurable Options:** Defaults for format, ignore rules, line numbers, recursion, source directory, and selection limits can be saved.
4.  **Save Configuration:** Explicit CLI flag (`--save-config`) to generate or update the configuration file.

**Text-Based User Interface (TUI) Specific Features:**

1.  **Interactive File Browser:** Displays files and folders in a list view.
2.  **Navigation:** Keyboard navigation (`Up`/`Down`, `PgUp`/`PgDn`, `Home`/`End`, `Enter` to open, `Backspace` implied via `..`).
3.  **Selection:**
    *   Toggle individual items (`Space`).
    *   Toggle select all visible (`*`).
    *   Visual indicators for selected items (`[X]`).
    *   Recursive selection/deselection of folder contents.
4.  **Folder Expansion:**
    *   Toggle single-level expansion/collapse (`Tab`).
    *   Toggle recursive expansion/collapse (`Shift+Tab` or `BackTab`).
5.  **Real-time Feedback:** Status bar shows item counts, selection counts, toggle states, and current format.
6.  **Modal Dialogs:** Displays information for:
    *   Copy statistics (file/folder counts, lines, size).
    *   Help screen with keybindings.
    *   Error/Warning messages (e.g., selection limit).
7.  **Selection Limit:** Prevents selecting an excessive number of items (configurable, default 400) with background counting and warning modal.
8.  **Icons:** Displays icons next to files/folders based on type/extension.
9.  **Search Mode:** Interactive filtering of the file list.

**Utilities:**

1.  **Human-Readable Sizes:** Formats byte sizes into KB, MB, etc. (used in copy stats).
2.  **Path Normalization:** Converts paths to use forward slashes internally.
3.  **Basic Logging:** Logs events to a timestamped file per run for debugging.


---

### **Summary Table of Major Features**

| Area                | Details                                                                    |
|---------------------|----------------------------------------------------------------------------|
| Aggregation         | File/folder crawl, recursive, filtering, ignore support, symlink safe      |
| CLI Mode            | Argument parsing, output destination, config merge, full export            |
| TUI Mode            | Interactive selector, keyboard shortcuts, live feedback, search/filtering  |
| Output Formats      | XML, Markdown, JSON, AI/LLM-friendly, with optional line numbers           |
| Clipboard           | Cross-platform copy/paste, internal/exported contents                      |
| Dependency Mapping  | Internal/external import detection, per-file dependency view (LLM mode)    |
| Config System       | TOML config in home, CLI and TUI sections, merge and save capabilities     |
| Status Display      | Modal popups for stats/help/messages, persistent status bar                |

---