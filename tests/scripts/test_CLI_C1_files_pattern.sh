#!/bin/bash

# Test script for aibundle CLI: --files option with single pattern

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Files Pattern"
echo "Starting test: $TEST_NAME"

# Setup temporary directory for output files
setup_temp_dir
trap cleanup_temp_dir EXIT SIGINT SIGTERM

TEST_SOURCE_DIR="tests/files" # Relative to project root
OUTPUT_FILE_PROJECT_RELATIVE="$TEMP_TEST_DIR_PATH_FROM_ROOT/files_pattern_output.txt"

# Test: --files with a single file pattern (e.g., *.py)
echo "Test: --files '*.py'"
run_aibundle_cli "$TEST_SOURCE_DIR --files '*.py' --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - File Exists"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains main.py"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "config.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains config.py"

# Ensure non-Python files are not present
if grep -q "requirements.txt" "../../$OUTPUT_FILE_PROJECT_RELATIVE"; then
    echo -e "${RED}✗ File unexpectedly contains 'requirements.txt' for --files '*.py'.${NC}"
    print_cli_test_result "FAIL" "$TEST_NAME - Should Not Contain requirements.txt"
fi
echo "--files pattern test passed."

print_cli_test_result "PASS" "$TEST_NAME"