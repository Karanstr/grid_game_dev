#!/bin/bash

# Find llvm tools
LLVM_COV=$(find ~/.rustup -name llvm-cov)

if [ -z "$LLVM_COV" ]; then
    echo "Error: LLVM tools not found. Please install them with:"
    echo "rustup component add llvm-tools-preview"
    exit 1
fi

if [ ! -f "coverage/grid_game.profdata" ]; then
    echo "Error: No coverage data found at coverage/grid_game.profdata"
    echo "Please run coverage.sh first to generate coverage data"
    exit 1
fi

echo -e "\n=== Coverage Summary ===\n"
$LLVM_COV report target/debug/Voxel-Test-1 \
    --instr-profile=coverage/grid_game.profdata \
    src/engine/systems/collisions.rs

echo -e "\n=== Detailed Coverage Report ===\n"
$LLVM_COV show target/debug/Voxel-Test-1 \
    --instr-profile=coverage/grid_game.profdata \
    --show-instantiations \
    --show-line-counts-or-regions \
    --use-color \
    src/engine/systems/collisions.rs
