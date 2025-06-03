# AIBundle TUI Testing Suite

This directory contains a comprehensive testing suite for the AIBundle TUI application using tmux for automated interaction testing.

## Overview

The testing suite uses tmux to simulate user interactions with the TUI application, capturing screen output and verifying expected behaviors. This allows for automated testing of the user interface, key bindings, navigation, selection mechanics, and other interactive features.

## Directory Structure

```
tests/
├── files/                    # Test data files (Python application structure)
│   ├── main.py              # Main entry point with imports
│   ├── config.py            # Configuration module
│   ├── models/              # Data models package
│   ├── services/            # Business logic package
│   ├── utils/               # Utilities package
│   ├── requirements.txt     # Dependencies
│   └── .gitignore           # Python gitignore
├── scripts/                 # Test scripts
│   ├── common_test_functions.sh  # Shared utilities
│   ├── run_all_tests.sh     # Main test runner
│   └── test_*.sh            # Individual test scripts
├── captured-panes/          # Output captures (created during test runs)
└── README.md               # This file
```

## Prerequisites

1. **tmux**: Required for running the tests
   ```bash
   # Ubuntu/Debian
   sudo apt-get install tmux

   # macOS
   brew install tmux
   ```

2. **AIBundle Binary**: Build the application before running tests
   ```bash
   cargo build --release
   # or for debug build
   cargo build
   ```

## Running Tests

### Run All Tests
```bash
# From the project root directory
./tests/scripts/run_all_tests.sh
```

### Run Individual Tests
```bash
# Example: Run just the startup test
./tests/scripts/test_A1_default_startup.sh
```

### Make Scripts Executable (if needed)
```bash
chmod +x tests/scripts/*.sh
```

## Test Categories

### A. Startup and Basic UI
- **A1**: Default startup (current directory)
- **A2**: Startup with specific directory argument

### B. Navigation and File Listing
- **B1**: List navigation (j/k, arrows, Page Up/Down, Home/End)
- **B2**: Directory traversal (Enter on folders, ../ navigation)

### C. Selection Mechanics
- **C1**: Single item selection/deselection (Space key)
- **C2**: Select all/deselect all ('a' key)

### D. Search Functionality
- **D1**: Search mode ('/' key, filtering, backspace, escape)

### E. Toggle Options & Filters
- **E1**: Toggle output format ('f' key cycling through LLM/XML/Markdown/JSON)

### G. Help and Modals
- **G1**: Help modal ('h' and '?' keys, navigation, closing)

### I. Quit Functionality
- **I1**: Quit ('q' key, handling selections, clean exit)

## Test Output

### Captured Panes
- Screen captures are saved to `tests/captured-panes/`
- Files are named `test_capture-[TEST_ID].txt`
- Additional intermediate captures may be saved with descriptive suffixes

### Test Results
- Summary results are saved to `tests/captured-panes/test_results_[TIMESTAMP].txt`
- Each test run creates a new results file
- Results include pass/fail status, output, and timing information

### Console Output
- Real-time test progress with colored output
- ✓ Green for passed tests
- ✗ Red for failed tests
- Summary statistics at the end

## Test Files Structure

The `tests/files/` directory contains a realistic Python application structure with:

- **10+ Python files** organized in packages
- **Proper imports** and dependencies between modules
- **Various file types** (.py, .txt, .gitignore, etc.)
- **Directory structure** for testing navigation
- **Real code content** for authentic file testing

This structure allows testing of:
- File listing and icon display
- Directory navigation
- Search and filtering
- Selection mechanics across different file types
- Real-world application scenarios

## Writing New Tests

### Test Script Template
```bash
#!/bin/bash

# Test XX: Description
# Brief description of what this test verifies

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common_test_functions.sh"

TEST_NAME="XX_test_name"
CAPTURE_FILE="test_capture-XX.txt"

echo "Starting Test XX: Description"

setup_test_session "$TEST_NAME"

{
    echo "Step 1: Launch application"
    launch_app "" 3

    echo "Step 2: Perform test actions"
    send_keys "some_key" 1
    capture_pane "$CAPTURE_FILE"

    echo "Step 3: Verify results"
    if verify_text_exists "$CAPTURE_FILE" "expected_text"; then
        print_test_result "PASS" "$TEST_NAME"
        exit 0
    fi

    print_test_result "FAIL" "$TEST_NAME"
    exit 1
} || {
    print_test_result "FAIL" "$TEST_NAME"
    exit 1
}
```

### Key Functions Available

- `setup_test_session(test_name)` - Initialize tmux session
- `launch_app(args, delay)` - Start the application
- `send_keys(keys, delay)` - Send key presses
- `capture_pane(filename)` - Save screen content
- `verify_text_exists(file, text)` - Check for expected text
- `verify_multiple_texts(file, texts...)` - Check multiple patterns
- `cleanup_session()` - Clean up tmux session
- `print_test_result(result, name)` - Display test outcome

### Adding New Tests

1. Create new test script in `tests/scripts/`
2. Follow the naming convention: `test_[CATEGORY][NUMBER]_[NAME].sh`
3. Make script executable: `chmod +x tests/scripts/test_*.sh`
4. Add to the test list in `run_all_tests.sh`

## Troubleshooting

### Common Issues

1. **tmux not found**: Install tmux package
2. **Binary not found**: Build the application with `cargo build --release`
3. **Permission denied**: Make scripts executable with `chmod +x`
4. **Test timeouts**: Increase delays in test scripts if needed
5. **Session conflicts**: Kill stray sessions with `tmux kill-server`

### Debugging Tests

1. **Manual tmux session**: Create a session and run the app manually
   ```bash
   tmux new-session -s debug -c tests/files
   ../../target/release/aibundle
   ```

2. **Check captured output**: Examine files in `tests/captured-panes/`

3. **Verbose mode**: Add debug output to test scripts
   ```bash
   echo "Debug: Current state"
   capture_pane "debug-capture.txt"
   cat tests/captured-panes/debug-capture.txt
   ```

4. **Session inspection**: Use tmux commands to inspect running sessions
   ```bash
   tmux list-sessions
   tmux capture-pane -t session_name -p
   ```

## Contributing

When adding new features to the TUI:

1. **Add corresponding tests** for new functionality
2. **Update existing tests** if behavior changes
3. **Test edge cases** and error conditions
4. **Verify test coverage** for all user-facing features

The goal is to maintain comprehensive automated testing that catches regressions and ensures consistent user experience across different environments.