# Materials for MC

Make materials for use in neutronics workflows.

🦀Rust back end for speed
🐍Python API for ease of use
🕸️WASM API for online deployment

General features:
- Build Nuclides
    - Get microscopic cross sections 
- Build up Materials
    - Set densities
    - Add nuclides
    - Add elements
    - Get macroscopic cross sections
- Set the nuclear data paths
    - Per nuclide, material or globally
    - Using local JSON files
    - On the fly downloading
    - Caches nuclides to avoid duplicate reading

Features speciffically for usage Monte Carlo Tranport 
- Get unified energy grid for material
- Calculate MT reactions cross sections at specified energy
- Calculate total macroscopic cross section
- Calculate mean free path

## User install

Install as a Python package if you want to use the package with your own Python code
```bash
pip install materials_for_mc
```

Install the Rust crate if you want to use the package with your own Rust code
```
cargo install materials_for_mc
```

## Developer install

Depending on your usage you may need to  Rust, Python 
```
git clone git@github.com:fusion-neutronics/materials_for_mc.git
cd materials_for_mc
```

### Example python usage

```
python3.11 -m venv .materials_for_mc_env

source .materials_for_mc_env/bin/activate

pip install maturin

maturin develop --features pyo3

python examples/use_in_python.py
```

### Example rust usage

```
cargo build
cd example_use
cargo build
cargo run
```

### WebAssembly Support

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


### Testing

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
