# AIBundle 📦

[![Version](https://img.shields.io/badge/version-0.5.0-blue.svg)](https://crates.io/crates/aibundle)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A TUI (Terminal User Interface) tool for bundling files and directories into AI/LLM-friendly formats. Perfect for sharing code snippets and project structures with AI assistants.

![AIBundle Screenshot](screenshot.png)

## AIBundle Format Specification v1 📋

AIBundle is a multi-format specification designed for optimal code sharing with Large Language Models:

### Core Features
- **Multiple Representations**: Each bundle can be exported as:
  - XML (hierarchical and structured)
  - Markdown (human-readable and LLM-friendly)
  - JSON (parseable and programmatic)
- **File Hierarchy**: Preserves directory structure and relationships
- **Content Preservation**: Maintains original file content and formatting
- **Binary Handling**: Identifies and marks binary files
- **Path Normalization**: Uses forward slashes for cross-platform compatibility
- **Line Numbers**: Optional line numbering for precise code references
- **Selection Limits**: Smart handling of large directories with selection limits
- **Async Operations**: Non-blocking UI during large operations

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

## Features 🚀

- 📁 Interactive file browser with folder expansion/collapse
- 🔍 Live search functionality with real-time filtering
- ✨ Rich file icons for over 200 file types
- 📋 Export in multiple formats (XML/MD/JSON)
- 🎯 Smart file filtering:
  - `.gitignore` support
  - Default ignore patterns (node_modules, target, etc.)
  - Binary file handling
- ⌨️ Intuitive keyboard shortcuts
- 🖥️ Cross-platform clipboard support (Windows, Linux X11/Wayland, WSL)
- 📝 Line number support for all formats
- 🔄 Format switching on the fly
- ⚡ Performance optimizations for large directories
- 🎨 Beautiful TUI with modal dialogs and help screens
- 📊 Detailed copy statistics with file counts and sizes
- 🔒 Selection limits to prevent memory issues

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

Launch AIBundle in any directory:
```bash
aibundle
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
- `f` - Toggle format (XML → Markdown → JSON)
- `n` - Toggle line numbers
- `/` - Search (ESC to cancel)

Filters:
- `i` - Toggle default ignores
- `g` - Toggle .gitignore support
- `b` - Toggle binary files

Other:
- `h` - Show help
- `q` - Quit (copies if items selected)

## Dependencies 📚

Core dependencies:
- [crossterm](https://crates.io/crates/crossterm) - Terminal manipulation and events
- [ratatui](https://crates.io/crates/ratatui) - Terminal user interface framework
- [cli-clipboard](https://crates.io/crates/cli-clipboard) - Cross-platform clipboard operations
- [ignore](https://crates.io/crates/ignore) - .gitignore support and file filtering

## B# Clean build

```bash
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

AIBundle v0.5.0 includes several performance optimizations:
- Early bailout for large directory selections
- Asynchronous item counting
- Smart caching of directory contents
- Efficient path normalization
- Optimized search filtering

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