#!/bin/bash

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Find llvm tools
LLVM_COV=$(find ~/.rustup -name llvm-cov)

if [ -z "$LLVM_COV" ]; then
    echo "Error: LLVM tools not found. Please install them with:"
    echo "rustup component add llvm-tools-preview"
    exit 1
fi

if [ ! -f "$SCRIPT_DIR/coverage/grid_game.profdata" ]; then
    echo "Error: No coverage data found at $SCRIPT_DIR/coverage/grid_game.profdata"
    echo "Please run coverage.sh first to generate coverage data"
    exit 1
fi

echo -e "\n=== Coverage Summary ===\n"
$LLVM_COV report target/debug/Grid-Game \
    --instr-profile="$SCRIPT_DIR/coverage/grid_game.profdata" \
    src/engine/systems/collisions.rs

echo -e "\n=== Detailed Coverage Report ===\n"
$LLVM_COV show target/debug/Grid-Game \
    --instr-profile="$SCRIPT_DIR/coverage/grid_game.profdata" \
    --show-instantiations \
    --show-line-counts-or-regions \
    --use-color \
    src/engine/systems/collisions.rs
