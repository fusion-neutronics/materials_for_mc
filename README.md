# Materials for MC

A Rust package with Python bindings and WebAssembly support for making neutronics materials.

Features:

- Built up materials
    - set densities
    - add nuclides
- Set the nuclear data paths
    - Package caches nuclides to avoid duplicate reading
- Get unified energy grid for material
- Calculate MT reactions cross sections at specified energy
- Calculate total macroscopic cross section
- Calculate mean free path
- Bindings for Python and WASM are fully optional

## Prerequisites

Depending on your usage you may need to  Rust, Python 
```
git clone git@github.com:fusion-neutronics/materials_for_mc.git
cd materials_for_mc
```

## Example python usage

```
python3.11 -m venv .materials_for_mc_env

source .materials_for_mc_env/bin/activate

pip install maturin

maturin develop --features pyo3

python examples/use_in_python.py
```

## Example rust usage

```
cargo build
cd example_use
cargo build
cargo run
```

## WebAssembly Support

The package can be compiled to WebAssembly for use in web browsers:

to test the wasm bindings
```bash
wasm-pack test --headless --firefox
```

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
wasm-pack build --target web --features wasm
cp -r pkg examples/wasm/
# Serve the demo pages locally
python -m http.server 8000
# Open the demo pages in your browser, I use firefox here but others should also work.
firefox http://localhost:8000/examples/wasm/reaction_plotter.html
```

The WebAssembly demos include:
- Material creation and manipulation
- Cross section calculation and visualization
- Predefined materials
- Interactive plotting with Plotly


## Testing

Tests are present for both the core Rust code and the Python wrapper
```
cargo test
pytest
```

### WebAssembly (WASM) Testing

To run tests for the WASM bindings, you must enable the optional `wasm-test` feature and use a browser (e.g., Firefox):

```bash
cargo test --features wasm-test
wasm-pack test --headless --firefox --features wasm-test
```

This ensures that WASM-specific tests and dependencies are only included when needed.