# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2025-01-21

### Fixed
- **LLM Output Format**: Fixed critical issues with file counting and dependency analysis
  - Resolved duplicate statistics calculation causing incorrect file/folder counts
  - Enhanced dependency analysis with improved regex patterns for multiple languages
  - Added comprehensive file mapping for better internal dependency resolution
  - Fixed tree building consistency across different code paths
  - Improved import detection with line-by-line parsing for better accuracy

### Enhanced
- **Dependency Analysis**: Significantly improved language support and accuracy
  - Added support for Python, Rust, JavaScript/TypeScript, C/C++, Java, Go, Ruby, PHP, Swift, shell scripts
  - Enhanced internal vs external dependency categorization
  - Added proper file path variations for better import resolution
  - Improved Rust module detection with `::` and `/` path handling
  - Added support for common import patterns like `mod.rs`, `index.js`, etc.

### Improved
- **Code Quality**: Cleaned up LLM output module architecture
  - Centralized tree building and statistics calculation
  - Removed duplicate code between clipboard handler and LLM formatter
  - Better separation of concerns in output formatting
  - Fixed compiler warnings and unused code

### Technical Details
- **File Counting**: Now accurately tracks selected files vs total files in analysis
- **Tree Structure**: Consistent tree building across all output formats
- **Performance**: More efficient dependency analysis with better regex compilation
- **Output Quality**: Enhanced readability with emojis and sorted dependency listings

## [0.6.13] - Previous Release
- Initial stable release with TUI and CLI modes
- Multiple output formats (XML, Markdown, JSON, LLM)
- Basic dependency analysis
- Cross-platform clipboard support
- File filtering and ignore patterns