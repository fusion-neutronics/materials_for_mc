use pyo3::prelude::*;
// use pyo3::types::PyDict;
use crate::material::Material;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyDict;
use std::collections::HashMap;

#[pyclass(name = "Material")]
pub struct PyMaterial {
    internal: Material,
}

#[pymethods]
impl PyMaterial {
    #[new]
    fn new() -> Self {
        PyMaterial {
            internal: Material::new(),
        }
    }

    fn add_nuclide(&mut self, nuclide: String, fraction: f64) -> PyResult<()> {
        self.internal
            .add_nuclide(&nuclide, fraction)
            .map_err(|e| PyValueError::new_err(e))
    }

    fn set_density(&mut self, unit: String, value: f64) -> PyResult<()> {
        self.internal
            .set_density(&unit, value)
            .map_err(|e| PyValueError::new_err(e))
    }

    /// Get the material nuclides as a tuple of (name, fraction) pairs
    #[getter]
    fn nuclides(&self) -> Vec<(String, f64)> {
        // Convert HashMap to a Vec of tuples
        let mut nuclide_vec: Vec<(String, f64)> = self.internal.nuclides
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        
        // Sort by nuclide name for consistent order
        nuclide_vec.sort_by(|a, b| a.0.cmp(&b.0));
        
        nuclide_vec
    }

    #[getter]
    fn volume(&self) -> Option<f64> {
        self.internal.volume
    }

    #[setter]
    fn set_volume(&mut self, value: f64) -> PyResult<()> {
        self.internal
            .volume(Some(value))
            .map_err(|e| PyValueError::new_err(e))?;
        Ok(())
    }
    // Then try this:
    fn get_nuclide_names(&self) -> Vec<String> {
        self.internal.get_nuclides()
    }

    /// String representation of the Material
    fn __str__(&self) -> PyResult<String> {
        let mut result = String::from("Material:\n");

        // Add density information
        if let Some(density) = self.internal.density {
            result.push_str(&format!(
                "  Density: {} {}\n",
                density, self.internal.density_units
            ));
        } else {
            result.push_str("  Density: not set\n");
        }

        // Add volume information
        if let Some(volume) = self.internal.volume {
            result.push_str(&format!("  Volume: {} cm³\n", volume));
        } else {
            result.push_str("  Volume: not set\n");
        }

        // Add nuclide information
        result.push_str("  Composition:\n");
        for (nuclide, fraction) in &self.internal.nuclides {
            result.push_str(&format!("    {}: {}\n", nuclide, fraction));
        }

        Ok(result)
    }

    /// Return the same string as __str__
    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    #[getter]
    fn density(&self) -> Option<f64> {
        self.internal.density
    }

    #[getter]
    fn density_units(&self) -> String {
        self.internal.density_units.clone()
    }

    fn read_nuclides_from_json(&mut self, py: Python, nuclide_json_map: &PyDict) -> PyResult<()> {
        let mut rust_map = HashMap::new();
        for (k, v) in nuclide_json_map.iter() {
            let key: String = k.extract()?;
            let val: String = v.extract()?;
            rust_map.insert(key, val);
        }
        self.internal.read_nuclides_from_json(&rust_map)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Return the pointer address of the Arc<Nuclide> for a given nuclide name (for debugging/sharing checks)
    fn nuclide_ptr_addr(&self, nuclide: &str) -> Option<usize> {
        self.internal.nuclide_data.get(nuclide).map(|arc| {
            let ptr: *const crate::nuclide::Nuclide = std::sync::Arc::as_ptr(arc);
            ptr as usize
        })
    }

    #[getter]
    fn temperature(&self) -> String {
        self.internal.temperature.clone()
    }
    
    #[setter]
    fn set_temperature(&mut self, temperature: &str) {
        self.internal.set_temperature(temperature);
    }

    /// Return the unified energy grid for neutrons across all MT reactions
    fn unified_energy_grid_neutron(&mut self) -> Vec<f64> {
        self.internal.unified_energy_grid_neutron()
    }

    /// Calculate microscopic cross sections for neutrons on the unified energy grid
    /// If mt_filter is provided, only those MTs will be included (by string match)
    #[pyo3(signature = (mt_filter=None))]
    fn calculate_microscopic_xs_neutron(
        &mut self,
        mt_filter: Option<Vec<String>>,
    ) -> HashMap<String, HashMap<String, Vec<f64>>> {
        let mt_filter_ref = mt_filter.as_ref();
        self.internal.calculate_microscopic_xs_neutron(mt_filter_ref)
    }

    /// Calculate macroscopic cross sections for neutrons on the unified energy grid
    /// If unified_energy_grid is None or not provided, it will use the cached grid or build a new one
    /// If mt_filter is provided, only those MTs will be included (by string match)
    #[pyo3(signature = (mt_filter=None))]
    fn calculate_macroscopic_xs_neutron(
        &mut self,
        mt_filter: Option<Vec<String>>,
    ) -> (Vec<f64>, HashMap<String, Vec<f64>>) {
        let mt_filter_ref = mt_filter.as_ref();
        let (energy_grid, xs_dict_i32) = self.internal.calculate_macroscopic_xs_neutron(mt_filter_ref);
        let xs_dict = xs_dict_i32.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        (energy_grid, xs_dict)
    }

    /// Calculate the total cross section for neutrons by summing over all relevant MT reactions
    fn calculate_total_xs_neutron(&mut self) -> HashMap<i32, Vec<f64>> {
        self.internal.calculate_total_xs_neutron()
    }

    /// Get the macroscopic cross sections for neutrons
    #[getter]
    fn macroscopic_xs_neutron(&self) -> HashMap<String, Vec<f64>> {
        self.internal.macroscopic_xs_neutron
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    /// Get the atoms per cubic centimeter for each nuclide in the material
    fn get_atoms_per_cc(&self) -> HashMap<String, f64> {
        self.internal.get_atoms_per_cc()
    }

    /// Calculate the neutron mean free path at a given energy
    /// 
    /// This method calculates the mean free path of a neutron at a specific energy
    /// by interpolating the total macroscopic cross section and then taking 1/Σ.
    /// 
    /// If the total macroscopic cross section hasn't been calculated yet, it will
    /// automatically call calculate_total_xs_neutron() first.
    /// 
    /// # Arguments
    /// * `energy` - The energy of the neutron in eV
    /// 
    /// # Returns
    /// * The mean free path in cm, or None if there's no cross section data
    fn mean_free_path_neutron(&mut self, energy: f64) -> Option<f64> {
        self.internal.mean_free_path_neutron(energy)
    }

    fn add_element(&mut self, element: String, fraction: f64) -> PyResult<()> {
        self.internal
            .add_element(&element, fraction)
            .map_err(|e| PyValueError::new_err(e))
    }

    /// Get the sorted list of all unique MT numbers available in this material (across all nuclides)
    #[getter]
    fn reaction_mts(&mut self) -> PyResult<Vec<String>> {
        self.internal
            .reaction_mts()
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}


// Add these helper methods in a separate impl block
impl PyMaterial {
    // Helper method for other Python modules to access the internal Material
    pub(crate) fn get_internal(&self) -> &Material {
        &self.internal
    }
    
    // Helper method to create a PyMaterial from a Material
    pub(crate) fn from_material(material: Material) -> Self {
        PyMaterial { internal: material }
    }
}