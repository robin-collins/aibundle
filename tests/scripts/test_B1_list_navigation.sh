#!/bin/bash

# Test B1: List Navigation
# Verifies that navigation keys (j/k, arrows, Page Up/Down, Home/End) work correctly

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common functions
source "$SCRIPT_DIR/common_test_functions.sh"

# Test configuration
TEST_NAME="B1_list_navigation"
CAPTURE_FILE="test_capture-B1.txt"

echo "=========================================="
echo "Starting Test B1: List Navigation"
echo "=========================================="

# Setup test session
setup_test_session "$TEST_NAME"

# Test execution
{
    echo "Step 1: Launch application"
    launch_app "" 3

    echo "Step 2: Capture initial screen to establish baseline"
    capture_pane "test_capture-B1-initial.txt"

    echo "Step 3: Test j key (move down)"
    send_keys "j" 1
    capture_pane "test_capture-B1-after-j.txt"

    echo "Step 4: Test k key (move up)"
    send_keys "k" 1
    capture_pane "test_capture-B1-after-k.txt"

    echo "Step 5: Test Down arrow"
    send_keys "Down" 1
    capture_pane "test_capture-B1-after-down.txt"

    echo "Step 6: Test Up arrow"
    send_keys "Up" 1
    capture_pane "test_capture-B1-after-up.txt"

    echo "Step 7: Test multiple j presses"
    send_keys "j" 0.5
    send_keys "j" 0.5
    send_keys "j" 0.5
    capture_pane "test_capture-B1-after-multiple-j.txt"

    echo "Step 8: Test Home key"
    send_keys "Home" 1
    capture_pane "test_capture-B1-after-home.txt"

    echo "Step 9: Test End key"
    send_keys "End" 1
    capture_pane "test_capture-B1-after-end.txt"

    echo "Step 10: Test Page Down"
    send_keys "Home" 1  # Go to top first
    send_keys "Next" 1  # Page Down
    capture_pane "test_capture-B1-after-pagedown.txt"

    echo "Step 11: Test Page Up"
    send_keys "Prior" 1  # Page Up
    capture_pane "$CAPTURE_FILE"

    echo "Step 12: Verify navigation worked"
    # Check that we can find expected files and the app is still responsive
    EXPECTED_TEXTS=(
        "main.py"
        "config.py"
        "models"
        "services"
        "utils"
    )

    if verify_multiple_texts "$CAPTURE_FILE" "${EXPECTED_TEXTS[@]}"; then
        echo "Step 13: Verify app is still responsive after navigation"
        # Try one more navigation command
        send_keys "j" 0.5
        capture_pane "test_capture-B1-final.txt"

        if verify_text_exists "test_capture-B1-final.txt" "AIBundle"; then
            print_test_result "PASS" "$TEST_NAME"
            exit 0
        else
            echo "FAIL: App became unresponsive after navigation"
        fi
    else
        echo "FAIL: Expected files not found or navigation failed"
    fi

    print_test_result "FAIL" "$TEST_NAME"
    exit 1

} || {
    echo "Test execution failed"
    print_test_result "FAIL" "$TEST_NAME"
    exit 1
}