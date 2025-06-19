#!/bin/bash

# Test C2: Select All/Deselect All
# Verifies that 'a' key selects all items and pressing again deselects all

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common functions
source "$SCRIPT_DIR/common_test_functions.sh"

# Test configuration
TEST_NAME="C2_select_all"
CAPTURE_FILE="test_capture-C2.txt"

echo "=========================================="
echo "Starting Test C2: Select All/Deselect All"
echo "=========================================="

# Setup test session
setup_test_session "$TEST_NAME"

# Test execution
{
    echo "Step 1: Launch application"
    launch_app "" 3

    echo "Step 2: Capture initial screen (no selections)"
    capture_pane "test_capture-C2-initial.txt"

    echo "Step 3: Verify initial state has no selections"
    # Check status bar shows 0 selected and no files have checkboxes marked
    # First, check for 0 selected in the status bar
    if verify_text_exists "test_capture-C2-initial.txt" "(0 selected)"; then
        # Then, check the file list (excluding status bar) for any selected items
        # The status bar is typically in the last 2-3 lines of the screen
        # We use head -n -3 to exclude the last 3 lines, then grep for [x] checkboxes
        CONTENT_WITHOUT_STATUS=$(head -n -3 "$CAPTURE_DIR/test_capture-C2-initial.txt")
        if ! echo "$CONTENT_WITHOUT_STATUS" | grep -q "\[x\]"; then
            echo "✓ Initial state has no selections"

            echo "Step 4: Press 'a' to select all"
            send_keys "a" 2
            capture_pane "test_capture-C2-after-select-all.txt"

            echo "Step 5: Verify all items are selected"
            # Should see multiple [x] indicators and updated count in status bar (7 items in tests/files root)
            if verify_text_exists "test_capture-C2-after-select-all.txt" "\[x\]" && verify_text_exists "test_capture-C2-after-select-all.txt" "(16 selected)"; then
                echo "✓ Items are selected after 'a' key and count is correct"

                echo "Step 6: Press 'a' again to deselect all"
                send_keys "a" 2
                capture_pane "test_capture-C2-after-deselect-all.txt"

                echo "Step 7: Verify all items are deselected"
                CONTENT_WITHOUT_STATUS_DESELECT=$(head -n -3 "$CAPTURE_DIR/test_capture-C2-after-deselect-all.txt")
                if verify_text_exists "test_capture-C2-after-deselect-all.txt" "(0 selected)" && ! echo "$CONTENT_WITHOUT_STATUS_DESELECT" | grep -q "\[x\]" && echo "$CONTENT_WITHOUT_STATUS_DESELECT" | grep -q "\[ \]"; then
                    echo "✓ Items are deselected after second 'a' key and count is 0"

                    echo "Step 8: Test select all again to ensure it's repeatable"
                    send_keys "a" 2
                    capture_pane "test_capture-C2-repeat-select.txt"

                    if verify_text_exists "test_capture-C2-repeat-select.txt" "\[x\]" && verify_text_exists "test_capture-C2-repeat-select.txt" "(16 selected)"; then
                        echo "✓ Select all is repeatable and count is correct"

                        echo "Step 9: Final verification"
                        capture_pane "$CAPTURE_FILE"

                        # Verify the app is still functional
                        if verify_text_exists "$CAPTURE_FILE" "AIBundle"; then
                            print_test_result "PASS" "$TEST_NAME"
                            exit 0
                        else
                            echo "FAIL: App became unresponsive"
                        fi
                    else
                        echo "FAIL: Select all not repeatable or count incorrect"
                    fi
                else
                    echo "FAIL: Items not properly deselected or count not 0"
                fi
            else
                echo "FAIL: No items selected after 'a' key or count incorrect"
            fi
        else
            echo "FAIL: Initial state already has selected files in the file list"
        fi
    else
        echo "FAIL: Initial state does not show 0 selected in status bar"
    fi

    print_test_result "FAIL" "$TEST_NAME"
    exit 1

} || {
    echo "Test execution failed"
    print_test_result "FAIL" "$TEST_NAME"
    exit 1
}