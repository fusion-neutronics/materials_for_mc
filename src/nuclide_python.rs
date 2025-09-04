#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::types::PyDict;
use crate::nuclide::Nuclide;
use crate::reaction::Reaction;
use std::collections::HashMap;

#[cfg(feature = "pyo3")]
/// Nuclide data container exposed to Python.
///
/// Create a new (optionally named) nuclide instance.
///
/// Args:
///     name (Optional[str]): Optional nuclide identifier (e.g. "Li6", "Fe56"). If not
///         supplied you must pass `path` to `read_nuclide_from_json` later.
///
/// Attributes:
///     name (Optional[str]): Identifier like "Li6" or "Fe56".
///     element (Optional[str]): Element symbol, e.g. "Fe".
///     atomic_symbol (Optional[str]): Full atomic symbol (same as element for now).
///     atomic_number (Optional[int]): Z (protons).
///     neutron_number (Optional[int]): N (neutrons).
///     mass_number (Optional[int]): A (Z+N).
///     library (Optional[str]): Source nuclear data library identifier.
///     energy (Optional[Dict[str, List[float]]]): Temperature -> energy grid(s).
///     reactions (Dict[str, Dict[int, Reaction]]): Temperature -> MT -> Reaction data.
///     fissionable (bool): True if nuclide is fissionable.
///     available_temperatures (List[str]): All temperatures present in file.
///     loaded_temperatures (List[str]): Subset actually loaded.
///     data_path (Optional[str]): Path to data file used.
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
    /// Create a new (optionally named) nuclide.
    ///
    /// Args:
    ///     name (Optional[str]): Optional nuclide identifier (e.g. "Li6", "Fe56"). If not
    ///         supplied you must pass `path` to `read_nuclide_from_json` later.
    ///
    /// Returns:
    ///     Nuclide: A nuclide object with no data loaded yet.
    #[new]
    #[pyo3(text_signature = "(name=None)")]
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

    /// Load nuclear data from a JSON file.
    ///
    /// You can either provide a JSON file path explicitly via `path` or rely on
    /// the `name` given at construction if your application resolves it.
    ///
    /// When `temperatures` is provided only those temperatures are loaded while
    /// `available_temperatures` always lists every temperature present in the
    /// file. The subset actually loaded is stored in `loaded_temperatures`.
    ///
    /// Args:
    ///     path (Optional[str]): Optional path to the nuclide JSON file. If omitted the
    ///         constructor `name` must have been set and is used as the path / key.
    ///     temperatures (Optional[List[str]]): Temperature strings (e.g. ["293K"]).
    ///         If given only these temperatures are loaded.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     ValueError: If neither `path` nor `name` is available, or if the
    ///         JSON cannot be read / parsed.
    #[pyo3(signature = (path=None, temperatures=None), text_signature = "(self, path=None, temperatures=None)")]
    pub fn read_nuclide_from_json(&mut self, path: Option<String>, temperatures: Option<Vec<String>>) -> PyResult<()> {
        use std::collections::HashSet;
        let identifier = if let Some(p) = &path { p.as_str() } else {
            self.name.as_deref().ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Nuclide name not set and no path provided"))?
        };
        let temps_set: Option<HashSet<String>> = temperatures.map(|v| v.into_iter().collect());
        
        // First load without temperature filtering to get all available temperatures
        let full_nuclide = crate::nuclide::read_nuclide_from_json(identifier, None)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        // Now load with temperature filtering if specified
        let nuclide = if temps_set.is_some() {
            crate::nuclide::read_nuclide_from_json(identifier, temps_set.as_ref())
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?
        } else {
            full_nuclide.clone()
        };

        self.name = nuclide.name;
        self.element = nuclide.element;
        self.atomic_symbol = nuclide.atomic_symbol;
        self.atomic_number = nuclide.atomic_number;
        self.neutron_number = nuclide.neutron_number;
        self.mass_number = nuclide.mass_number;
        self.library = nuclide.library;
        self.energy = nuclide.energy;
        self.reactions = nuclide.reactions;
        // Always preserve full available_temperatures from the unfiltered load
        self.available_temperatures = full_nuclide.available_temperatures;
        self.loaded_temperatures = nuclide.loaded_temperatures;
        self.data_path = nuclide.data_path;
        Ok(())
    }

    /// Mapping of temperature -> MT number -> reaction data.
    ///
    /// Returns:
    ///     Dict[str, Dict[int, Dict[str, Any]]]: Nested dictionary. The innermost
    ///     dictionary has these keys:
    ///
    ///         - cross_section (List[float])
    ///         - threshold_idx (int)
    ///         - interpolation (List[int])
    ///         - energy (Optional[List[float]]): Present when reaction has its own grid
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

    /// List of MT numbers available for the (first) loaded temperature.
    ///
    /// Returns:
    ///     Optional[List[int]]: List of MT identifiers or None if no data.
    #[getter]
    pub fn reaction_mts(&self) -> Option<Vec<i32>> {
        Nuclide::from(self.clone()).reaction_mts()
    }

    /// Energy grids by temperature.
    ///
    /// Returns:
    ///     Optional[Dict[str, List[float]]]: Map of temperature key to energy grid
    ///     or None if no energy data loaded.
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
    
    /// Get the energy grid for a specific temperature.
    ///
    /// Args:
    ///     temperature (str): Temperature key (e.g. "293K").
    ///
    /// Returns:
    ///     Optional[List[float]]: The energy grid or None if not present.
    pub fn energy_grid(&self, temperature: &str) -> Option<Vec<f64>> {
        let nuclide = Nuclide::from(self.clone());
        nuclide.energy_grid(temperature).cloned()
    }
    
    /// Get the reaction-specific energy grid for a given MT at a temperature.
    ///
    /// Args:
    ///     temperature (str): Temperature key (e.g. "293K").
    ///     mt (int): ENDF MT reaction number.
    ///
    /// Returns:
    ///     Optional[List[float]]: Reaction energy grid if present.
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
