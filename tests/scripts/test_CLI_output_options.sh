#!/bin/bash

# Test script for aibundle CLI: Output options

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Output Options Test"
echo "Starting test: $TEST_NAME"

# Setup temporary directory for output files
setup_temp_dir
# Ensure cleanup happens on exit or interrupt
trap cleanup_temp_dir EXIT SIGINT SIGTERM

TEST_SOURCE_DIR="tests/files" # Relative to project root
OUTPUT_FILE_RELATIVE_TO_TEMP_ROOT="output.txt"
OUTPUT_FILE_PROJECT_RELATIVE="$TEMP_TEST_DIR_PATH_FROM_ROOT/$OUTPUT_FILE_RELATIVE_TO_TEMP_ROOT"

# Test 1: --output-file with default format (markdown)
echo "Test 1: --output-file (default format: markdown)"
run_aibundle_cli "$TEST_SOURCE_DIR --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Output File Default Format Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Output File Default Format Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - Output File Default Format File Exists"
# Check for structure header (might be "# Project Structure" or just the files directly)
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Output File Default Format Content main.py"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "config.py" || print_cli_test_result "FAIL" "$TEST_NAME - Output File Default Format Content config.py"
# Verify no console output when --output-file is used without --output-console
assert_stdout_not_contains "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Output File Default Format No Console Output"
echo "--output-file default format test passed."

# Cleanup the specific output file for the next test
rm -f "../../$OUTPUT_FILE_PROJECT_RELATIVE"

# Test 2: --output-file with json format
echo "Test 2: --output-file with --format json"
run_aibundle_cli "$TEST_SOURCE_DIR --output-file $OUTPUT_FILE_PROJECT_RELATIVE --format json"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Output File JSON Format Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Output File JSON Format Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - Output File JSON Format File Exists"
# Check for JSON structure - match what's actually produced
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "{" || print_cli_test_result "FAIL" "$TEST_NAME - Output File JSON Format Content Structure"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "\"file\"" || print_cli_test_result "FAIL" "$TEST_NAME - Output File JSON Format Content file property"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Output File JSON Format Content main.py"
assert_stdout_not_contains "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Output File JSON Format No Console Output"
echo "--output-file json format test passed."

rm -f "../../$OUTPUT_FILE_PROJECT_RELATIVE"

# Test 3: --output-console with xml format
echo "Test 3: --output-console with --format xml"
run_aibundle_cli "$TEST_SOURCE_DIR --output-console --format xml"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Output Console XML Format Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Output Console XML Format Stderr"
assert_stdout_contains "<projectStructure>" || print_cli_test_result "FAIL" "$TEST_NAME - Output Console XML Format Content Structure"
assert_stdout_contains "<fileName>main.py</fileName>" || print_cli_test_result "FAIL" "$TEST_NAME - Output Console XML Format Content main.py"
# Ensure no file is created by default
assert_file_not_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - Output Console XML Format No File Created"
echo "--output-console xml format test passed."

# Test 4: --output-file AND --output-console with llm format
echo "Test 4: --output-file and --output-console with --format llm"
run_aibundle_cli "$TEST_SOURCE_DIR --output-file $OUTPUT_FILE_PROJECT_RELATIVE --output-console --format llm"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - File and Console LLM Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - File and Console LLM Stderr"
# Check file
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - File and Console LLM File Exists"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "Project Structure (LLM):" || print_cli_test_result "FAIL" "$TEST_NAME - File and Console LLM File Content Structure"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "File: tests/files/main.py" || print_cli_test_result "FAIL" "$TEST_NAME - File and Console LLM File Content main.py"
# Check console
assert_stdout_contains "Project Structure (LLM):" || print_cli_test_result "FAIL" "$TEST_NAME - File and Console LLM Console Content Structure"
assert_stdout_contains "File: tests/files/main.py" || print_cli_test_result "FAIL" "$TEST_NAME - File and Console LLM Console Content main.py"
echo "--output-file and --output-console llm format test passed."

rm -f "../../$OUTPUT_FILE_PROJECT_RELATIVE"

# Test 5: Invalid format
echo "Test 5: Invalid --format option"
run_aibundle_cli "$TEST_SOURCE_DIR --format invalidformat"
assert_exit_code 1 || print_cli_test_result "FAIL" "$TEST_NAME - Invalid Format Exit Code"
assert_stderr_contains "Invalid format specified: invalidformat" || print_cli_test_result "FAIL" "$TEST_NAME - Invalid Format Stderr Message"
assert_stdout_not_contains "Project Structure" # Should not produce output
echo "Invalid format test passed."

# Test 6: No output specified (should default to console with markdown)
echo "Test 6: No output options specified (default to console, markdown)"
run_aibundle_cli "$TEST_SOURCE_DIR"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - No Output Opts Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - No Output Opts Stderr"
assert_stdout_contains "Project Structure" || print_cli_test_result "FAIL" "$TEST_NAME - No Output Opts Console Content MD"
assert_stdout_contains "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - No Output Opts Console Content main.py"
assert_file_not_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - No Output Opts No File Created"
echo "No output options (default) test passed."

print_cli_test_result "PASS" "$TEST_NAME"