# File Operations 🛠️

## Clipboard Operations 📋

### Copy to Clipboard
1. Select files using Space or *
2. Press 'c' to copy
3. Status modal shows:
   - Files copied
   - Total lines
   - Size
   - Format used

[Image: Screenshot of copy operation modal]
*Copy operation feedback showing statistics*

### Copy Formats
- 'f' to cycle through formats:
  - XML (default)
  - Markdown
  - JSON

## File Navigation 📂

### Directory Operations
```
Enter    → Open directory
Tab      → Expand/collapse
Backspace→ Parent directory
```

### Search Operations
```
/        → Start search
ESC      → Clear search
↑/↓      → Navigate results
```

## Visual Indicators ✨

### Operation Status
- ⏳ Operation in progress
- ✅ Operation complete
- ❌ Operation failed
- 📋 Copied to clipboard

### File Type Indicators
```
📁 Directory
📄 Regular file
🦀 Rust source
⚙️ Configuration
📝 Documentation
🔗 Symbolic link
```

## Configuration Options ⚙️

Toggle settings with:
- 'i': Default ignores
- 'g': .gitignore support
- 'b': Binary files
- 'n': Line numbers

## Operation Feedback 📊

### Copy Statistics
- Number of files
- Number of folders
- Total lines
- Total size
- Format used

### Modal Information
```
Files copied: 5
Folders copied: 2
Total lines: 150
Total size: 4.2 KB
Format: XML
```

## Keyboard Reference 🎹

| Key | Operation | Description |
|-----|-----------|-------------|
| c | Copy | Copy selection to clipboard |
| f | Format | Toggle output format |
| n | Numbers | Toggle line numbers |
| i | Ignore | Toggle default ignores |
| g | Git | Toggle .gitignore support |
| b | Binary | Toggle binary file handling |
| q | Quit | Exit (copies if selection exists) |

## Best Practices 💡

1. Check format before copying
2. Use line numbers for reference
3. Configure ignores before selection
4. Watch feedback modals
5. Use keyboard shortcuts for efficiency
