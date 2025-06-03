#!/bin/bash

# Test A1: Default Startup (current directory)
# Verifies that the app launches with no arguments and displays proper UI elements

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common functions
source "$SCRIPT_DIR/common_test_functions.sh"

# Test configuration
TEST_NAME="A1_default_startup"
CAPTURE_FILE="test_capture-A1.txt"

echo "=========================================="
echo "Starting Test A1: Default Startup"
echo "=========================================="

# Setup test session
setup_test_session "$TEST_NAME"

# Test execution
{
    echo "Step 1: Launch application with no arguments"
    launch_app "" 3

    echo "Step 2: Capture initial screen"
    capture_pane "$CAPTURE_FILE"

    echo "Step 3: Verify expected UI elements"
    EXPECTED_TEXTS=(
        "AIBundle"
        "Files"
        "items"
        "selected"
        "Space: select"
    )

        # Verify all expected texts are present
    if verify_multiple_texts "$CAPTURE_FILE" "${EXPECTED_TEXTS[@]}"; then
        echo "Step 4: Verify we're in the correct directory by checking test files"
        # The current directory is verified by seeing our test files, not by a directory name in UI
        if verify_text_exists "$CAPTURE_FILE" "\[ \].*main\.py" && verify_text_exists "$CAPTURE_FILE" "\[ \].*config\.py"; then
            echo "Step 5: Verify file list shows expected test files with selection checkboxes"
            # Should show our test directory structure
            if verify_text_exists "$CAPTURE_FILE" "📁 models/" && verify_text_exists "$CAPTURE_FILE" "📁 services/" && verify_text_exists "$CAPTURE_FILE" "📁 utils/"; then
                echo "Step 6: Verify status bar shows toggle states and format"
                # Should show toggle states like [x] for enabled, [ ] for disabled, and format [LLM]
                if verify_text_exists "$CAPTURE_FILE" "\[LLM\]" && verify_text_exists "$CAPTURE_FILE" "ignores \[x\]"; then
                    echo "Step 7: Verify file icons are displayed"
                    # Should show folder and Python file icons
                    if verify_text_exists "$CAPTURE_FILE" "📁" && verify_text_exists "$CAPTURE_FILE" "🐍"; then
                        print_test_result "PASS" "$TEST_NAME"
                        exit 0
                    else
                        echo "FAIL: File icons not displayed"
                    fi
                else
                    echo "FAIL: Status bar toggle states or format not found"
                fi
                            else
                    echo "FAIL: Expected test directory structure not visible (missing models/, services/, utils/ folders)"
                fi
        else
            echo "FAIL: Not in expected test directory (missing main.py and config.py files)"
        fi
    else
        echo "FAIL: Missing expected UI elements"
    fi

    print_test_result "FAIL" "$TEST_NAME"
    exit 1

} || {
    echo "Test execution failed"
    print_test_result "FAIL" "$TEST_NAME"
    exit 1
}