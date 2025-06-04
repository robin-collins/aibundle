#!/bin/bash

# Common CLI test functions

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Determine APP_BINARY path relative to project root
# This script is in tests/scripts/
APP_BINARY_FROM_ROOT_RELEASE="target/release/aibundle"
APP_BINARY_FROM_ROOT_DEBUG="target/debug/aibundle"
APP_BINARY_EXEC="" # This will be the executable path from project root

# Check which binary to use (paths are relative to project root, script is in tests/scripts)
if [ -f "../../$APP_BINARY_FROM_ROOT_RELEASE" ]; then
    APP_BINARY_EXEC="$APP_BINARY_FROM_ROOT_RELEASE"
elif [ -f "../../$APP_BINARY_FROM_ROOT_DEBUG" ]; then
    APP_BINARY_EXEC="$APP_BINARY_FROM_ROOT_DEBUG"
else
    echo -e "${RED}ERROR: aibundle binary not found. Expected at $APP_BINARY_FROM_ROOT_RELEASE or $APP_BINARY_FROM_ROOT_DEBUG relative to project root.${NC}"
    exit 1 # Critical error for common functions
fi

# Function to run aibundle CLI command
# Usage: run_aibundle_cli "arguments"
# Sets: CMD_OUTPUT, CMD_ERROR, CMD_EXIT_CODE
run_aibundle_cli() {
    local args="$1"

    # Temporarily navigate to project root to execute the command
    pushd ../.. > /dev/null # Go up from tests/scripts to project root

    echo "Executing from $(pwd): ./$APP_BINARY_EXEC $args"

    local temp_stdout temp_stderr
    temp_stdout=$(mktemp)
    temp_stderr=$(mktemp)

    # Execute the command. Use eval if args might contain complex shell constructs,
    # but direct execution is safer if args are simple strings.
    # Assuming args are passed as a single string, direct execution is fine.
    ./$APP_BINARY_EXEC $args > "$temp_stdout" 2> "$temp_stderr"
    CMD_EXIT_CODE=$?
    CMD_OUTPUT=$(cat "$temp_stdout")
    CMD_ERROR=$(cat "$temp_stderr")

    rm "$temp_stdout" "$temp_stderr"

    popd > /dev/null # Return to original directory (tests/scripts)

    echo "Exit code: $CMD_EXIT_CODE"
    # Optionally print a snippet of output for quick debugging
    # if [ -n "$CMD_OUTPUT" ]; then echo "Stdout (first 5 lines):"; echo "$CMD_OUTPUT" | head -n 5; fi
    # if [ -n "$CMD_ERROR" ]; then echo "Stderr (first 5 lines):"; echo "$CMD_ERROR" | head -n 5; fi
}


# Assert exit code
# Usage: assert_exit_code <expected_code>
assert_exit_code() {
    local expected_code="$1"
    if [ "$CMD_EXIT_CODE" -eq "$expected_code" ]; then
        echo -e "${GREEN}✓ Correct exit code: $CMD_EXIT_CODE${NC}"
        return 0
    else
        echo -e "${RED}✗ Incorrect exit code. Expected $expected_code, got $CMD_EXIT_CODE${NC}"
        echo -e "${YELLOW}Stderr:${NC}
$CMD_ERROR"
        echo -e "${YELLOW}Stdout:${NC}
$CMD_OUTPUT"
        return 1
    fi
}

# Assert stdout contains text (fixed string match)
# Usage: assert_stdout_contains "some text"
assert_stdout_contains() {
    local expected_text="$1"
    if echo "$CMD_OUTPUT" | grep -qF -- "$expected_text"; then
        echo -e "${GREEN}✓ Stdout contains: $expected_text${NC}"
        return 0
    else
        echo -e "${RED}✗ Stdout does not contain: '$expected_text'${NC}"
        # echo "Full Stdout: $CMD_OUTPUT" # Uncomment for debugging
        return 1
    fi
}

# Assert stdout does not contain text (fixed string match)
# Usage: assert_stdout_not_contains "some text"
assert_stdout_not_contains() {
    local unexpected_text="$1"
    if ! echo "$CMD_OUTPUT" | grep -qF -- "$unexpected_text"; then
        echo -e "${GREEN}✓ Stdout does not contain: $unexpected_text${NC}"
        return 0
    else
        echo -e "${RED}✗ Stdout contains unexpected text: '$unexpected_text'${NC}"
        return 1
    fi
}

# Assert stderr contains text (fixed string match)
# Usage: assert_stderr_contains "some error"
assert_stderr_contains() {
    local expected_text="$1"
    if echo "$CMD_ERROR" | grep -qF -- "$expected_text"; then
        echo -e "${GREEN}✓ Stderr contains: $expected_text${NC}"
        return 0
    else
        echo -e "${RED}✗ Stderr does not contain: '$expected_text'${NC}"
        # echo "Full Stderr: $CMD_ERROR" # Uncomment for debugging
        return 1
    fi
}

