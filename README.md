# AIBundle 📦

[![Version](https://img.shields.io/badge/version-0.7.0-blue.svg)](https://crates.io/crates/aibundle)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A powerful CLI and TUI tool for bundling files and directories into AI/LLM-friendly formats. Perfect for sharing code snippets and project structures with AI assistants.

![AIBundle Screenshot](screenshot.png)

## AIBundle Format Specification v1 📋

AIBundle is a multi-format specification designed for optimal code sharing with Large Language Models:

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

### Core Features
- **Multiple Representations**: Each bundle can be exported as:
  - XML (hierarchical and structured)
  - Markdown (human-readable and LLM-friendly)
  - JSON (parseable and programmatic)
  - LLM (enhanced AI-optimized format with dependency analysis)
- **File Hierarchy**: Preserves directory structure and relationships
- **Content Preservation**: Maintains original file content and formatting
- **Binary Handling**: Identifies and marks binary files
- **Path Normalization**: Uses forward slashes for cross-platform compatibility
- **Line Numbers**: Optional line numbering for precise code references
- **Selection Limits**: Smart handling of large directories with selection limits (default: 400 files)
- **Async Operations**: Non-blocking UI during large operations, utilizes asynchronous file operations (`tokio`) for improved performance
- **Command Line Mode**: Full CLI support with config persistence
- **Enhanced Dependency Analysis**: Advanced file relationship detection (LLM format), featuring:
  - Multi-language import/include pattern recognition (Python, Rust, JS/TS, C/C++, Java, Go, Ruby, PHP, Swift, shell scripts)
  - Accurate internal vs external dependency categorization
  - Intelligent file path resolution with multiple variation matching
  - Line-by-line parsing for improved accuracy
- **Symlink Loop Detection**: Protects against infinite loops caused by symbolic links during traversal
- **Basic Logging**: Logs events to a timestamped file per run for debugging

### Format Examples

#### XML Format
```xml
<file name="src/main.rs">
// File contents here
</file>
<folder name="src/lib">
  <file name="src/lib/mod.rs">
  // File contents here
  </file>
</folder>
```

#### Markdown Format
````markdown
```src/main.rs
// File contents here
```

```src/lib/mod.rs
// File contents here
```
````

#### JSON Format
```json
[
  {"type":"file","path":"src/main.rs","binary":false,"content":"// File contents here"},
  {"type":"directory","path":"src/lib","contents":[
    {"type":"file","path":"src/lib/mod.rs","binary":false,"content":"// File contents here"}
  ]}
]
```

#### LLM Format
```markdown
# PROJECT ANALYSIS FOR AI ASSISTANT

## 📦 GENERAL INFORMATION
- **Project path**: `/home/user/project`
- **Total files**: 12
- **Files included in this analysis**: 8
- **Main languages used**:
  - Rust (5 files)
  - Markdown (2 files)
  - TOML (1 file)

## 🗂️ PROJECT STRUCTURE
```
/home/user/project
├── src
│   ├── main.rs
│   ├── lib.rs
│   └── utils
│       └── helpers.rs
├── tests
│   └── integration_test.rs
└── Cargo.toml
```

## 🔄 FILE RELATIONSHIPS
### Core Files (most referenced)
- **`src/lib.rs`** is imported by 3 files

### Dependencies by File
- **`src/main.rs`**:
  - *Internal dependencies*: `src/lib.rs`, `src/utils/helpers.rs`
  - *External dependencies*: `std::io`, `clap`

*The LLM format provides a specialized Markdown output designed for AI assistants, including:*
- Project overview (path, counts, languages)
- File tree structure visualization
- **Per-file dependency mapping** (internal and external dependencies, detected via import/include patterns for various languages)
- File contents with syntax highlighting hints and dependency summaries

## 📄 FILE CONTENTS
### src/main.rs
```rust
fn main() {
    println!("Hello, world!");
}
```

## Features 🚀

- 🖥️ **Dual-Mode Operation**:
  - Interactive TUI for visual file exploration
  - Full-featured CLI for scripting and automation
- 📁 Interactive file browser with folder expansion/collapse
- 🔍 Enhanced search functionality:
  - Real-time filtering
  - Glob pattern support
  - Recursive or non-recursive search modes
  - **Supports both substring and glob pattern search in TUI**
- ✨ Rich file icons for over 200 file types
- 📋 Export in multiple formats:
  - XML (hierarchical and structured)
  - Markdown (human-readable format)
  - JSON (parseable format)
  - LLM (AI-optimized with dependency analysis)
- 📊 Smart code analysis (LLM format):
  - **Advanced dependency detection** with multi-language support
  - **Accurate file counting** and statistics
  - Project structure visualization
  - **Internal vs external dependency mapping**
  - Focused project insights for AI
- 🎯 Smart file filtering:
  - `.gitignore` support
  - Default ignore patterns (node_modules, target, etc.)
  - Custom ignore patterns
  - Binary file handling
- ⌨️ Intuitive keyboard shortcuts
- 🖥️ Cross-platform clipboard support:
  - Windows (PowerShell)
  - macOS (pbcopy/pbpaste)
  - Linux - X11 (xclip)
  - Linux - Wayland (wl-copy/wl-paste)
  - WSL2 integration
- 📝 Line number support for all formats
- 🔄 Format switching on the fly
- ⚡ Performance optimizations:
  - Asynchronous directory scanning (using `tokio` for async file operations)
  - Iterative vs. recursive loading options
  - Early bailout for large selections
- 🎨 Beautiful TUI with modal dialogs and help screens
- 📊 Detailed copy statistics with file counts and sizes
- 🔒 Selection limits to prevent memory issues (configurable)
- ⚙️ Persistent configuration:
  - Save and load settings
  - Per-user configuration files
  - CLI and TUI configuration separation
- 🪢 **Symlink Loop Detection:** Prevents infinite loops during directory traversal
- 📝 **Logging:** Events are logged to a timestamped file per run for debugging

## Installation 📦

Using Cargo:
```bash
cargo install aibundle
```

From source:
```bash
git clone https://github.com/yourusername/aibundle
cd aibundle
cargo build --release
```

## Usage 🛠️

### TUI Mode

Launch AIBundle's interactive terminal interface in any directory:
```bash
aibundle                           # Current directory
aibundle /path/to/project          # Specific directory
```

### CLI Mode

Use AIBundle directly from the command line:
```bash
# Basic usage - copy all .rs files to clipboard in LLM format
aibundle --files "*.rs"

# Export to a file instead of clipboard
aibundle --files "*.{rs,toml}" --output-file project_bundle.md

# Print to console with line numbers
aibundle --files "src/*.rs" --output-console --line-numbers

# Specify format and other options
aibundle --files "*.py" --format markdown --recursive

# Save your preferences for future use
aibundle --format llm --gitignore true --recursive true --save-config
```

### Command Line Options

#### Basic Syntax
```bash
aibundle [OPTIONS] [SOURCE_DIR]
```

#### Core Features
- File pattern matching and selection
- Multiple output formats (XML, Markdown, JSON, LLM)
- Content search capabilities
- Gitignore integration
- Clipboard, file, or console output
- Configurable ignore patterns
- Configuration saving

#### Essential Options
| Option        | Description                     | Example                  |
|---------------|---------------------------------|--------------------------|
| `--files`, `-f` | File pattern to match           | `--files "*.rs"`         |
| `--format`, `-m` | Output format (xml/markdown/json/llm)| `--format markdown`     |
| `--output-file`, `-o` | Write to file instead of clipboard| `--output-file bundle.md`      |
| `--output-console`, `-p` | Write to console instead of clipboard| `--output-console`      |
| `--source-dir`, `-d` | Source directory (default: current)| `--source-dir ./src`  |
| `--search`, `-s` | Search pattern to match file contents | `--search "function"` |
| `--recursive`, `-r` | Include subfolders recursively | `--recursive` |
| `--line-numbers`, `-n` | Show line numbers in output | `--line-numbers` |
| `--gitignore`, `-g` | Use .gitignore files for filtering (default: true) | `--gitignore` |
| `--ignore`, `-i` | Ignore patterns (comma-separated list) | `--ignore node_modules,target` |
| `--save-config`, `-S` | Save current settings to .aibundle.config | `--save-config` |

#### File Selection Patterns
| Pattern | Description | Example |
|---------|-------------|---------|
| `*`     | Match any characters | `*.rs` |
| `?`     | Match single character | `test?.rs` |
| `**`    | Match directories recursively | `src/**/*.rs` |
| `{}`    | Group patterns | `*.{rs,toml}` |

### TUI Options

#### Selection Methods
- Space: Toggle selection of current item
- * (asterisk): Toggle all visible items

#### Search & Filter Features
- Press `/` to enter search mode
- Case-insensitive by default
- Matches file names and paths

#### Output Options
- Press 'f' to cycle through formats: XML, Markdown, JSON, LLM
- Press 'n' to toggle line numbers

#### File Operations
- Press 'c' to copy selected files
- Press 'Tab' to expand/collapse folders
- Press 'Enter' to navigate into directories

#### Configuration Options
- Press 'i' to toggle default ignores
- Press 'g' to toggle gitignore usage
- Press 'b' to toggle binary files inclusion
- Press 's' to save current configuration

#### Help and Navigation
- Press 'h' to show help
- Press 'q' to quit (copies selected items to clipboard)

#### Visual Indicators
- ⏳ Operation in progress
- ✅ Operation complete
- ❌ Operation failed

### Configuration

AIBundle saves configuration in `~/.aibundle.config.toml`:
```toml
[cli]
files = "*.rs"
format = "llm"
recursive = true

[tui]
format = "markdown"
line_numbers = true
selection_limit = 600
```

### Keyboard Shortcuts

Navigation:
- `↑/↓` - Move selection
- `PgUp/PgDn` - Move by 10 items
- `Enter` - Open directory
- `Tab` - Expand/collapse folder

Selection:
- `Space` - Select/deselect item
- `*` - Select/deselect all visible items

Actions:
- `c` - Copy selected items to clipboard
- `f` - Toggle format (XML → Markdown → JSON → LLM)
- `n` - Toggle line numbers
- `/` - Search (ESC to cancel)

Filters:
- `i` - Toggle default ignores
- `g` - Toggle .gitignore support
- `b` - Toggle binary files

Other:
- `h` - Show help
- `s` - Save configuration
- `q` - Quit (copies if items selected)

### Best Practices
1. Use XML for AI platforms that expect structured input
2. Use Markdown for documentation and readability
3. Use JSON for programmatic processing
4. Use LLM for optimized language model input
5. Enable line numbers for code review contexts
6. Save your preferred configuration with `--save-config`

## Dependencies 📚

Core dependencies:
- [crossterm](https://crates.io/crates/crossterm) - Terminal manipulation and events
- [ratatui](https://crates.io/crates/ratatui) - Terminal user interface framework
- [ignore](https://crates.io/crates/ignore) - .gitignore support and file filtering
- [clap](https://crates.io/crates/clap) - Command line argument parsing
- [serde](https://crates.io/crates/serde) - Serialization/deserialization framework
- [toml](https://crates.io/crates/toml) - TOML file parsing for configuration
- [glob](https://crates.io/crates/glob) - Glob pattern matching
- [regex](https://crates.io/crates/regex) - Regular expressions for dependency analysis
- [itertools](https://crates.io/crates/itertools) - Additional iterator adaptors

## Build from Source

```bash
# Clean build
cargo clean
cargo build --release

# Run all checks
cargo fmt --all -- --check
cargo fmt
cargo clippy --all-targets --all-features
cargo test --all-features
cargo doc --no-deps

# If publishing to crates.io
cargo publish --dry-run
```

## Performance 🚀

AIBundle v0.6.13 includes several performance optimizations:
- Asynchronous processing:
  - Non-blocking item counting during selection
  - Background processing of large operations
- Smart selection management:
  - Early bailout for large directory selections
  - Default selection limit of 400 files (configurable)
  - Hierarchical selection cascading
- Improved file navigation:
  - Iterative vs. recursive directory traversal options
  - Smart caching of directory contents
  - Efficient path normalization
- Search optimizations:
  - Pattern-based (glob) or text-based search options
  - Recursive or non-recursive search modes
  - Real-time filtering with pattern matching

## Contributing 🤝

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License 📄

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments 🙏

- The Rust community for amazing crates
- [ratatui](https://github.com/tui-rs-revival/ratatui) for the excellent TUI framework
- All contributors who have helped shape this project

---

Made with ❤️ by the AIBundle Team