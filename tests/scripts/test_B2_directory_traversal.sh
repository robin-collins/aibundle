#!/bin/bash

# Test B2: Directory Traversal
# Verifies that Enter key navigates into directories and ../ works for going back

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common functions
source "$SCRIPT_DIR/common_test_functions.sh"

# Test configuration
TEST_NAME="B2_directory_traversal"
CAPTURE_FILE="test_capture-B2.txt"

echo "=========================================="
echo "Starting Test B2: Directory Traversal"
echo "=========================================="

# Setup test session
setup_test_session "$TEST_NAME"

# Test execution
{
    echo "Step 1: Launch application"
    launch_app "" 3

    echo "Step 2: Capture initial screen (should be in tests/files)"
    capture_pane "test_capture-B2-initial.txt"

    echo "Step 3: Navigate to models directory"
    # Find and navigate to models directory
    send_keys "Home" 1
    # Look for models directory by navigating
    for i in {1..10}; do
        capture_pane "test_capture-B2-search-models-$i.txt"
        if verify_text_exists "test_capture-B2-search-models-$i.txt" "📁 models/"; then
            echo "✓ Found models directory at position $i"
            break
        fi
        send_keys "j" 0.3
    done

    echo "Step 4: Enter models directory with Enter key"
    send_keys "Return" 2
    capture_pane "test_capture-B2-in-models.txt"

    echo "Step 5: Verify we're in models directory"
    if verify_text_exists "test_capture-B2-in-models.txt" "models" && (verify_text_exists "test_capture-B2-in-models.txt" "user.py" || verify_text_exists "test_capture-B2-in-models.txt" "__init__.py"); then
        echo "✓ Successfully navigated into models directory"

        echo "Step 6: Look for ../ entry"
        send_keys "Home" 1
        capture_pane "test_capture-B2-look-for-parent.txt"

        echo "Step 7: Navigate back using ../ entry"
        # The ../ should be at the top
        if verify_text_exists "test_capture-B2-look-for-parent.txt" "../" || verify_text_exists "test_capture-B2-look-for-parent.txt" ".."; then
            echo "✓ Found ../ entry"
            send_keys "Return" 2
            capture_pane "test_capture-B2-back-to-parent.txt"

            echo "Step 8: Verify we're back in parent directory"
            if verify_text_exists "test_capture-B2-back-to-parent.txt" "main.py" || verify_text_exists "test_capture-B2-back-to-parent.txt" "config.py"; then
                echo "✓ Successfully navigated back to parent directory"

                echo "Step 9: Test navigation into another directory (services)"
                # Navigate to services directory
                send_keys "Home" 1
                for i in {1..10}; do
                    capture_pane "test_capture-B2-search-services-$i.txt"
                    if verify_text_exists "test_capture-B2-search-services-$i.txt" "📁 services/"; then
                        echo "✓ Found services directory"
                        send_keys "Return" 2
                        break
                    fi
                    send_keys "j" 0.3
                done

                capture_pane "$CAPTURE_FILE"

                echo "Step 10: Verify final state"
                if verify_text_exists "$CAPTURE_FILE" "services" && (verify_text_exists "$CAPTURE_FILE" "data_service.py" || verify_text_exists "$CAPTURE_FILE" "auth_service.py"); then
                    echo "✓ Directory traversal working correctly"
                    print_test_result "PASS" "$TEST_NAME"
                    exit 0
                else
                    echo "FAIL: Could not navigate into services directory"
                fi
            else
                echo "FAIL: Did not return to parent directory properly"
            fi
        else
            echo "FAIL: ../ entry not found"
        fi
    else
        echo "FAIL: Could not navigate into models directory"
    fi

    print_test_result "FAIL" "$TEST_NAME"
    exit 1

} || {
    echo "Test execution failed"
    print_test_result "FAIL" "$TEST_NAME"
    exit 1
}