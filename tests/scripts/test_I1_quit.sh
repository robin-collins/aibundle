#!/bin/bash

# Test I1: Quit Functionality
# Verifies that 'q' key properly exits the application

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common functions
source "$SCRIPT_DIR/common_test_functions.sh"

# Test configuration
TEST_NAME="I1_quit"
CAPTURE_FILE="test_capture-I1.txt"

echo "=========================================="
echo "Starting Test I1: Quit Functionality"
echo "=========================================="

# Setup test session
setup_test_session "$TEST_NAME"

# Test execution
{
    echo "Step 1: Launch application"
    launch_app "" 3

    echo "Step 2: Capture initial screen"
    capture_pane "test_capture-I1-initial.txt"

    echo "Step 3: Verify app is running"
    if verify_text_exists "test_capture-I1-initial.txt" "AIBundle"; then
        echo "✓ App is running"

        echo "Step 4: Select some items before quitting"
        send_keys "Space" 1  # Select first item
        send_keys "j" 0.5
        send_keys "Space" 1  # Select second item
        capture_pane "test_capture-I1-with-selections.txt"

        echo "Step 5: Quit with 'q' key"
        send_keys "q" 2

        echo "Step 6: Verify application has exited"
        # Wait a moment for the application to fully exit
        sleep 2

        # Try to capture the pane - should be empty or show shell prompt
        capture_pane "$CAPTURE_FILE"

        echo "Step 7: Check if session is still alive"
        if session_alive; then
            echo "Session still alive, checking if app exited to shell"
            # If session is alive, the app should have exited and returned to shell
            if ! verify_text_exists "$CAPTURE_FILE" "AIBundle"; then
                echo "✓ Application exited successfully"

                # Test if we're back to shell
                send_keys "echo 'test_quit_successful'" 1
                capture_pane "test_capture-I1-shell-test.txt"

                if verify_text_exists "test_capture-I1-shell-test.txt" "test_quit_successful"; then
                    echo "✓ Returned to shell successfully"
                    print_test_result "PASS" "$TEST_NAME"
                    exit 0
                else
                    echo "FAIL: Not returned to shell properly"
                fi
            else
                echo "FAIL: Application still running after 'q'"
            fi
        else
            echo "✓ Session terminated (application exited)"
            # If session died, that means the app exited completely
            print_test_result "PASS" "$TEST_NAME"
            exit 0
        fi
    else
        echo "FAIL: App not running initially"
    fi

    print_test_result "FAIL" "$TEST_NAME"
    exit 1

} || {
    echo "Test execution failed"
    print_test_result "FAIL" "$TEST_NAME"
    exit 1
}