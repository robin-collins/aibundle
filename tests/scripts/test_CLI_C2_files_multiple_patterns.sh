#!/bin/bash

# Test script for aibundle CLI: --files option with multiple patterns

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Files Multiple Patterns"
echo "Starting test: $TEST_NAME"

# Setup temporary directory for output files
setup_temp_dir
trap cleanup_temp_dir EXIT SIGINT SIGTERM

TEST_SOURCE_DIR="tests/files" # Relative to project root
OUTPUT_FILE_PROJECT_RELATIVE="$TEMP_TEST_DIR_PATH_FROM_ROOT/files_multi_pattern_output.txt"

# Test: --files with multiple patterns (e.g., '*.py,*.txt')
echo "Test: --files '*.py,*.txt'"
run_aibundle_cli "$TEST_SOURCE_DIR --files '*.py,*.txt' --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - File Exists"

# Check for Python files
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains main.py"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "config.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains config.py"

# Check for text files
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "requirements.txt" || print_cli_test_result "FAIL" "$TEST_NAME - Contains requirements.txt"

# Ensure files with other extensions are not present
if grep -q "\.gitignore" "../../$OUTPUT_FILE_PROJECT_RELATIVE"; then
    echo -e "${RED}✗ File unexpectedly contains '.gitignore' for --files '*.py,*.txt'.${NC}"
    print_cli_test_result "FAIL" "$TEST_NAME - Should Not Contain .gitignore"
fi
echo "--files multiple patterns test passed."

print_cli_test_result "PASS" "$TEST_NAME"