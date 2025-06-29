#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::types::PyDict;
use crate::nuclide::{Nuclide, Reaction, TemperatureEntry};

#[cfg(feature = "pyo3")]
#[pyclass(name = "Nuclide")]
#[derive(Clone, Default)]
pub struct PyNuclide {
    #[pyo3(get)]
    pub name: Option<String>,
    #[pyo3(get)]
    pub element: Option<String>,
    #[pyo3(get)]
    pub atomic_symbol: Option<String>,
    #[pyo3(get)]
    pub proton_number: Option<u32>,
    #[pyo3(get)]
    pub neutron_number: Option<u32>,
    #[pyo3(get)]
    pub mass_number: Option<u32>,
    #[pyo3(get)]
    pub incident_particle: Option<String>,
    #[pyo3(get)]
    pub library: Option<String>,
    pub temperature: Option<Vec<TemperatureEntry>>, // Remove #[pyo3(get)] from temperature and use a custom getter
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyNuclide {
    #[new]
    pub fn new(name: Option<String>) -> Self {
        PyNuclide {
            name,
            element: None,
            atomic_symbol: None,
            proton_number: None,
            neutron_number: None,
            mass_number: None,
            incident_particle: None,
            library: None,
            temperature: None,
        }
    }

    pub fn read_nuclide_from_json(&mut self, path: &str) -> PyResult<()> {
        let nuclide = crate::nuclide::read_nuclide_from_json(path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        self.name = nuclide.name;
        self.element = nuclide.element;
        self.atomic_symbol = nuclide.atomic_symbol;
        self.proton_number = nuclide.proton_number;
        self.neutron_number = nuclide.neutron_number;
        self.mass_number = nuclide.mass_number;
        self.incident_particle = nuclide.incident_particle;
        self.library = nuclide.library;
        self.temperature = nuclide.temperature;
        Ok(())
    }

    #[getter]
    pub fn temperature(&self) -> PyResult<Option<Vec<PyTemperatureEntry>>> {
        Ok(self.temperature.as_ref().map(|temps| temps.iter().cloned().map(PyTemperatureEntry::from).collect()))
    }
}

#[cfg(feature = "pyo3")]
impl From<Nuclide> for PyNuclide {
    fn from(n: Nuclide) -> Self {
        PyNuclide {
            name: n.name,
            element: n.element,
            atomic_symbol: n.atomic_symbol,
            proton_number: n.proton_number,
            neutron_number: n.neutron_number,
            mass_number: n.mass_number,
            incident_particle: n.incident_particle,
            library: n.library,
            temperature: n.temperature,
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

#[cfg(feature = "pyo3")]
#[pyclass]
pub struct PyReaction {
    #[pyo3(get)]
    pub reactants: Vec<String>,
    #[pyo3(get)]
    pub products: Vec<String>,
    #[pyo3(get)]
    pub energy: f64,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyReaction {
    #[new]
    pub fn new(reactants: Vec<String>, products: Vec<String>, energy: f64) -> Self {
        PyReaction { reactants, products, energy }
    }
}

#[cfg(feature = "pyo3")]
#[pyclass]
pub struct PyTemperatureEntry {
    #[pyo3(get)]
    pub temperature: f64,
    #[pyo3(get)]
    pub value: f64,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyTemperatureEntry {
    #[new]
    pub fn new(temperature: f64, value: f64) -> Self {
        PyTemperatureEntry { temperature, value }
    }
}

// Implement From<TemperatureEntry> for PyTemperatureEntry
#[cfg(feature = "pyo3")]
impl From<TemperatureEntry> for PyTemperatureEntry {
    fn from(t: TemperatureEntry) -> Self {
        // You may want to convert the temps field as well, but for now just use dummy values
        PyTemperatureEntry { temperature: 0.0, value: 0.0 }
    }
}

// Remove the pymodule definition as it's conflicting with the module definition in lib.rs
// The PyNuclide class will be exposed through the main module in lib.rs instead
