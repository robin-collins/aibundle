#!/bin/bash

# Test C1: Single Item Selection/Deselection
# Verifies that Space key selects/deselects items and status bar updates correctly

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common functions
source "$SCRIPT_DIR/common_test_functions.sh"

# Test configuration
TEST_NAME="C1_single_selection"
CAPTURE_FILE="test_capture-C1.txt"

echo "=========================================="
echo "Starting Test C1: Single Item Selection"
echo "=========================================="

# Setup test session
setup_test_session "$TEST_NAME"

# Test execution
{
    echo "Step 1: Launch application"
    launch_app "" 3

    echo "Step 2: Capture initial screen (no selections)"
    capture_pane "test_capture-C1-initial.txt"

    echo "Step 3: Navigate to first item"
    send_keys "Home" 1

    echo "Step 4: Select first item with Space"
    send_keys "Space" 1
    capture_pane "test_capture-C1-after-select.txt"

    echo "Step 5: Verify selection indicator appears"
    if verify_text_exists "test_capture-C1-after-select.txt" "\[x\]"; then
        echo "✓ Selection indicator found"

        echo "Step 6: Verify status bar shows updated count"
        # Status bar should show selected count > 0 (e.g., "7 items (1 selected)")
        if verify_text_exists "test_capture-C1-after-select.txt" "(1 selected)" || verify_text_exists "test_capture-C1-after-select.txt" "selected)" && ! verify_text_exists "test_capture-C1-after-select.txt" "(0 selected)"; then
            echo "✓ Status bar shows selection"

            echo "Step 7: Deselect item with Space again"
            send_keys "Space" 1
            capture_pane "test_capture-C1-after-deselect.txt"

            echo "Step 8: Verify selection indicator disappears"
            if verify_text_exists "test_capture-C1-after-deselect.txt" "\[ \]"; then
                echo "✓ Selection indicator cleared"

                echo "Step 9: Test selection on multiple items"
                # Navigate and select a few different items
                send_keys "j" 0.5
                send_keys "Space" 1
                send_keys "j" 0.5
                send_keys "Space" 1
                capture_pane "test_capture-C1-multiple.txt"

                echo "Step 10: Verify multiple selections"
                if verify_text_exists "test_capture-C1-multiple.txt" "\[x\]"; then
                    echo "✓ Multiple selections work"

                    echo "Step 11: Final verification"
                    capture_pane "$CAPTURE_FILE"

                    # Verify the app is still functional
                    if verify_text_exists "$CAPTURE_FILE" "AIBundle"; then
                        print_test_result "PASS" "$TEST_NAME"
                        exit 0
                    else
                        echo "FAIL: App became unresponsive"
                    fi
                else
                    echo "FAIL: Multiple selections not working"
                fi
            else
                echo "FAIL: Selection indicator not cleared properly"
            fi
        else
            echo "FAIL: Status bar not updated with selection"
        fi
    else
        echo "FAIL: Selection indicator not found"
    fi

    print_test_result "FAIL" "$TEST_NAME"
    exit 1

} || {
    echo "Test execution failed"
    print_test_result "FAIL" "$TEST_NAME"
    exit 1
}