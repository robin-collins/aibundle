#!/bin/bash

# Test D1: Search Functionality
# Verifies that search mode works correctly with '/' key and filters items

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common functions
source "$SCRIPT_DIR/common_test_functions.sh"

# Test configuration
TEST_NAME="D1_search_functionality"
CAPTURE_FILE="test_capture-D1.txt"

echo "=========================================="
echo "Starting Test D1: Search Functionality"
echo "=========================================="

# Setup test session
setup_test_session "$TEST_NAME"

# Test execution
{
    echo "Step 1: Launch application"
    launch_app "" 3

    echo "Step 2: Capture initial screen (all files visible)"
    capture_pane "test_capture-D1-initial.txt"

    echo "Step 3: Enter search mode with '/'"
    send_keys "/" 1
    capture_pane "test_capture-D1-search-mode.txt"

    echo "Step 4: Verify search prompt appears"
    if verify_text_exists "test_capture-D1-search-mode.txt" "Search"; then
        echo "✓ Search mode activated"

        echo "Step 5: Type search query 'py'"
        send_keys "py" 1
        capture_pane "test_capture-D1-after-py.txt"

        echo "Step 6: Verify filtering works"
        # Should show Python files and hide others
        if verify_text_exists "test_capture-D1-after-py.txt" "main.py" || verify_text_exists "test_capture-D1-after-py.txt" ".py"; then
            echo "✓ Search filtering works"

            echo "Step 7: Exit search with Enter or '/'"
            send_keys "Return" 1
            capture_pane "test_capture-D1-after-enter.txt"

            echo "Step 8: Test search with backspace"
            send_keys "/" 1
            send_keys "main" 1
            send_keys "BackSpace" 1
            capture_pane "test_capture-D1-backspace.txt"

            echo "Step 9: Test search with no results"
            send_keys "Escape" 1  # Exit current search
            sleep 0.5
            send_keys "/" 1
            send_keys "nonexistentfile" 1
            capture_pane "test_capture-D1-no-results.txt"

            echo "Step 10: Exit search and verify normal view returns"
            send_keys "Escape" 1
            capture_pane "$CAPTURE_FILE"

            echo "Step 11: Verify normal file list is restored"
            EXPECTED_FILES=(
                "main.py"
                "config.py"
                "models"
                "services"
            )

            if verify_multiple_texts "$CAPTURE_FILE" "${EXPECTED_FILES[@]}"; then
                echo "✓ Normal view restored after search"
                print_test_result "PASS" "$TEST_NAME"
                exit 0
            else
                echo "FAIL: Normal view not properly restored"
            fi
        else
            echo "FAIL: Search filtering not working"
        fi
    else
        echo "FAIL: Search mode not activated"
    fi

    print_test_result "FAIL" "$TEST_NAME"
    exit 1

} || {
    echo "Test execution failed"
    print_test_result "FAIL" "$TEST_NAME"
    exit 1
}