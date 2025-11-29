#!/usr/bin/bash 

# Common compiler flags
RUSTC="rustc --edition 2021"
LIBS="-lraylib -lm -lpthread -ldl -lrt -lX11"

# List of source files to build
# Output name is auto-derived by removing .rs extension
TARGETS=(
    "gp2d_collision_demo.rs"
    "gp2d_advanced_demo.rs"
    "snake_game.rs"
)

# Files that need panic=abort
PANIC_ABORT=()

echo "Building projects..."

for src in "${TARGETS[@]}"; do
    # Derive output name by removing .rs extension
    out="${src%.rs}"
    
    echo "  Building $out..."
    
    # Check if this file needs panic=abort
    if [[ " ${PANIC_ABORT[@]} " =~ " ${src} " ]]; then
        $RUSTC -C panic=abort "$src" -o "$out" $LIBS
    else
        $RUSTC "$src" -o "$out" $LIBS
    fi
    
    if [ $? -ne 0 ]; then
        echo "  ✗ Failed to build $out"
        exit 1
    fi
done

echo "✓ All builds completed successfully!"
