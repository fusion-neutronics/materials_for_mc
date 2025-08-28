// First, import any modules and re-export the types for Rust usage
mod material;
mod materials;
mod nuclide;
mod utilities;
mod config;
mod reaction;
mod element;

pub use material::Material;
pub use materials::Materials;
pub use utilities::{interpolate_linear, interpolate_log_log};
pub use config::Config;
pub use reaction::Reaction;
pub use element::*;

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
#[cfg(feature = "pyo3")]
mod config_python;
#[cfg(feature = "pyo3")]
mod element_python;

// Re-export Python modules for Maturin to find
#[cfg(feature = "pyo3")]
pub use material_python::*;
#[cfg(feature = "pyo3")]
pub use materials_python::*;
#[cfg(feature = "pyo3")]
pub use nuclide_python::*;
#[cfg(feature = "pyo3")]
pub use nuclide_python::PyNuclide as Nuclide;
#[cfg(feature = "pyo3")]
pub use config_python::*;
#[cfg(feature = "pyo3")]
pub use element_python::*;

// Declare the data module
pub mod data;

#[cfg(feature = "wasm")]
mod data_wasm;
#[cfg(feature = "wasm")]
pub use data_wasm::*;

#[cfg(feature = "pyo3")]
mod data_python;
#[cfg(feature = "pyo3")]
pub use data_python::*;

// WASM Modules
#[cfg(feature = "wasm")]
pub mod material_wasm;
#[cfg(feature = "wasm")]
pub mod config_wasm;
#[cfg(feature = "wasm")]
pub mod nuclide_wasm;
#[cfg(feature = "wasm")]
pub mod reaction_wasm;
#[cfg(feature = "wasm")]
pub mod element_wasm;

// WASM setup
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
}

// Export WASM modules
#[cfg(feature = "wasm")]
pub use material_wasm::*;
#[cfg(feature = "wasm")]
pub use config_wasm::*;
#[cfg(feature = "wasm")]
pub use nuclide_wasm::*;
#[cfg(feature = "wasm")]
pub use reaction_wasm::*;
#[cfg(feature = "wasm")]
pub use element_wasm::*;

// If you have a main Python module entry point, update it to include PyMaterials:
#[cfg(feature = "pyo3")]
#[pymodule]
fn materials_for_mc(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<material_python::PyMaterial>()?;
    m.add_class::<materials_python::PyMaterials>()?;
    m.add_class::<nuclide_python::PyNuclide>()?;
    m.add_class::<nuclide_python::PyReaction>()?;
    m.add_class::<config_python::PyConfig>()?;
    m.add_class::<element_python::PyElement>()?;
    m.add_function(wrap_pyfunction!(nuclide_python::py_read_nuclide_from_json, m)?)?;
    m.add_function(wrap_pyfunction!(nuclide_python::clear_nuclide_cache_py, m)?)?;
    m.add_function(wrap_pyfunction!(crate::data_python::natural_abundance, m)?)?;
    m.add_function(wrap_pyfunction!(crate::data_python::element_nuclides, m)?)?;
    m.add_function(wrap_pyfunction!(crate::data_python::sum_rules, m)?)?;
    m.add_function(wrap_pyfunction!(crate::data_python::element_names, m)?)?;
    m.add_function(wrap_pyfunction!(crate::data_python::atomic_masses, m)?)?;
    Ok(())
}