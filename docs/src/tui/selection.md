# File Selection in TUI 📋

## Selection Methods 🎯

### Single Selection
- Space: Toggle selection of current item
- Selection limit: 400 items
- Visual feedback with [X] indicator

### Bulk Selection
- * (asterisk): Toggle all visible items
- Automatic limit checking
- Progress indicator for large directories

[Image: Screenshot showing selection states]
*Different selection states and indicators in the TUI*

## Selection States 🔄

| Indicator | Meaning |
|-----------|---------|
| [ ] | Unselected item |
| [X] | Selected item |
| [>] | Current directory |
| [..] | Parent directory |

## Directory Selection 📂

When selecting a directory:
1. Directory itself is selected
2. All contained files are selected
3. Selection respects ignore patterns
4. Counts toward 400 item limit

```
[X] 📁 src/
    [X] 📄 main.rs
    [X] 📄 lib.rs
    [X] 📁 modules/
        [X] 📄 mod.rs
```

## Selection Limits ⚠️

- Maximum 400 items total
- Warning modal appears when limit reached
- Automatic limit checking for directories
- Clear indication of current count

## Selection Feedback 📊

### Status Bar
- Total items selected
- Current selection size
- Format indication
- Operation status

### Modal Dialogs
- Selection limit warnings
- Operation confirmations
- Copy success/failure
- Error messages

## Best Practices 💡

1. Use search to narrow selection scope
2. Expand directories before bulk selection
3. Watch status bar for selection count
4. Use Space for precise selection
5. Use * for bulk operations
