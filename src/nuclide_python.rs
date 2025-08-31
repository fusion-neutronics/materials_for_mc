#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::types::PyDict;
use crate::nuclide::Nuclide;
use crate::reaction::Reaction;
use std::collections::HashMap;

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
    pub atomic_number: Option<u32>,
    #[pyo3(get)]
    pub neutron_number: Option<u32>,
    #[pyo3(get)]
    pub mass_number: Option<u32>,
    #[pyo3(get)]
    pub library: Option<String>,
    pub energy: Option<HashMap<String, Vec<f64>>>,
    pub reactions: HashMap<String, HashMap<i32, Reaction>>,
    #[pyo3(get)]
    pub fissionable: bool,
    #[pyo3(get)]
    pub available_temperatures: Vec<String>,
    #[pyo3(get)]
    pub loaded_temperatures: Vec<String>,
    #[pyo3(get)]
    pub data_path: Option<String>,
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
            atomic_number: None,
            neutron_number: None,
            mass_number: None,
            library: None,
            energy: None,
            reactions: HashMap::new(),
            fissionable: false,
            available_temperatures: Vec::new(),
            loaded_temperatures: Vec::new(),
            data_path: None,
        }
    }

    #[pyo3(signature = (path=None, temperatures=None))]
    pub fn read_nuclide_from_json(&mut self, path: Option<String>, temperatures: Option<Vec<String>>) -> PyResult<()> {
        use std::collections::HashSet;
        let identifier = if let Some(p) = &path { p.as_str() } else {
            self.name.as_deref().ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Nuclide name not set and no path provided"))?
        };
        let temps_set: Option<HashSet<String>> = temperatures.map(|v| v.into_iter().collect());
    let nuclide = crate::nuclide::read_nuclide_from_json(identifier, temps_set.as_ref())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        self.name = nuclide.name;
        self.element = nuclide.element;
        self.atomic_symbol = nuclide.atomic_symbol;
        self.atomic_number = nuclide.atomic_number;
        self.neutron_number = nuclide.neutron_number;
        self.mass_number = nuclide.mass_number;
        self.library = nuclide.library;
        self.energy = nuclide.energy;
        self.reactions = nuclide.reactions;
        self.available_temperatures = nuclide.available_temperatures;
        self.loaded_temperatures = nuclide.loaded_temperatures;
    self.data_path = nuclide.data_path;
    Ok(())
    }

    #[getter]
    pub fn reactions(&self, py: Python) -> PyResult<PyObject> {
        let py_dict = PyDict::new(py);
        
        // Create a dictionary of temperature -> mt -> reaction
        for (temp, mt_map) in &self.reactions {
            let mt_dict = PyDict::new(py);
            for (mt, reaction) in mt_map {
                let reaction_dict = PyDict::new(py);
                reaction_dict.set_item("cross_section", &reaction.cross_section)?;
                reaction_dict.set_item("threshold_idx", reaction.threshold_idx)?;
                reaction_dict.set_item("interpolation", &reaction.interpolation)?;
                if !reaction.energy.is_empty() {
                    reaction_dict.set_item("energy", &reaction.energy)?;
                }
                mt_dict.set_item(mt, reaction_dict)?;
            }
            py_dict.set_item(temp, mt_dict)?;
        }
        
        Ok(py_dict.into())
    }

    #[getter]
    pub fn reaction_mts(&self) -> Option<Vec<i32>> {
        Nuclide::from(self.clone()).reaction_mts()
    }

    #[getter]
    pub fn energy(&self, py: Python) -> PyResult<Option<PyObject>> {
        if let Some(energy_map) = &self.energy {
            let py_dict = PyDict::new(py);
            for (temp_key, energy_grid) in energy_map.iter() {
                py_dict.set_item(temp_key, energy_grid)?;
            }
            Ok(Some(py_dict.into()))
        } else {
            Ok(None)
        }
    }
    
    // Get the energy grid for a specific temperature
    pub fn energy_grid(&self, temperature: &str) -> Option<Vec<f64>> {
        let nuclide = Nuclide::from(self.clone());
        nuclide.energy_grid(temperature).cloned()
    }
    
    // Get the reaction energy grid for a specific reaction
    pub fn get_reaction_energy_grid(&self, temperature: &str, mt: i32) -> Option<Vec<f64>> {
        if let Some(temp_reactions) = self.reactions.get(temperature) {
            if let Some(reaction) = temp_reactions.get(&mt) {
                if !reaction.energy.is_empty() {
                    return Some(reaction.energy.clone());
                }
            }
        }
        None
    }
}

#[cfg(feature = "pyo3")]
impl From<Nuclide> for PyNuclide {
    fn from(n: Nuclide) -> Self {
        PyNuclide {
            name: n.name,
            element: n.element,
            atomic_symbol: n.atomic_symbol,
            atomic_number: n.atomic_number,
            neutron_number: n.neutron_number,
            mass_number: n.mass_number,
            library: n.library,
            energy: n.energy,
            reactions: n.reactions,
            fissionable: n.fissionable,
            available_temperatures: n.available_temperatures,
            loaded_temperatures: n.loaded_temperatures,
            data_path: n.data_path,
        }
    }
}

impl From<PyNuclide> for Nuclide {
    fn from(py: PyNuclide) -> Self {
        Nuclide {
            name: py.name,
            element: py.element,
            atomic_symbol: py.atomic_symbol,
            atomic_number: py.atomic_number,
            neutron_number: py.neutron_number,
            mass_number: py.mass_number,
            library: py.library,
            energy: py.energy,
            reactions: py.reactions,
            fissionable: py.fissionable,
            available_temperatures: py.available_temperatures,
            loaded_temperatures: py.loaded_temperatures,
            data_path: py.data_path,
        }
    }
}

#[cfg(feature = "pyo3")]
#[pyfunction]
pub fn py_read_nuclide_from_json(path: &str) -> PyResult<PyNuclide> {
    let nuclide = crate::nuclide::read_nuclide_from_json(path, None)
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
#[pyfunction]
pub fn clear_nuclide_cache() {
    crate::nuclide::clear_nuclide_cache();
}
