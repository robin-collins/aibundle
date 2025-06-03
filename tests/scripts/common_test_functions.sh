#!/bin/bash

# Common test functions for tmux-based TUI testing
# Source this file in test scripts with: source "$(dirname "$0")/common_test_functions.sh"

# Global variables
SESSION_NAME=""
TEST_NAME=""
CAPTURE_DIR="$(pwd)/tests/captured-panes"
APP_BINARY="../../target/release/aibundle"
TEST_TIMEOUT=30

# Tmux session configuration
# Set consistent window dimensions for reliable testing
# Width: number of columns (characters horizontally)
# Height: number of rows (lines vertically)
TMUX_WIDTH=180
TMUX_HEIGHT=40

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Setup test environment
setup_test_session() {
    local test_name="$1"
    TEST_NAME="$test_name"
    SESSION_NAME="test_${test_name}_$$_$(date +%s)"

    # Ensure capture directory exists
    mkdir -p "$CAPTURE_DIR"

    # Check if tmux is available
    if ! command -v tmux &> /dev/null; then
        echo -e "${RED}ERROR: tmux is not installed or not in PATH${NC}"
        exit 1
    fi
    # Create tmux session in tests/files directory
    cd "files" || exit 1

    # Check if application binary exists (now we're in tests/files)
    if [ ! -f "$APP_BINARY" ]; then
        echo -e "${YELLOW}WARNING: Application binary not found at $APP_BINARY${NC}"
        echo -e "${YELLOW}Attempting to use debug binary...${NC}"
        APP_BINARY="../../target/debug/aibundle"
        if [ ! -f "$APP_BINARY" ]; then
            echo -e "${RED}ERROR: Neither release nor debug binary found${NC}"
            echo -e "${RED}Please build the application first: cargo build --release${NC}"
            exit 1
        fi
    fi
    tmux new-session -d -s "$SESSION_NAME" -c "$(pwd)" -x "$TMUX_WIDTH" -y "$TMUX_HEIGHT"

    # Set up timeout for the session
    tmux set-option -t "$SESSION_NAME" remain-on-exit on

    echo -e "${GREEN}Test session '$SESSION_NAME' created for test: $TEST_NAME${NC}"
}

# Launch the application with optional arguments
launch_app() {
    local app_args="$1"
    local init_delay="${2:-2}"

    echo "Launching application with args: $app_args"
    tmux send-keys -t "$SESSION_NAME" "$APP_BINARY $app_args" C-m
    sleep "$init_delay"  # Allow app to initialize
}

# Send keys to the tmux session with optional delay
send_keys() {
    local keys="$1"
    local delay="${2:-0.5}"

    echo "Sending keys: $keys"
    tmux send-keys -t "$SESSION_NAME" "$keys"
    sleep "$delay"
}

# Send keys and press Enter
send_keys_enter() {
    local keys="$1"
    local delay="${2:-0.5}"

    echo "Sending keys with Enter: $keys"
    tmux send-keys -t "$SESSION_NAME" "$keys" C-m
    sleep "$delay"
}

# Capture pane output to file
capture_pane() {
    local capture_file="$1"
    local full_path="$CAPTURE_DIR/$capture_file"

    echo "Capturing pane to: $full_path"
    tmux capture-pane -t "$SESSION_NAME" -p > "$full_path"

    if [ -f "$full_path" ]; then
        echo "Capture saved successfully"
        return 0
    else
        echo -e "${RED}ERROR: Failed to capture pane${NC}"
        return 1
    fi
}

# Check if text exists in captured output
verify_text_exists() {
    local capture_file="$1"
    local expected_text="$2"
    local full_path="$CAPTURE_DIR/$capture_file"

    if [ ! -f "$full_path" ]; then
        echo -e "${RED}ERROR: Capture file not found: $full_path${NC}"
        return 1
    fi

    if grep -q "$expected_text" "$full_path"; then
        echo -e "${GREEN}✓ Found expected text: $expected_text${NC}"
        return 0
    else
        echo -e "${RED}✗ Expected text not found: $expected_text${NC}"
        echo "Capture content:"
        cat "$full_path"
        return 1
    fi
}

# Check if text does NOT exist in captured output
verify_text_not_exists() {
    local capture_file="$1"
    local unexpected_text="$2"
    local full_path="$CAPTURE_DIR/$capture_file"

    if [ ! -f "$full_path" ]; then
        echo -e "${RED}ERROR: Capture file not found: $full_path${NC}"
        return 1
    fi

    if grep -q "$unexpected_text" "$full_path"; then
        echo -e "${RED}✗ Unexpected text found: $unexpected_text${NC}"
        return 1
    else
        echo -e "${GREEN}✓ Confirmed text not present: $unexpected_text${NC}"
        return 0
    fi
}

# Verify multiple text patterns exist
verify_multiple_texts() {
    local capture_file="$1"
    shift
    local texts=("$@")
    local all_found=true

    for text in "${texts[@]}"; do
        if ! verify_text_exists "$capture_file" "$text"; then
            all_found=false
        fi
    done

    if $all_found; then
        echo -e "${GREEN}✓ All expected texts found${NC}"
        return 0
    else
        echo -e "${RED}✗ Some expected texts missing${NC}"
        return 1
    fi
}

# Wait for specific text to appear (with timeout)
wait_for_text() {
    local expected_text="$1"
    local timeout="${2:-10}"
    local interval="${3:-1}"
    local temp_capture="/tmp/wait_capture_$$"

    echo "Waiting for text: $expected_text (timeout: ${timeout}s)"

    for ((i=0; i<timeout; i+=interval)); do
        tmux capture-pane -t "$SESSION_NAME" -p > "$temp_capture"
        if grep -q "$expected_text" "$temp_capture"; then
            echo -e "${GREEN}✓ Text appeared after ${i}s${NC}"
            rm -f "$temp_capture"
            return 0
        fi
        sleep "$interval"
    done

    echo -e "${RED}✗ Text did not appear within ${timeout}s${NC}"
    rm -f "$temp_capture"
    return 1
}

# Cleanup test session
cleanup_session() {
    local force="${1:-false}"

    if [ -n "$SESSION_NAME" ]; then
        echo "Cleaning up session: $SESSION_NAME"
        tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true

        # Additional cleanup - kill any remaining processes
        if $force; then
            pkill -f "$APP_BINARY" 2>/dev/null || true
        fi
    fi
}

# Trap to ensure cleanup on script exit
trap 'cleanup_session true' EXIT INT TERM

# Print test result
print_test_result() {
    local result="$1"
    local test_name="$2"

    echo "==========================================="
    if [ "$result" = "PASS" ]; then
        echo -e "${GREEN}TEST RESULT: PASS - $test_name${NC}"
    else
        echo -e "${RED}TEST RESULT: FAIL - $test_name${NC}"
    fi
    echo "==========================================="
}

# Check if session is still alive
session_alive() {
    tmux has-session -t "$SESSION_NAME" 2>/dev/null
}

# Get session info for debugging
debug_session() {
    echo "Session info:"
    tmux list-sessions | grep "$SESSION_NAME" || echo "Session not found"
    echo "Session windows:"
    tmux list-windows -t "$SESSION_NAME" 2>/dev/null || echo "No windows found"
}