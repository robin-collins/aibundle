# AIFormat 📋

[![Version](https://img.shields.io/badge/version-0.4.0-blue.svg)](https://crates.io/crates/aiformat)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A TUI (Terminal User Interface) file browser and formatter designed to help developers quickly browse, select, and copy files in a structured format. Perfect for sharing code snippets and directory structures with AI assistants.

![AIFormat Screenshot](screenshot.png)

## Features 🚀

- 📁 Interactive file browser with folder expansion/collapse
- 🔍 Live search functionality
- ✨ File icons for different file types
- 📋 Copy files and directories in multiple formats:
  - XML format with structured tags
  - Markdown format with code blocks
  - JSON format (minimized, perfect for parsing)
- 🎯 Smart file filtering:
  - `.gitignore` support
  - Default ignore patterns (node_modules, target, etc.)
  - Binary file handling
- ⌨️ Intuitive keyboard shortcuts
- 🖥️ Cross-platform clipboard support (Windows, Linux X11/Wayland, WSL)
- 📝 Line number support for all formats
- 🔄 Format switching on the fly

## Installation 📦

Using Cargo:
```bash
cargo install aiformat
```

From source:
```bash
git clone https://github.com/yourusername/aiformat
cd aiformat
cargo build --release
```

## Usage 🛠️

Launch AIFormat in any directory:
```bash
aiformat
```

### Keyboard Shortcuts

- `Space` - Select/deselect item
- `*` - Select/deselect all visible items
- `Enter` - Open directory
- `Tab` - Expand/collapse folder
- `/` - Toggle search mode (ESC to cancel)
- `c` - Copy selected items to clipboard
- `i` - Toggle default ignores
- `g` - Toggle .gitignore support
- `b` - Toggle binary file inclusion
- `f` - Toggle output format (XML → Markdown → JSON)
- `n` - Toggle line numbers
- `q` - Quit (copies selection if items are selected)

### Navigation
- `↑` `↓` - Move selection
- `PageUp` `PageDown` - Move selection by 10 items
- `Enter` - Enter directory
- `..` - Go to parent directory

## Output Formats 📝

### XML Format
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

### Markdown Format
````markdown
```src/main.rs
// File contents here
```

```src/lib/mod.rs
// File contents here
```
````

### JSON Format
```json
[
  {"type":"file","path":"src/main.rs","binary":false,"content":"// File contents here"},
  {"type":"directory","path":"src/lib","contents":[
    {"type":"file","path":"src/lib/mod.rs","binary":false,"content":"// File contents here"}
  ]}
]
```

## Dependencies 📚

- [crossterm](https://crates.io/crates/crossterm) - Terminal manipulation
- [ratatui](https://crates.io/crates/ratatui) - Terminal user interface
- [cli-clipboard](https://crates.io/crates/cli-clipboard) - Clipboard operations
- [ignore](https://crates.io/crates/ignore) - .gitignore support

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

Made with ❤️ by the AIFormat Team 