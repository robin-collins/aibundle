#!/bin/bash

# Test E1: Toggle Output Format
# Verifies that 'f' key cycles through output formats (LLM, XML, Markdown, JSON)

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common functions
source "$SCRIPT_DIR/common_test_functions.sh"

# Test configuration
TEST_NAME="E1_toggle_format"
CAPTURE_FILE="test_capture-E1.txt"

echo "=========================================="
echo "Starting Test E1: Toggle Output Format"
echo "=========================================="

# Setup test session
setup_test_session "$TEST_NAME"

# Test execution
{
    echo "Step 1: Launch application"
    launch_app "" 3

    echo "Step 2: Capture initial screen (default format - LLM)"
    capture_pane "test_capture-E1-initial.txt"

    echo "Step 3: Verify initial format is LLM"
    if verify_text_exists "test_capture-E1-initial.txt" "\[LLM\]"; then
        echo "✓ Initial format displayed as [LLM]"

        echo "Step 4: Press 'f' to cycle to next format"
        send_keys "f" 1
        capture_pane "test_capture-E1-after-f1.txt"

        echo "Step 5: Verify format changed (should be XML)"
        if verify_text_exists "test_capture-E1-after-f1.txt" "\[XML\]" && ! verify_text_exists "test_capture-E1-after-f1.txt" "\[LLM\]"; then
            echo "✓ Format changed from LLM"

            echo "Step 6: Press 'f' again to cycle to next format"
            send_keys "f" 1
            capture_pane "test_capture-E1-after-f2.txt"

            echo "Step 7: Press 'f' again to cycle to next format"
            send_keys "f" 1
            capture_pane "test_capture-E1-after-f3.txt"

            echo "Step 8: Press 'f' again to cycle back to beginning"
            send_keys "f" 1
            capture_pane "test_capture-E1-after-f4.txt"

            echo "Step 9: Verify we cycled through all formats"
            # Should be back to LLM or first format
            if verify_text_exists "test_capture-E1-after-f4.txt" "\[LLM\]"; then
                echo "✓ Format cycling works correctly"

                echo "Step 10: Test format with multiple presses"
                send_keys "f" 0.5
                send_keys "f" 0.5
                capture_pane "$CAPTURE_FILE"

                echo "Step 11: Verify app is still responsive"
                if verify_text_exists "$CAPTURE_FILE" "AIBundle" && (verify_text_exists "$CAPTURE_FILE" "\[LLM\]" || verify_text_exists "$CAPTURE_FILE" "\[XML\]" || verify_text_exists "$CAPTURE_FILE" "\[Markdown\]" || verify_text_exists "$CAPTURE_FILE" "\[JSON\]"); then
                    echo "✓ App responsive after format toggles"
                    print_test_result "PASS" "$TEST_NAME"
                    exit 0
                else
                    echo "FAIL: App became unresponsive"
                fi
            else
                echo "FAIL: Format cycling not working properly"
            fi
        else
            echo "FAIL: Format not changing with 'f' key"
        fi
    else
        echo "FAIL: Initial format not displayed"
    fi

    print_test_result "FAIL" "$TEST_NAME"
    exit 1

} || {
    echo "Test execution failed"
    print_test_result "FAIL" "$TEST_NAME"
    exit 1
}