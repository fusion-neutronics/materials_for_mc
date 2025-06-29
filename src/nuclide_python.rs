#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::types::PyDict;
use std::collections::HashMap;
use crate::nuclide::{Nuclide, Reaction, TemperatureEntry};

#[cfg(feature = "pyo3")]
#[pyclass]
#[derive(Clone)]
pub struct PyReaction {
    #[pyo3(get)]
    pub reaction_products: String,
    #[pyo3(get)]
    pub mt_reaction_number: u32,
    #[pyo3(get)]
    pub cross_section: Vec<f64>,
    #[pyo3(get)]
    pub energy: Vec<f64>,
}

#[cfg(feature = "pyo3")]
#[pyclass]
#[derive(Clone)]
pub struct PyTemperatureEntry {
    #[pyo3(get)]
    pub temps: HashMap<String, Vec<PyReaction>>,
}

#[cfg(feature = "pyo3")]
#[pyclass]
#[derive(Clone)]
pub struct PyNuclide {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub data: Option<Nuclide>,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyNuclide {
    #[new]
    pub fn new(name: String) -> Self {
        PyNuclide { name, data: None }
    }

    pub fn read_nuclide_from_json(&mut self, path: &str) -> PyResult<()> {
        let nuclide = crate::nuclide::read_nuclide_from_json(path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        self.data = Some(nuclide);
        Ok(())
    }
}

#[cfg(feature = "pyo3")]
impl From<Reaction> for PyReaction {
    fn from(r: Reaction) -> Self {
        PyReaction {
            reaction_products: r.reaction_products,
            mt_reaction_number: r.mt_reaction_number,
            cross_section: r.cross_section,
            energy: r.energy,
        }
    }
}

#[cfg(feature = "pyo3")]
impl From<TemperatureEntry> for PyTemperatureEntry {
    fn from(t: TemperatureEntry) -> Self {
        let temps = t.temps.into_iter()
            .map(|(k, v)| (k, v.into_iter().map(PyReaction::from).collect()))
            .collect();
        PyTemperatureEntry { temps }
    }
}

#[cfg(feature = "pyo3")]
impl From<Nuclide> for PyNuclide {
    fn from(n: Nuclide) -> Self {
        PyNuclide {
            name: n.element.clone(),
            data: Some(n),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pyfunction]
pub fn py_read_nuclide_from_json(path: &str) -> PyResult<PyNuclide> {
    let nuclide = crate::nuclide::read_nuclide_from_json(path)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    Ok(PyNuclide::from(nuclide))
}

// Remove the pymodule definition as it's conflicting with the module definition in lib.rs
// The PyNuclide class will be exposed through the main module in lib.rs instead
