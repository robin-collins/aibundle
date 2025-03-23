
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->
# Refactoring Plan for Rust Codebase

Overview

This refactoring plan focuses on improving the modularity, readability, and maintainability of the codebase without modifying any functionality. The approach divides the monolithic `main.rs` file into logical modules following Rust's module system conventions, while ensuring all existing functionality remains intact.

## Workflow:

- [Stage Workflow Instructions](WORKFLOW.md)

## Stages:

- [Stage 1: Create Basic Module Structure](stage_1.md)
- [Stage 2: Extract Constants and Types to Models Module](stage_2.md)
- [Stage 3: Extract CLI Module](stage_3.md)
- [Stage 4: Extract Config Module](stage_4.md)
- [Stage 5: Expand File System Module](stage_5.md)
- [Stage 6: Extract Clipboard Module](stage_6.md)
- [Stage 7: Extract Utility Functions](stage_7.md)
- [Stage 8: Extract Output Formatters](stage_8.md)
- [Stage 9: Extract and Modularize TUI Components](stage_9.md)
- [Stage 10: Refactor Main Entry Point](stage_10.md)


## Final Structure

With the enhanced TUI modularization, the final structure now looks like this:

```
src/
├── cli/
│   ├── mod.rs         - CLI module exports
│   └── options.rs     - CLI option parsing and CLI mode
├── tui/
│   ├── mod.rs         - TUI module exports
│   ├── app.rs         - Core App coordinator (slim orchestration layer)
│   ├── components/    - UI visual components
│   │   ├── mod.rs
│   │   ├── modal.rs   - Modal dialog component
│   │   ├── file_list.rs - File list display component 
│   │   └── status_bar.rs - Status bar component
│   ├── state/         - Application state management
│   │   ├── mod.rs
│   │   ├── app_state.rs - Main application state
│   │   ├── selection.rs - Selection state management
│   │   └── search.rs    - Search state management
│   ├── handlers/      - Event and action handlers
│   │   ├── mod.rs
│   │   ├── keyboard.rs  - Keyboard event handlers
│   │   ├── clipboard.rs - Clipboard operation handlers
│   │   └── file_ops.rs  - File operation handlers
│   └── views/         - Screen compositions
│       ├── mod.rs
│       ├── main_view.rs  - Main application view
│       ├── help_view.rs  - Help screen view
│       └── message_view.rs - Message/notification view
├── config/
│   └── mod.rs         - Configuration handling
├── fs/
│   └── mod.rs         - File and directory operations
├── output/
│   ├── mod.rs         - Output module exports
│   ├── format.rs      - Common formatting functions
│   ├── xml.rs         - XML output format
│   ├── markdown.rs    - Markdown output format
│   ├── json.rs        - JSON output format
│   └── llm.rs         - LLM output format
├── clipboard/
│   └── mod.rs         - Clipboard operations
├── models/
│   ├── mod.rs         - Model module exports
│   ├── app_config.rs  - Configuration structures
│   ├── enums.rs       - Enum definitions
│   └── constants.rs   - Constants
├── utils/
│   └── mod.rs         - Utility functions
└── main.rs            - Application entry point
```

