#!/bin/bash

# Test Status Overview
# Shows which test scripts exist and their status

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=============================================="
echo -e "${BLUE}AIBundle TUI Test Status${NC}"
echo "=============================================="
echo ""

# Check prerequisites
echo "Prerequisites:"
echo "-------------"

# Check tmux
if command -v tmux &> /dev/null; then
    echo -e "tmux: ${GREEN}✓ Available${NC}"
else
    echo -e "tmux: ${RED}✗ Not found${NC}"
fi

# Check binary
cd "$(dirname "$SCRIPT_DIR")" || exit 1
if [ -f "tests/files/../../target/release/aibundle" ]; then
    echo -e "aibundle (release): ${GREEN}✓ Available${NC}"
elif [ -f "tests/files/../../target/debug/aibundle" ]; then
    echo -e "aibundle (debug): ${YELLOW}⚠ Available (debug only)${NC}"
else
    echo -e "aibundle: ${RED}✗ Not found (run: cargo build --release)${NC}"
fi

echo ""

# Test script status
echo "Test Scripts:"
echo "------------"

TOTAL_SCRIPTS=0
EXECUTABLE_SCRIPTS=0

# List of expected test scripts
EXPECTED_TESTS=(
    "test_A1_default_startup.sh"
    "test_A2_startup_specific_dir.sh"
    "test_B1_list_navigation.sh"
    "test_B2_directory_traversal.sh"
    "test_C1_single_selection.sh"
    "test_C2_select_all.sh"
    "test_D1_search_functionality.sh"
    "test_E1_toggle_format.sh"
    "test_G1_help_modal.sh"
    "test_I1_quit.sh"
)

for test_script in "${EXPECTED_TESTS[@]}"; do
    TOTAL_SCRIPTS=$((TOTAL_SCRIPTS + 1))
    full_path="$SCRIPT_DIR/$test_script"

    if [ -f "$full_path" ]; then
        if [ -x "$full_path" ]; then
            echo -e "${GREEN}✓${NC} $test_script (executable)"
            EXECUTABLE_SCRIPTS=$((EXECUTABLE_SCRIPTS + 1))
        else
            echo -e "${YELLOW}⚠${NC} $test_script (not executable - run: chmod +x)"
        fi
    else
        echo -e "${RED}✗${NC} $test_script (missing)"
    fi
done

echo ""

# Additional scripts
echo "Support Scripts:"
echo "---------------"

SUPPORT_SCRIPTS=(
    "common_test_functions.sh"
    "run_all_tests.sh"
    "test_status.sh"
)

for script in "${SUPPORT_SCRIPTS[@]}"; do
    full_path="$SCRIPT_DIR/$script"
    if [ -f "$full_path" ]; then
        if [ -x "$full_path" ]; then
            echo -e "${GREEN}✓${NC} $script"
        else
            echo -e "${YELLOW}⚠${NC} $script (not executable)"
        fi
    else
        echo -e "${RED}✗${NC} $script (missing)"
    fi
done

echo ""

# Test data status
echo "Test Data:"
echo "---------"

cd "tests/files" 2>/dev/null || {
    echo -e "${RED}✗ tests/files directory not found${NC}"
    exit 1
}

# Check for test files
TEST_FILES=(
    "main.py"
    "config.py"
    "models/__init__.py"
    "services/__init__.py"
    "utils/__init__.py"
    "requirements.txt"
    ".gitignore"
)

for file in "${TEST_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} $file"
    else
        echo -e "${RED}✗${NC} $file (missing)"
    fi
done

cd - > /dev/null

echo ""

# Directories
echo "Directories:"
echo "-----------"

REQUIRED_DIRS=(
    "tests/files"
    "tests/scripts"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo -e "${GREEN}✓${NC} $dir"
    else
        echo -e "${RED}✗${NC} $dir (missing)"
    fi
done

# Check if captured-panes directory exists (created during test runs)
if [ -d "tests/captured-panes" ]; then
    echo -e "${GREEN}✓${NC} tests/captured-panes"

    # Count existing captures
    capture_count=$(find "tests/captured-panes" -name "*.txt" 2>/dev/null | wc -l)
    if [ "$capture_count" -gt 0 ]; then
        echo "  └─ $capture_count capture files found"
    fi
else
    echo -e "${YELLOW}⚠${NC} tests/captured-panes (will be created during test runs)"
fi

echo ""

# Summary
echo "Summary:"
echo "-------"
echo "Test scripts: $EXECUTABLE_SCRIPTS/$TOTAL_SCRIPTS executable"

if [ $EXECUTABLE_SCRIPTS -eq $TOTAL_SCRIPTS ]; then
    echo -e "Status: ${GREEN}Ready to run tests${NC}"
    echo ""
    echo "Run tests with:"
    echo "  ./tests/scripts/run_all_tests.sh"
else
    echo -e "Status: ${YELLOW}Setup incomplete${NC}"
    echo ""
    echo "To fix issues:"
    echo "  chmod +x tests/scripts/*.sh    # Make scripts executable"
    echo "  cargo build --release          # Build application"
fi

echo "=============================================="