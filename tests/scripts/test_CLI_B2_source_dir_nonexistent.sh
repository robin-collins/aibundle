#!/bin/bash

# Test script for aibundle CLI: Nonexistent source directory

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Nonexistent Source Directory"
echo "Starting test: $TEST_NAME"

# Test: Invalid SOURCE_DIR
echo "Test: Using nonexistent SOURCE_DIR"
run_aibundle_cli "invalid/source/dir/hopefully --output-console"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Exit Code"
assert_stdout_contains "No items selected" || print_cli_test_result "FAIL" "$TEST_NAME - Stdout Message (no items selected)"
echo "Nonexistent SOURCE_DIR test passed."

print_cli_test_result "PASS" "$TEST_NAME"