# **Work in progress, not ready to use**

# Materials for MC

A Rust package with Python bindings for making neutronics materials.



# Developer install

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

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

# Testing

```
pytest
cargo test
```