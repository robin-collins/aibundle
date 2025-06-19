# AIBundle Modular Development Guide

## Build Commands
```bash
# Build and run
cargo build
cargo run -- [args]

# Testing
cargo test                    # Run all tests
cargo test test_name          # Run single test
cargo test -- --nocapture     # Show test output

# Code quality
cargo fmt                     # Format code
cargo clippy                  # Lint code
```

## Code Style
- **Imports**: Group by source (std first, then external, then internal)
- **Naming**: snake_case for variables/functions, PascalCase for types/structs/enums
- **Error Handling**: Use Result with ? operator, descriptive messages
- **Types**: Leverage Rust's type system with structs/enums, Option/Result
- **Formatting**: 4-space indentation, run cargo fmt before committing
- **Documentation**: Use /// rustdoc comments for public interfaces
- **Modules**: Follow mod.rs pattern for module organization
- **Functions**: Single responsibility, clear parameter/return types

## Project Structure
- CLI interface in `/src/cli/`
- TUI components in `/src/tui/`
- Core business logic in dedicated modules
- Output formatters in `/src/output/`