# Assert stderr is empty
# Usage: assert_stderr_empty
assert_stderr_empty() {
    if [ -z "$CMD_ERROR" ]; then
        echo -e "${GREEN}✓ Stderr is empty${NC}"
        return 0
    else
        echo -e "${RED}✗ Stderr is not empty:${NC}"
        echo "$CMD_ERROR"
        return 1
    fi
}

# Assert file exists
# File path should be relative to the project root
# Usage: assert_file_exists "path/to/file/from/project_root.txt"
assert_file_exists() {
    local file_path_from_root="$1"
    # Check from the script's current directory (tests/scripts)
    if [ -f "../../$file_path_from_root" ]; then
        echo -e "${GREEN}✓ File exists: $file_path_from_root${NC}"
        return 0
    else
        echo -e "${RED}✗ File does not exist: $file_path_from_root (checked as ../../$file_path_from_root)${NC}"
        return 1
    fi
}

# Assert file does not exist
# File path should be relative to the project root
# Usage: assert_file_not_exists "path/to/file/from/project_root.txt"
assert_file_not_exists() {
    local file_path_from_root="$1"
     if [ ! -f "../../$file_path_from_root" ]; then
        echo -e "${GREEN}✓ File does not exist: $file_path_from_root${NC}"
        return 0
    else
        echo -e "${RED}✗ File unexpectedly exists: $file_path_from_root (checked as ../../$file_path_from_root)${NC}"
        return 1
    fi
}

# Assert file content contains text (fixed string match)
# File path should be relative to the project root
# Usage: assert_file_contains "path/to/file_from_project_root.txt" "some text"
assert_file_contains() {
    local file_path_from_root="$1"
    local expected_text="$2"
    local actual_file_path="../../$file_path_from_root"

    if [ ! -f "$actual_file_path" ]; then
        echo -e "${RED}✗ File not found for content check: $file_path_from_root (checked as $actual_file_path)${NC}"
        return 1
    fi
    if grep -qF -- "$expected_text" "$actual_file_path"; then
        echo -e "${GREEN}✓ File '$file_path_from_root' contains: $expected_text${NC}"
        return 0
    else
        echo -e "${RED}✗ File '$file_path_from_root' does not contain: '$expected_text'${NC}"
        # echo "File content (first 10 lines of $file_path_from_root):" # Uncomment for debugging
        # head -n 10 "$actual_file_path" # Uncomment for debugging
        return 1
    fi
}

# Print test result for CLI tests
# Usage: print_cli_test_result "PASS|FAIL" "TestName"
print_cli_test_result() {
    local result="$1"
    local test_name="$2"
    local SCRIPT_NAME=$(basename "$0")

    echo "==========================================="
    if [ "$result" = "PASS" ]; then
        echo -e "${GREEN}CLI TEST RESULT ($SCRIPT_NAME): PASS - $test_name${NC}"
        exit 0
    else
        echo -e "${RED}CLI TEST RESULT ($SCRIPT_NAME): FAIL - $test_name${NC}"
        exit 1
    fi
}

# Setup a temporary test directory (relative to project root)
# Usage: setup_temp_dir
# Sets: TEMP_TEST_DIR_PATH_FROM_ROOT (e.g., "tests/temp_cli_work_XXXX")
# Call this at the beginning of a test script that needs a temp dir.
setup_temp_dir() {
    # Create a unique directory name within tests/
    TEMP_DIR_NAME="temp_cli_$(basename "$0" .sh)_$(date +%s)_$RANDOM"
    TEMP_TEST_DIR_PATH_FROM_ROOT="tests/$TEMP_DIR_NAME"

    # Create it from project root perspective (script is in tests/scripts/)
    mkdir -p "../../$TEMP_TEST_DIR_PATH_FROM_ROOT"
    echo "Temporary test directory created: $TEMP_TEST_DIR_PATH_FROM_ROOT (full path: $(pwd)/../../$TEMP_TEST_DIR_PATH_FROM_ROOT)"
}

# Cleanup temporary test directory
# Usage: cleanup_temp_dir
# Needs TEMP_TEST_DIR_PATH_FROM_ROOT to be set by setup_temp_dir.
# Call this at the end of a test script (e.g., using trap).
cleanup_temp_dir() {
    if [ -n "$TEMP_TEST_DIR_PATH_FROM_ROOT" ] && [ -d "../../$TEMP_TEST_DIR_PATH_FROM_ROOT" ]; then
        echo "Cleaning up temporary test directory: $TEMP_TEST_DIR_PATH_FROM_ROOT"
        rm -rf "../../$TEMP_TEST_DIR_PATH_FROM_ROOT"
    else
        echo "No temporary directory path set or directory not found for cleanup: $TEMP_TEST_DIR_PATH_FROM_ROOT"
    fi
}

# Example of how to use trap in a test script:
# trap cleanup_temp_dir EXIT SIGINT SIGTERM