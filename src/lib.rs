// First, import any modules and re-export the types for Rust usage
mod material;
mod materials;
mod nuclide;
pub use material::Material;
pub use materials::Materials;

// Import PyO3 items conditionally
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::pymodule;
#[cfg(feature = "pyo3")]
use pyo3::wrap_pyfunction;

// Conditionally include the Python modules
#[cfg(feature = "pyo3")]
mod material_python;
#[cfg(feature = "pyo3")]
mod materials_python;
#[cfg(feature = "pyo3")]
mod nuclide_python;

// Re-export Python modules for Maturin to find
#[cfg(feature = "pyo3")]
pub use material_python::*;
#[cfg(feature = "pyo3")]
pub use materials_python::*;
#[cfg(feature = "pyo3")]
pub use nuclide_python::*;
#[cfg(feature = "pyo3")]
pub use nuclide_python::PyNuclide as Nuclide;

// If you have a main Python module entry point, update it to include PyMaterials:
#[cfg(feature = "pyo3")]
#[pymodule]
fn materials_for_mc(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<material_python::PyMaterial>()?;
    m.add_class::<materials_python::PyMaterials>()?;
    m.add_class::<nuclide_python::PyNuclide>()?;
    m.add_class::<nuclide_python::PyReaction>()?;
    m.add_class::<nuclide_python::PyTemperatureEntry>()?;
    m.add_function(wrap_pyfunction!(nuclide_python::py_read_nuclide_from_json, m)?)?;
    Ok(())
}