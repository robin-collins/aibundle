# TUI Navigation Guide 🧭

## Basic Navigation 🚀

### Movement Keys
- ⬆️ Up Arrow: Move selection up
- ⬇️ Down Arrow: Move selection down
- PgUp: Move up 10 items
- PgDn: Move down 10 items

### Directory Operations
- Enter: Open selected directory
- Backspace/Left Arrow: Go to parent directory
- Tab: Expand/collapse directory

[Image: Animated GIF showing navigation between directories]
*Navigation demonstration showing key movements and directory operations*

## File Selection 📋

### Selection Keys
- Space: Toggle selection of current item
- * (asterisk): Toggle selection of all visible items

### Selection Indicators
- [ ] Unselected item
- [X] Selected item
- [>] Current directory
- [..] Parent directory

## Search Navigation 🔍

1. Press `/` to enter search mode
2. Type your search query
3. Results filter in real-time
4. Use Up/Down to navigate results
5. Press ESC to clear search
6. Press Enter to select current result

## Visual Cues 👀

### File Type Icons
- 📂 Directory
- 📄 Regular file
- 🔗 Symbolic link
- 🦀 Rust source file
- ⚙️ Configuration file
- 📝 Documentation

### Status Indicators
- 📌 Current selection
- 🔍 Search active
- ⚡ Operation in progress
- ✅ Operation complete

## Keyboard Reference 🎹

| Key | Action | Context |
|-----|--------|---------|
| ↑/↓ | Move selection | Always |
| Enter | Open directory | On directory |
| Space | Toggle selection | Always |
| Tab | Expand directory | On directory |
| / | Start search | Always |
| ESC | Clear search/Close modal | Search/Modal |
| q | Quit application | Always |

## Tips & Tricks 💡

1. Use Tab to preview directory contents
2. Combine search with selection
3. Use PgUp/PgDn for faster navigation
4. Watch status bar for current context