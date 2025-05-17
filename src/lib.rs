// File: src/lib.rs

// First, import any modules and re-export the types for Rust usage
mod material;
pub use material::Material;

// Or if Material is defined directly in lib.rs:
// #[derive(Debug, Clone)]
// pub struct Material { ... }
// impl Material { ... }

// Then, conditionally define the Python module
#[cfg(feature = "pyo3")]
mod python {
    use pyo3::prelude::*;
    use super::Material;
    
    #[pymodule]
    fn materials_for_mc(_py: Python, m: &PyModule) -> PyResult<()> {
        m.add_class::<Material>()?;
        Ok(())
    }
}

// You can re-export Python module for Maturin to find
#[cfg(feature = "pyo3")]
pub use python::*;