#!/bin/bash

# Config
SESSION="aibundle_test"
APP_ROOT_DIR="/home/tech/projects/aibundle-modular"
APP_PATH="$APP_ROOT_DIR/target/debug/aibundle"
OUTDIR="tests/snapshots"
CURRENT_DIR=$(pwd)

cd "$APP_ROOT_DIR" || exit 1

# Statically defined test sizes
RESOLUTIONS=("80x24" "100x30" "120x40")

# Add the current terminal size from stty
if output=$(stty size 2>/dev/null); then
    read -r rows cols <<< "$output"
    RESOLUTIONS+=("${cols}x${rows}")
else
    echo "Warning: could not get terminal size from 'stty size'"
fi

verify_snapshot() {
    local actual_file="$1"
    local expected_file="$2"
    local label="$3"

    if [ ! -f "$expected_file" ]; then
        echo "⚠ No expected snapshot found for $label"
        return 1
    fi

    if diff -u "$expected_file" "$actual_file" > /dev/null; then
        echo "✅ $label: PASS"
    else
        echo "❌ $label: FAIL (diff below)"
        diff -u "$expected_file" "$actual_file" || true
    fi
}


# Ensure output directory exists
mkdir -p "$OUTDIR"

# Run for each resolution
for res in "${RESOLUTIONS[@]}"; do
    IFS="x" read -r WIDTH HEIGHT <<< "$res"
    OUTPUT_FILE="$OUTDIR/output_${WIDTH}x${HEIGHT}.txt"

    # Clean up any previous session
    tmux kill-session -t $SESSION 2>/dev/null

    echo "🔧 Running test at ${WIDTH}x${HEIGHT}..."

    # Start the app in a detached session at the correct size
    tmux new-session -d -s "$SESSION" -x "$WIDTH" -y "$HEIGHT" "$APP_PATH"

    # Let it initialize and draw
    sleep 1

    # Capture full viewport
    tmux capture-pane -t "$SESSION" -pS -"$HEIGHT" > "$OUTPUT_FILE"

    # Send 'q' to quit
    tmux send-keys -t "$SESSION" "q"

    # Let it quit cleanly
    sleep 0.5

    # Kill the session
    tmux kill-session -t "$SESSION"

    echo "✔ Snapshot saved to $OUTPUT_FILE"
done

cd "$CURRENT_DIR" || exit 1
