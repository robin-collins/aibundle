# AIBundle Development Guide

## Build Commands
```bash
# Build with cargo
cargo build --release
# Install locally
cargo install --path .
# Run tests
./run_tests.sh
# Code quality
cargo fmt
cargo clippy --all-targets --all-features
```

## Code Style
- **Naming**: snake_case for functions/variables, PascalCase for types
- **Formatting**: Use `cargo fmt` before committing
- **Error handling**: Return `io::Result<T>` and use descriptive messages
- **Functions**: Keep function length reasonable (<50 lines when possible)
- **Comments**: Add comments for complex logic, but prefer self-documenting code
- **Imports**: Group imports by crate, with std imports first

## Project Organization
- Core app logic in `src/main.rs`
- File system operations in `src/fs/mod.rs`
- TUI/CLI modes are both supported
- Default format is XML, also supports Markdown and JSON