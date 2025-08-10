#!/bin/bash

# Simple runner script for pathfinding test
# Run this in a separate terminal to test pathfinding with obstacles

set -e

echo "=== Starting Minion Pathfinding Test ==="
echo

# Ensure we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Must run from minion project root directory"
    exit 1
fi

# Set test config directory
TEST_CONFIG_DIR="$HOME/.config/minion_test"

# Check if test setup has been run
if [ ! -f "$TEST_CONFIG_DIR/config.toml" ]; then
    echo "Error: Test configuration not found!"
    echo "Please run './test_pathfinding.sh' first to set up the test environment."
    exit 1
fi

echo "Using test configuration from: $TEST_CONFIG_DIR"
echo "Map: pathfinding_test.bin (32x32 with large obstacles)"
echo
echo "=== Debug Information to Watch For ==="
echo "Look for these messages in the output:"
echo "  • 'Applying X environment objects to navigation grid'"
echo "  • 'Navigation grid: X/Y cells blocked (Z%)'"
echo "  • 'Pathfinding target set: (X, Y, Z)' - when you click"
echo "  • 'Using pathfinding waypoint: (X, Y, Z)' - when pathfinding is active"
echo "  • 'Pathfinding success: raw_path=X waypoints, filtered_path=Y waypoints'"
echo
echo "=== Testing Instructions ==="
echo "1. Click around the map to move the player"
echo "2. Try clicking on the opposite side of trees/rocks"
echo "3. Watch if the player takes detours around obstacles"
echo "4. Check the terminal output for pathfinding debug messages"
echo
echo "Press Ctrl+C to stop the game when done testing."
echo
echo "Starting game in 3 seconds..."
sleep 3

# Set environment variable and run the game
export MINION_CONFIG_DIR="$TEST_CONFIG_DIR"
export RUST_LOG="info,minion::pathfinding=debug,minion::plugins::player=debug"

echo "=== Game Starting ==="
cargo run
