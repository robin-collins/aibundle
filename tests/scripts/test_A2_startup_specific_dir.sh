#!/bin/bash

# Test A2: Startup with Specific Directory
# Verifies that the app launches with a directory argument and displays that directory

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common functions
source "$SCRIPT_DIR/common_test_functions.sh"

# Test configuration
TEST_NAME="A2_startup_specific_dir"
CAPTURE_FILE="test_capture-A2.txt"

echo "=========================================="
echo "Starting Test A2: Startup with Specific Directory"
echo "=========================================="

# Setup test session
setup_test_session "$TEST_NAME"

# Test execution
{
    echo "Step 1: Launch application with models directory argument"
    launch_app "models" 3

    echo "Step 2: Capture initial screen"
    capture_pane "$CAPTURE_FILE"

    echo "Step 3: Verify expected UI elements"
    EXPECTED_TEXTS=(
        "AIBundle"
        "models"
    )

    # Verify basic UI elements are present
    if verify_multiple_texts "$CAPTURE_FILE" "${EXPECTED_TEXTS[@]}"; then
        echo "Step 4: Verify models directory is shown in header"
        if verify_text_exists "$CAPTURE_FILE" "models"; then
            echo "Step 5: Verify models directory contents are displayed"
            # Should show model files like user.py, task.py, __init__.py
            if verify_text_exists "$CAPTURE_FILE" "user.py" || verify_text_exists "$CAPTURE_FILE" "__init__.py"; then
                echo "Step 6: Verify navigation is possible"
                # Navigate up and down to test basic movement
                send_keys "j" 0.5
                send_keys "k" 0.5

                # Capture after navigation
                capture_pane "test_capture-A2-after-nav.txt"

                print_test_result "PASS" "$TEST_NAME"
                exit 0
            else
                echo "FAIL: Models directory contents not visible"
            fi
        else
            echo "FAIL: Models directory not shown in header"
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