## AIBundle TUI Keyboard Shortcuts

### 🧭 **Navigation**
| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Page Up` | Move selection up by 10 items |
| `Page Down` | Move selection down by 10 items |
| `Home` | Jump to first item |
| `End` | Jump to last item |
| `Enter` | Open directory / Enter folder |
| `Backspace` | Go to parent directory (when ".." is selected) |

### 🎯 **Selection**
| Key | Action |
|-----|--------|
| `Space` | Select/deselect current item |
| `a` | Select/deselect all items |

### 📁 **Folder Management**
| Key | Action |
|-----|--------|
| `Tab` | Expand/collapse current folder (single level) |
| `Shift+Tab` | Expand/collapse current folder recursively (all subfolders) |

### ⚡ **Actions**
| Key | Action |
|-----|--------|
| `c` | Copy selected items to clipboard |
| `q` | Copy selected items to clipboard and quit |
| `Ctrl+C` | Copy selected items to clipboard and quit immediately |

### 🔧 **Format & Output**
| Key | Action |
|-----|--------|
| `f` | Toggle output format (XML → Markdown → JSON → LLM → XML) |
| `n` | Toggle line numbers in output (not available in JSON mode) |

### 🔍 **Search & Filtering**
| Key | Action |
|-----|--------|
| `/` | Enter/exit search mode |
| `Esc` (in search) | Cancel search and clear search query |
| `Enter` (in search) | Apply search and exit search mode |
| `Backspace` (in search) | Delete last character in search query |
| Any character (in search) | Add character to search query |

### 🚫 **Ignore Rules**
| Key | Action |
|-----|--------|
| `d` | Toggle default ignore patterns (node_modules, .git, etc.) |
| `g` | Toggle .gitignore rules |
| `b` | Toggle inclusion of binary files |
| `r` | Toggle recursive directory traversal mode |

### 🛠️ **Configuration & Help**
| Key | Action |
|-----|--------|
| `h` / `?` / `F1` | Show/hide help modal |
| `S` (capital S) | Save current configuration to disk |

### 📖 **Modal Navigation** (when help or other modals are open)
| Key | Action |
|-----|--------|
| `Esc` / `q` | Close modal |
| `Page Up` / `↑` / `k` | Scroll modal content up |
| `Page Down` / `↓` / `j` | Scroll modal content down |

### 🔄 **Special Context: Counting/Loading**
| Key | Action |
|-----|--------|
| `Esc` (during counting) | Cancel selection count operation |

### 📝 **Notes:**
- When in **search mode**, most navigation and action keys are disabled except for search-specific keys
- The `Enter` key behaves differently for files vs. directories (directories are opened, files currently do nothing)
- Some keys like `n` (line numbers) are context-sensitive (disabled in JSON format)
- Capital `S` is used for save to distinguish it from search functionality
- The TUI supports both arrow keys and vi-style navigation (`j`/`k`)
- Modal dialogs capture most input until closed with `Esc` or `q`

This comprehensive key mapping shows that AIBundle has a rich set of keyboard shortcuts covering navigation, selection, file operations, format control, search, filtering, and configuration management.
