# Developer Info

This guide shows how to clone, build, test, and generate documentation for the project. Instructions are written for Linux but equivilent commands should be possible in Windows and Mac OS.

## 1. Clone the Repository
```bash
git clone https://github.com/fusion-neutronics/materials_for_mc.git
cd materials_for_mc
```

## 2. Build & Test the Rust Library
Install Rust Linux and Mac: https://www.rust-lang.org/tools/install
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Windows https://forge.rust-lang.org/infra/other-installation-methods.html

Install a C compiler
```
sudo apt update && sudo apt install build-essential
```

Build Rust Package:
```bash
cargo build
```
Run Rust tests:
```bash
cargo test
```

## 3. Build the Python Extension (maturin)

(Optional) Create a Python virtual environment:
```bash
sudo apt install python3-venv
python3 -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
```

Install maturin (once):
```bash
pip install maturin
```

Build & develop-install the Python module:
```bash
maturin develop --features pyo3
```

(Optional) Build a wheel:
```bash
maturin build --features pyo3
```

## 4. Run Python Tests
Install test dependencies:
```bash
pip install pytest
```
Run tests:
```bash
pytest
```

## 5. Build Rust API Docs
```bash
cargo doc --no-deps
# Open target/doc/materials_for_mc/index.html in a browser
```

## 6. Build Python (Sphinx) Docs
Install doc requirements:
```bash
pip install -r docs/requirements.txt
```
Ensure extension is built (maturin develop) then build docs:
```bash
sphinx-build -b html docs/source docs/build/html
```
Open docs:
```bash
python -m webbrowser docs/build/html/index.html
```
