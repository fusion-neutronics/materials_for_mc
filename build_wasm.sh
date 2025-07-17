#!/bin/bash

# Build script for Materials for MC WASM package
echo "Building Materials for MC WASM package..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM package
wasm-pack build --target web --features wasm

echo "WASM package built successfully!"
echo "The package is available in the 'pkg' directory."
echo "To use it, open the macroscopic_xs_plotter.html file in a web browser."
