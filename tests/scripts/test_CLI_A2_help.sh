#!/bin/bash

# Test script for aibundle CLI: --help option
# Tests both --help and -h for showing help information

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Help Check"
echo "Starting test: $TEST_NAME"

# Test 1: --help
echo "Test 1: Checking --help"
run_aibundle_cli "--help"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Help Check"
assert_stdout_contains "Usage: aibundle" || print_cli_test_result "FAIL" "$TEST_NAME - Help Usage Output"
assert_stdout_contains "Options:" || print_cli_test_result "FAIL" "$TEST_NAME - Help Options Output"
assert_stdout_contains "Arguments:" || assert_stdout_contains "ARGS:" || print_cli_test_result "FAIL" "$TEST_NAME - Help Arguments Output"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Help Stderr"
echo "Help test passed."

# Test 2: -h (short for --help)
echo "Test 2: Checking -h"
run_aibundle_cli "-h"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Short Help Check"
assert_stdout_contains "Usage: aibundle" || print_cli_test_result "FAIL" "$TEST_NAME - Short Help Usage Output"
assert_stdout_contains "Options:" || print_cli_test_result "FAIL" "$TEST_NAME - Short Help Options Output"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Short Help Stderr"
echo "Short help test passed."

print_cli_test_result "PASS" "$TEST_NAME"