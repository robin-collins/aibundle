# AIBundle 📦

[![Version](https://img.shields.io/badge/version-0.6.13-blue.svg)](https://crates.io/crates/aibundle)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A powerful CLI and TUI tool for bundling files and directories into AI/LLM-friendly formats. Perfect for sharing code snippets and project structures with AI assistants.

![AIBundle Screenshot](screenshot.png)

## AIBundle Format Specification v1 📋

AIBundle is a multi-format specification designed for optimal code sharing with Large Language Models:

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
- **Async Operations**: Non-blocking UI during large operations
- **Command Line Mode**: Full CLI support with config persistence
- **Smart Dependency Analysis**: Detects and visualizes file relationships (LLM format)

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

## 📄 FILE CONTENTS
### src/main.rs
```rust
fn main() {
    println!("Hello, world!");
}
```
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
- ✨ Rich file icons for over 200 file types
- 📋 Export in multiple formats:
  - XML (hierarchical and structured)
  - Markdown (human-readable format)
  - JSON (parseable format)
  - LLM (AI-optimized with dependency analysis)
- 📊 Smart code analysis (LLM format):
  - Dependency detection between files
  - Project structure visualization
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
  - Asynchronous directory scanning
  - Iterative vs. recursive loading options
  - Early bailout for large selections
- 🎨 Beautiful TUI with modal dialogs and help screens
- 📊 Detailed copy statistics with file counts and sizes
- 🔒 Selection limits to prevent memory issues (configurable)
- ⚙️ Persistent configuration:
  - Save and load settings
  - Per-user configuration files
  - CLI and TUI configuration separation

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