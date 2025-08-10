#!/bin/bash

# Pathfinding Test Script
# This script sets up a controlled environment to test pathfinding with obstacles

set -e  # Exit on any error

echo "=== Minion Pathfinding Test Setup ==="
echo

# Ensure we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Must run from minion project root directory"
    exit 1
fi

# Create test config directory
TEST_CONFIG_DIR="$HOME/.config/minion_test"
mkdir -p "$TEST_CONFIG_DIR"

echo "1. Creating test configuration..."

# Create test-specific config
cat > "$TEST_CONFIG_DIR/config.toml" << 'EOF'
# Minion Pathfinding Test Configuration
# Optimized settings for testing pathfinding with obstacles

username = "PathfindingTester"
score = 0

[settings]

# =============================================================================
# PLAYER SETTINGS - Optimized for testing
# =============================================================================

player_movement_speed = 8.0         # Slower for easier observation
player_max_health = 100.0
player_max_mana = 50.0
player_max_energy = 100.0

# Movement settings for clear pathfinding behavior
player_stopping_distance = 0.3      # Precise stopping
player_slowdown_distance = 1.5      # Quick slowdown for responsive feel

# =============================================================================
# ENEMY SETTINGS - Reduced for cleaner testing
# =============================================================================

enemy_movement_speed = 3.0           # Slower enemies
enemy_max_health = 3.0
enemy_max_mana = 25.0
enemy_max_energy = 50.0
enemy_chase_distance = 6.0           # Shorter chase distance
enemy_collision_distance = 1.0

enemy_stopping_distance = 1.2
enemy_speed_multiplier = 0.6         # Even slower relative to player

# LOD settings - use medium for good performance
max_lod_level = "medium"
enemy_lod_distance_high = 4.0
enemy_lod_distance_low = 12.0

# Spawning settings
enemy_spawn_distance_min = 4.0
enemy_spawn_distance_max = 8.0
score_per_enemy = 10

# =============================================================================
# COMBAT SETTINGS - Standard
# =============================================================================

bullet_speed = 15.0
bullet_damage = 2.0
bullet_lifetime = 3.0
bullet_collision_distance = 0.6

magic_damage_per_second = 150.0
magic_area_radius = 3.0
magic_area_duration = 2.0

poison_damage_per_second = 80.0
poison_area_radius = 4.0
poison_area_duration = 4.0

# =============================================================================
# UI SETTINGS - Smaller window for easier testing
# =============================================================================

window_width = 1024.0               # Smaller window
window_height = 768.0

hud_font_size = 14.0                # Slightly smaller UI
tooltip_font_size = 10.0
max_username_length = 20

# =============================================================================
# MAP SETTINGS - Using test map with large obstacles
# =============================================================================

map_file_path = "assets/maps/pathfinding_test.bin"

# =============================================================================
# VISUAL SETTINGS
# =============================================================================

ambient_light_brightness = 350.0    # Brighter for better visibility

# UI Colors
health_bar_color = [0.8, 0.2, 0.2]
mana_bar_color = [0.2, 0.2, 0.8]
energy_bar_color = [0.8, 0.8, 0.2]
EOF

echo "   ✓ Test config created at: $TEST_CONFIG_DIR/config.toml"

echo
echo "2. Ensuring test map exists..."

# Check if test map exists, create if needed
if [ ! -f "assets/maps/pathfinding_test.bin" ]; then
    echo "   Creating pathfinding test map..."
    cargo run --bin mapgen -- \
        --name pathfinding_test \
        --size 32x32 \
        --terrain-type flat \
        --objects 0.4 \
        --object-types tree,rock \
        --object-scale 2.5,4.0 \
        --scale 1.0 \
        --output pathfinding_test.bin
    echo "   ✓ Test map created"
else
    echo "   ✓ Test map already exists"
fi

echo
echo "3. Displaying test map information..."
cargo run --example map_info -- --input pathfinding_test.bin --verbose

echo
echo "=== Test Environment Ready ==="
echo
echo "INSTRUCTIONS FOR TESTING:"
echo "1. Run the game with: MINION_CONFIG_DIR=\"$TEST_CONFIG_DIR\" cargo run"
echo "2. Look for these debug messages in the logs:"
echo "   - 'Applying X environment objects to navigation grid'"
echo "   - 'Navigation grid: X/Y cells blocked'"
echo "   - 'Pathfinding target set: (X, Y, Z)'"
echo "   - 'Using pathfinding waypoint: (X, Y, Z)'"
echo
echo "3. Test pathfinding by:"
echo "   - Click to move around the large trees/rocks"
echo "   - Try clicking on the opposite side of obstacles"
echo "   - Watch if the player takes detours around obstacles"
echo
echo "4. Expected behavior:"
echo "   - Player should path around large obstacles (trees/rocks)"
echo "   - Debug logs should show waypoints being used"
echo "   - Paths should have multiple waypoints when avoiding obstacles"
echo
echo "5. To run the test:"
echo "   MINION_CONFIG_DIR=\"$TEST_CONFIG_DIR\" cargo run"
echo
echo "Press Ctrl+C to stop the game when testing is complete."
echo
