use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;

use crate::nuclide::{Nuclide, ReactionData, read_nuclide_from_json};

#[derive(Clone)]
#[pyclass]
pub struct PyReactionData {
    #[pyo3(get)]
    pub energies: Vec<f64>,
    #[pyo3(get)]
    pub cross_sections: Vec<f64>,
}

#[pyclass]
pub struct PyNuclide {
    #[pyo3(get)]
    pub element: String,
    #[pyo3(get)]
    pub atomic_symbol: String,
    #[pyo3(get)]
    pub proton_number: u32,
    #[pyo3(get)]
    pub neutron_number: u32,
    #[pyo3(get)]
    pub mass_number: u32,
    #[pyo3(get)]
    pub temperature: f64,
    #[pyo3(get)]
    pub reactions: HashMap<u32, PyReactionData>,
}

impl From<Nuclide> for PyNuclide {
    fn from(n: Nuclide) -> Self {
        let reactions = n.reactions.into_iter().map(|(k, v)| (k, PyReactionData {
            energies: v.energies,
            cross_sections: v.cross_sections,
        })).collect();
        PyNuclide {
            element: n.element,
            atomic_symbol: n.atomic_symbol,
            proton_number: n.proton_number,
            neutron_number: n.neutron_number,
            mass_number: n.mass_number,
            temperature: n.temperature,
            reactions,
        }
    }
}

#[pyfunction]
pub fn py_read_nuclide_from_json(path: &str) -> PyResult<PyNuclide> {
    let nuclide = read_nuclide_from_json(path).map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    Ok(PyNuclide::from(nuclide))
}

#[pymodule]
fn nuclide_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyNuclide>()?;
    m.add_class::<PyReactionData>()?;
    m.add_function(wrap_pyfunction!(py_read_nuclide_from_json, m)?)?;
    Ok(())
}
