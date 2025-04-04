# **Work in progress, not ready to use**

# Materials for MC

A Rust package with Python bindings for making neutronics materials.



# developer install
python3.11 -m venv .materials_for_mc_env

source .materials_for_mc_env/bin/activate

pip install maturin

maturin develop

python examples/use_in_python.py
