#!/bin/bash

# Test G1: Help Modal
# Verifies that help modal opens with 'h', can be navigated, and closed

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common functions
source "$SCRIPT_DIR/common_test_functions.sh"

# Test configuration
TEST_NAME="G1_help_modal"
CAPTURE_FILE="test_capture-G1.txt"

echo "=========================================="
echo "Starting Test G1: Help Modal"
echo "=========================================="

# Setup test session
setup_test_session "$TEST_NAME"

# Test execution
{
    echo "Step 1: Launch application"
    launch_app "" 3

    echo "Step 2: Capture initial screen"
    capture_pane "test_capture-G1-initial.txt"

    echo "Step 3: Open help modal with 'h'"
    send_keys "h" 1
    capture_pane "test_capture-G1-help-open.txt"

    echo "Step 4: Verify help modal opened"
    if verify_text_exists "test_capture-G1-help-open.txt" "Keyboard" || verify_text_exists "test_capture-G1-help-open.txt" "Help" || verify_text_exists "test_capture-G1-help-open.txt" "Controls"; then
        echo "✓ Help modal opened"

        echo "Step 5: Test help navigation with Page Down"
        send_keys "Next" 1  # Page Down
        capture_pane "test_capture-G1-help-pagedown.txt"

        echo "Step 6: Test help navigation with Page Up"
        send_keys "Prior" 1  # Page Up
        capture_pane "test_capture-G1-help-pageup.txt"

        echo "Step 7: Verify help content contains key bindings"
        if verify_text_exists "test_capture-G1-help-pageup.txt" "j" || verify_text_exists "test_capture-G1-help-pageup.txt" "Space" || verify_text_exists "test_capture-G1-help-pageup.txt" "q"; then
            echo "✓ Help content contains key bindings"

            echo "Step 8: Close help modal with Escape"
            send_keys "Escape" 1
            capture_pane "test_capture-G1-help-closed.txt"

            echo "Step 9: Verify help modal closed and normal view restored"
            if verify_text_exists "test_capture-G1-help-closed.txt" "main.py" || verify_text_exists "test_capture-G1-help-closed.txt" "models"; then
                echo "✓ Normal view restored after closing help"

                echo "Step 10: Test help with question mark key"
                send_keys "?" 1
                capture_pane "test_capture-G1-help-question.txt"

                echo "Step 11: Verify help opens with '?' key"
                if verify_text_exists "test_capture-G1-help-question.txt" "Help" || verify_text_exists "test_capture-G1-help-question.txt" "Keyboard"; then
                    echo "✓ Help opens with '?' key"

                    echo "Step 12: Close help and verify final state"
                    send_keys "Escape" 1
                    capture_pane "$CAPTURE_FILE"

                    if verify_text_exists "$CAPTURE_FILE" "AIBundle" && verify_text_exists "$CAPTURE_FILE" "main.py"; then
                        print_test_result "PASS" "$TEST_NAME"
                        exit 0
                    else
                        echo "FAIL: App not in normal state after help"
                    fi
                else
                    echo "FAIL: Help not opening with '?' key"
                fi
            else
                echo "FAIL: Normal view not restored after closing help"
            fi
        else
            echo "FAIL: Help content missing key bindings"
        fi
    else
        echo "FAIL: Help modal not opened"
    fi

    print_test_result "FAIL" "$TEST_NAME"
    exit 1

} || {
    echo "Test execution failed"
    print_test_result "FAIL" "$TEST_NAME"
    exit 1
}