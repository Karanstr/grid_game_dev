#!/bin/bash

# Exit on any error
set -e

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo "Cleaning previous build and coverage data..."
cargo clean
rm -rf "$SCRIPT_DIR/coverage"
mkdir -p "$SCRIPT_DIR/coverage"

echo "Building with coverage instrumentation..."
RUSTFLAGS="-C instrument-coverage" cargo build

echo "Running program with coverage tracking..."
RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="default-%p-%m.profraw" cargo run

echo "Processing coverage data..."
# Find llvm tools
LLVM_PROFDATA=$(find ~/.rustup -name llvm-profdata)
LLVM_COV=$(find ~/.rustup -name llvm-cov)

if [ -z "$LLVM_PROFDATA" ] || [ -z "$LLVM_COV" ]; then
    echo "Error: LLVM tools not found. Please install them with:"
    echo "rustup component add llvm-tools-preview"
    exit 1
fi

# Check for profraw files
PROFRAW_FILES=$(find . -name "default*.profraw")
if [ -z "$PROFRAW_FILES" ]; then
    echo "Error: No coverage data files (*.profraw) found."
    echo "Please make sure to interact with the program and close it properly."
    exit 1
fi

# Move profraw files to coverage directory
echo "Moving coverage data files..."
mv default*.profraw "$SCRIPT_DIR/coverage/"

# Merge coverage data
echo "Merging coverage data..."
$LLVM_PROFDATA merge -sparse "$SCRIPT_DIR/coverage"/*.profraw -o "$SCRIPT_DIR/coverage/grid_game.profdata"

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
