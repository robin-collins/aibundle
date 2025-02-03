#!/usr/bin/env bash

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cargo fmt && cargo build --release && sudo cp target/release/aibundle /usr/local/bin/aibundle
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    cargo fmt && cargo build --release && cp target/release/aibundle.exe /c/tools/bin/
else
    echo "Unsupported OS"
    exit 1
fi
