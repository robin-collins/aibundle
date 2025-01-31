# Icons Reference Guide 🎨

## File Type Icons

AIBundle uses intuitive icons to represent different file types in the TUI interface.

### Basic File Types 📁

| Icon | Type | Description |
|------|------|-------------|
| 📄 | File | Regular file |
| 📁 | Directory | Folder/Directory |
| 🔗 | Symlink | Symbolic link |
| 📦 | Binary | Binary file |
| ❓ | Unknown | Unknown file type |

### Code Files 👨‍💻

| Icon | Type | Extensions |
|------|------|------------|
| 🦀 | Rust | `.rs` |
| ⚙️ | Config | `.toml`, `.yaml`, `.json` |
| 📝 | Doc | `.md`, `.txt` |
| 🔧 | Build | `Cargo.toml`, `Cargo.lock` |

### Special Files 🎯

| Icon | Type | Name Pattern |
|------|------|-------------|
| 📋 | License | `LICENSE*` |
| 📘 | Readme | `README*` |
| 🔒 | Lock | `*.lock` |
| ⚡ | Git | `.git*` |

## Status Icons 📊

### Selection Status
| Icon | Meaning |
|------|---------|
| [ ] | Unselected |
| [X] | Selected |
| [>] | Current item |
| [..] | Parent directory |

### Operation Status
| Icon | Meaning |
|------|---------|
| ⏳ | Processing |
| ✅ | Success |
| ❌ | Error |
| 📋 | Copied |

## Navigation Indicators 🧭

### Directory Structure
```
📁 project/
├── 📄 README.md
├── 🦀 main.rs
├── ⚙️ Cargo.toml
└── 📁 src/
    ├── 🦀 lib.rs
    └── 📁 modules/
```

### Path Components
| Icon | Component |
|------|-----------|
| 📁 | Directory |
| 📄 | File |
| 🔗 | Link |
| ↩️ | Back |
| ⬆️ | Parent |

## Format Indicators 🎨

### Output Formats
| Icon | Format |
|------|--------|
| 📑 | XML |
| 📝 | Markdown |
| 🔄 | JSON |

### Line Numbers
| Icon | Status |
|------|--------|
| 🔢 | Enabled |
| ⬜ | Disabled |

## Operation Modes 🔧

### Search Mode
| Icon | Mode |
|------|------|
| 🔍 | Search active |
| 🎯 | Match found |
| ❌ | No matches |

### Filter Mode
| Icon | Status |
|------|--------|
| 🔧 | Filter active |
| ✨ | Custom filter |
| 🚫 | Ignore active |

## Best Practices 💡

### Icon Usage
1. Visual Hierarchy
   - Primary: File types
   - Secondary: Status
   - Tertiary: Operations

2. Navigation
   - Use icons for quick identification
   - Combine with text for clarity
   - Consistent placement

3. Status Feedback
   - Clear operation status
   - Selection indication
   - Process feedback

### Custom Icons
```toml
# .aibundle.config
[icons]
rust = "🦀"
config = "⚙️"
doc = "📝"
binary = "📦"
```

## Icon Combinations 🎯

### Common Patterns
| Pattern | Meaning |
|---------|---------|
| 📁 [X] | Selected directory |
| 🦀 [>] | Current Rust file |
| 📝 ⚡ | Modified doc file |
| 🔗 ❌ | Broken symlink |

### Status Sequences
| Sequence | Meaning |
|----------|---------|
| ⏳ → ✅ | Operation complete |
| 🔍 → 🎯 | Search success |
| 📋 → ✅ | Copy successful |

## Accessibility Notes ♿

### Text Alternatives
- All icons have text equivalents
- Status shown in status bar
- Keyboard indicators available

### Configuration
```toml
# .aibundle.config
[accessibility]
show_text_labels = true    # Show text with icons
use_ascii = false         # Use ASCII alternatives
high_contrast = false     # High contrast mode
```
