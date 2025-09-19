# WebAssembly

The materials_for_mc library provides WebAssembly (WASM) bindings, allowing the library to run in web browsers and other JavaScript environments. This enables client-side nuclear data processing and visualization without requiring server-side computation.

## Features

The WebAssembly version supports:
- Material creation and manipulation
- Cross section calculation and visualization  
- Predefined materials
- Interactive plotting with Plotly
- All core functionality available in the Rust version

## Building WebAssembly

### Prerequisites

First, install the required tools:

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Building the Package

To quickly build the WebAssembly package:

```bash
wasm-pack build --target web --features wasm
```

To build the optimized WebAssembly package:
```bash
wasm-pack build --target web --features wasm --release
```

This creates a `pkg/` directory containing the generated WebAssembly files and JavaScript bindings.

Create an index.html that makes use of the WASM file.

Copy the generated package to the examples directory and serve locally:

```bash
cp -r pkg examples/wasm/
# Serve the demo pages locally
python -m http.server 8000
# Open the demo pages in your browser
firefox http://localhost:8000/examples/wasm/index.html
```

### Running WASM Tests

To test the WebAssembly bindings:

```bash
wasm-pack test --headless --firefox
```

### Running WASM-Specific Tests

For tests that require WASM-specific features, enable the optional `wasm-test` feature:

```bash
cargo test --features wasm-test
wasm-pack test --headless --firefox --features wasm-test
```

This ensures that WASM-specific tests and dependencies are only included when needed.


## JavaScript API

The WebAssembly build exposes a JavaScript API that mirrors the Rust API. Example usage:

```javascript
import init, { Material, Nuclide } from './pkg/materials_for_mc.js';

async function main() {
    await init();
    
    // Create a material
    let material = new Material();
    material.add_element("Li", 1.0);
    material.set_density("g/cm3", 0.534);
    
    // Calculate cross sections
    let xs_data = material.calculate_macroscopic_xs_neutron([1, 2], false);
    console.log("Cross section data:", xs_data);
}

main();
```
