# **Work in progress, not ready to use**

# Materials for MC

A Rust package with Python bindings and WebAssembly support for making neutronics materials.



# Developer install

```
python3.11 -m venv .materials_for_mc_env

source .materials_for_mc_env/bin/activate

pip install maturin

maturin develop --features pyo3
```

# Example python usage

```
python examples/use_in_python.py
```

# Example rust usage

```
cargo build
cd example_use
cargo build
cargo run
```

# WebAssembly Support

The package can be compiled to WebAssembly for use in web browsers:

```bash
# Build the WASM package
./build_wasm.sh

# Serve the demo page locally
./serve.sh

# Open the demo page in your browser
# http://localhost:8000/macroscopic_xs_plotter.html
```

The WebAssembly demo includes:
- Material creation and manipulation
- Cross section calculation and visualization
- Predefined materials (Natural Lithium and Enriched Lithium)
- Interactive plotting with Plotly

# Testing

```
pytest
cargo test
```