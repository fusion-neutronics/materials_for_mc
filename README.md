# **Work in progress, not ready to use**

# Materials for MC

A Rust package with Python bindings for making neutronics materials.



# Developer install

```
python3.11 -m venv .materials_for_mc_env

source .materials_for_mc_env/bin/activate

pip install maturin

maturin develop
```

# Example python usage

```
python examples/use_in_python.py
```

# Example rust usage

```
cargo build
cd examples
cargo build
cargo run examples/use_in_python.py
```

# Testing

```
pytest
cargo test
```