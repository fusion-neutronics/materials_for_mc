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
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
wasm-pack build --target web --features wasm

# Serve the demo pages locally
python -m http.server 8000
# Open the demo pages in your browser:
# http://localhost:8000/macroscopic_xs_plotter.html
# http://localhost:8000/reaction_plotter.html
```

The WebAssembly demos include:
- Material creation and manipulation
- Cross section calculation and visualization
- Predefined materials (Natural Lithium and Enriched Lithium)
- Interactive plotting with Plotly

# Testing

```
pytest
cargo test
```