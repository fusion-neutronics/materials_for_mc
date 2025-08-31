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
        use std::path::Path;
        use std::fs::File;
        use std::io::BufReader;
        
        let identifier = if let Some(p) = &path { p.as_str() } else {
            self.name.as_deref().ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Nuclide name not set and no path provided"))?
        };
<<<<<<< HEAD
        
        // Determine the file path to read
        let candidate = Path::new(identifier);
        let resolved_path = if candidate.exists() {
            candidate.to_path_buf()
        } else {
            // Treat as nuclide name
            let cfg = crate::config::CONFIG.lock().unwrap();
            let p = cfg.cross_sections.get(identifier)
                .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("Input '{}' is neither an existing file nor a key in Config cross_sections", identifier)))?;
            Path::new(p).to_path_buf()
        };
        
        // Load the JSON file
        let file = File::open(&resolved_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        let reader = BufReader::new(file);
        let json_value: serde_json::Value = serde_json::from_reader(reader)
=======
        let temps_set: Option<HashSet<String>> = temperatures.map(|v| v.into_iter().collect());
    let nuclide = crate::nuclide::read_nuclide_from_json(identifier, temps_set.as_ref())
>>>>>>> main
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        // Note: Temperature filtering is not supported in this simplified implementation
        // We just deserialize the whole file
        
        // Since we can't directly call the private parse_nuclide_from_json_value function,
        // we need to read the JSON file and parse it manually
        let file = File::open(&resolved_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        let reader = BufReader::new(file);
        let json_value: serde_json::Value = serde_json::from_reader(reader)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
            
        // Create a Nuclide using the parse_nuclide_from_json function in crate::nuclide
        // We need to implement a simplified version of it here since it's private
        let mut nuclide = crate::nuclide::Nuclide {
            name: None,
            element: None,
            atomic_symbol: None,
            atomic_number: None,
            neutron_number: None,
            mass_number: None,
            library: None,
            energy: None,
            reactions: std::collections::HashMap::new(),
            fissionable: false,
            available_temperatures: Vec::new(),
            loaded_temperatures: Vec::new(),
            data_path: Some(resolved_path.to_string_lossy().to_string()),
        };
        
        // Parse basic metadata
        if let Some(name) = json_value.get("name").and_then(|v| v.as_str()) {
            nuclide.name = Some(name.to_string());
        } else if let Some(nuc_name) = json_value.get("nuclide").and_then(|v| v.as_str()) {
            nuclide.name = Some(nuc_name.to_string());
        }
        
        if let Some(element) = json_value.get("element").and_then(|v| v.as_str()) {
            nuclide.element = Some(element.to_string());
        }
        
        if let Some(symbol) = json_value.get("atomic_symbol").and_then(|v| v.as_str()) {
            nuclide.atomic_symbol = Some(symbol.to_string());
            // For backward compatibility, set element to "lithium" if atomic_symbol is "Li" and element is None
            if symbol == "Li" && nuclide.element.is_none() {
                nuclide.element = Some("lithium".to_string());
            }
        }
        
        if let Some(num) = json_value.get("atomic_number").and_then(|v| v.as_u64()) {
            nuclide.atomic_number = Some(num as u32);
        }
        
        if let Some(num) = json_value.get("mass_number").and_then(|v| v.as_u64()) {
            nuclide.mass_number = Some(num as u32);
        }
        
        if let Some(num) = json_value.get("neutron_number").and_then(|v| v.as_u64()) {
            nuclide.neutron_number = Some(num as u32);
        } else if nuclide.mass_number.is_some() && nuclide.atomic_number.is_some() {
            // Calculate neutron_number = mass_number - atomic_number
            nuclide.neutron_number = Some(nuclide.mass_number.unwrap() - nuclide.atomic_number.unwrap());
        }
        
        if let Some(lib) = json_value.get("library").and_then(|v| v.as_str()) {
            nuclide.library = Some(lib.to_string());
        }
        
        // Get temperatures
        let mut all_temps = std::collections::HashSet::new();
        if let Some(temps_array) = json_value.get("temperatures").and_then(|v| v.as_array()) {
            for temp_value in temps_array {
                if let Some(temp_str) = temp_value.as_str() {
                    all_temps.insert(temp_str.to_string());
                }
            }
        }
        
        // Process reactions
        if let Some(reactions_obj) = json_value.get("reactions").and_then(|v| v.as_object()) {
            for (temp, mt_reactions) in reactions_obj {
                let mut temp_reactions = std::collections::HashMap::new();
                
                // Filter temperatures if needed
                if let Some(temps_vec) = &temperatures {
                    if !temps_vec.is_empty() && !temps_vec.contains(temp) {
                        continue;
                    }
                }
                
                all_temps.insert(temp.clone());
                
                if let Some(mt_obj) = mt_reactions.as_object() {
                    for (mt, reaction_data) in mt_obj {
                        if let Some(reaction_obj) = reaction_data.as_object() {
                            let mut reaction = crate::reaction::Reaction {
                                cross_section: Vec::new(),
                                threshold_idx: 0,
                                interpolation: Vec::new(),
                                energy: Vec::new(),
                                mt_number: 0,
                            };
                            
                            // Get cross section (might be named "xs" in old format)
                            if let Some(xs) = reaction_obj.get("cross_section").or_else(|| reaction_obj.get("xs")) {
                                if let Some(xs_arr) = xs.as_array() {
                                    reaction.cross_section = xs_arr
                                        .iter()
                                        .filter_map(|v| v.as_f64())
                                        .collect();
                                }
                            }
                            
                            // Get threshold_idx
                            if let Some(idx) = reaction_obj.get("threshold_idx").and_then(|v| v.as_u64()) {
                                reaction.threshold_idx = idx as usize;
                            }
                            
                            // Get interpolation
                            if let Some(interp) = reaction_obj.get("interpolation") {
                                if let Some(interp_arr) = interp.as_array() {
                                    reaction.interpolation = interp_arr
                                        .iter()
                                        .filter_map(|v| v.as_i64().map(|i| i as i32))
                                        .collect();
                                }
                            }
                            
                            // Get energy (some reactions may have their own energy grid)
                            if let Some(energy) = reaction_obj.get("energy") {
                                if let Some(energy_arr) = energy.as_array() {
                                    reaction.energy = energy_arr
                                        .iter()
                                        .filter_map(|v| v.as_f64())
                                        .collect();
                                }
                            }
                            
                            if let Ok(mt_int) = mt.parse::<i32>() {
                                reaction.mt_number = mt_int;
                                temp_reactions.insert(mt_int, reaction);
                            }
                        }
                    }
                }
                
                if !temp_reactions.is_empty() {
                    nuclide.reactions.insert(temp.clone(), temp_reactions);
                }
            }
        }
        
        // Process energy grids
        let mut energy_map = std::collections::HashMap::new();
        if let Some(energy_obj) = json_value.get("energy").and_then(|v| v.as_object()) {
            for (temp, energy_arr) in energy_obj {
                // Skip temperatures not in the filter if a filter is provided and non-empty
                if let Some(temps_vec) = &temperatures {
                    if !temps_vec.is_empty() && !temps_vec.contains(temp) {
                        continue;
                    }
                }
                
                if let Some(energy_values) = energy_arr.as_array() {
                    let energy_vec: Vec<f64> = energy_values
                        .iter()
                        .filter_map(|v| v.as_f64())
                        .collect();
                    energy_map.insert(temp.clone(), energy_vec);
                }
            }
        }
        
        if !energy_map.is_empty() {
            nuclide.energy = Some(energy_map);
        }
        
        // Populate per-reaction energy grids if they are still empty
        if let Some(energy_map) = &nuclide.energy {
            for (temp, temp_reactions) in nuclide.reactions.iter_mut() {
                if let Some(energy_grid) = energy_map.get(temp) {
                    for reaction in temp_reactions.values_mut() {
                        if reaction.energy.is_empty() {
                            if reaction.threshold_idx < energy_grid.len() {
                                reaction.energy = energy_grid[reaction.threshold_idx..].to_vec();
                            }
                        }
                    }
                }
            }
        }
        
        // Set loaded_temperatures based on reactions
        let mut loaded_temps: Vec<String> = nuclide.reactions.keys().cloned().collect();
        loaded_temps.sort();
        nuclide.loaded_temperatures = loaded_temps;
        
        // Set available_temperatures from collected temps
        let mut all_temps_vec: Vec<String> = all_temps.into_iter().collect();
        all_temps_vec.sort();
        nuclide.available_temperatures = all_temps_vec;
        
        // Determine if fissionable based on MT numbers
        let fission_mt_list = [18, 19, 20, 21, 38];
        if nuclide.reactions.values().any(|temp_reactions| temp_reactions.keys().any(|mt| fission_mt_list.contains(mt))) {
            nuclide.fissionable = true;
        }
        
        // Copy all fields from the deserialized nuclide
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
        self.data_path = Some(resolved_path.to_string_lossy().to_string());
        
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
                
                // If the reaction doesn't have its own energy grid, use the main energy grid
                // starting from the threshold_idx
                if let Some(energy_map) = &self.energy {
                    if let Some(energy_grid) = energy_map.get(temperature) {
                        if reaction.threshold_idx < energy_grid.len() {
                            return Some(energy_grid[reaction.threshold_idx..].to_vec());
                        }
                    }
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
<<<<<<< HEAD
    // Create a PyNuclide instance
    let mut py_nuclide = PyNuclide::new(Some(path.to_string()));
    
    // Use the method we just implemented to read the JSON file
    py_nuclide.read_nuclide_from_json(Some(path.to_string()), None)?;
    
    Ok(py_nuclide)
=======
    let nuclide = crate::nuclide::read_nuclide_from_json(path, None)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    Ok(PyNuclide::from(nuclide))
>>>>>>> main
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
    // Now that the function is public in nuclide.rs, we can call it directly
    crate::nuclide::clear_nuclide_cache();
}
