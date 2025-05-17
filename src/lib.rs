// First, import any modules and re-export the types for Rust usage
mod material;
pub use material::Material;

// Conditionally include the Python module
#[cfg(feature = "pyo3")]
mod python;

// Re-export Python module for Maturin to find
#[cfg(feature = "pyo3")]
pub use python::*;