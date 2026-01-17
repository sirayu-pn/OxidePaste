#!/bin/bash
# Build script for multiple platforms

set -e

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu"
    "aarch64-unknown-linux-musl"
)

mkdir -p dist

echo "=== OxidePaste Build Script ==="

# Build for current platform
build_native() {
    echo "Building for native platform..."
    cargo build --release
    echo "Done! Binary at: target/release/oxide-paste"
}

# Build for specific target
build_target() {
    local target=$1
    echo "Building for $target..."
    
    # Check if target is installed
    if ! rustup target list --installed | grep -q "$target"; then
        echo "Installing target: $target"
        rustup target add "$target"
    fi
    
    cargo build --release --target "$target"
    
    local name=$(echo "$target" | sed 's/unknown-//' | sed 's/-gnu//' | sed 's/-musl/-static/')
    cp "target/$target/release/oxide-paste" "dist/oxide-paste-$name"
    echo "Created: dist/oxide-paste-$name"
}

# Build all targets
build_all() {
    echo "Building for all targets..."
    for target in "${TARGETS[@]}"; do
        build_target "$target" || echo "Warning: Failed to build for $target"
    done
    echo "All builds complete! Check dist/ folder"
}

# Show help
show_help() {
    echo "Usage: ./build.sh [command]"
    echo ""
    echo "Commands:"
    echo "  native    Build for current platform (default)"
    echo "  all       Build for all supported Linux targets"
    echo "  TARGET    Build for specific target (e.g., aarch64-unknown-linux-gnu)"
    echo ""
    echo "Supported targets:"
    for target in "${TARGETS[@]}"; do
        echo "  - $target"
    done
}

case "${1:-native}" in
    native)
        build_native
        ;;
    all)
        build_all
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        if [[ " ${TARGETS[*]} " =~ " ${1} " ]]; then
            build_target "$1"
        else
            echo "Unknown target: $1"
            show_help
            exit 1
        fi
        ;;
esac
