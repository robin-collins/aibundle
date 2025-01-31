# Search & Filter Features 🔍

## Real-time Search 🚀

### Activating Search
- Press `/` to enter search mode
- Current search shown in status bar
- Results filter in real-time
- ESC to clear search

[Image: Screenshot showing search in action]
*Real-time search interface with highlighted results*

## Search Behavior 🎯

### Pattern Matching
- Case-insensitive by default
- Matches file names and paths
- Partial matches supported
- Multiple terms with spaces

Example searches:
```
test      → Matches test files
.rs       → Matches Rust files
src/lib   → Matches lib files in src
```

## Filter Options 🔧

### Built-in Filters
- 'i': Toggle default ignores
  - node_modules
  - .git
  - target
  - dist
  - build
  - coverage

### Gitignore Integration
- 'g': Toggle .gitignore support
- Respects all .gitignore patterns
- Cascading rules support
- Repository-wide patterns

### Binary File Handling
- 'b': Toggle binary file visibility
- Default: binary files hidden
- Common binary extensions filtered
- Size-based detection

## Search Context 📊

### Status Indicators
- 🔍 Search active
- 📊 Match count
- 🎯 Current match
- ❌ No matches

### Navigation
```
↑/↓      Navigate matches
Enter    Select current match
ESC      Clear search
/        New search
```

## Best Practices 💡

1. Start with specific terms
2. Use path components for context
3. Combine with filters for precision
4. Watch status bar for feedback
5. Clear search when done
