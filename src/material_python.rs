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

    // // Add a regular method for get_nuclides
    // fn get_nuclides(&self) -> Vec<String> {
    //     self.internal.get_nuclides()
    // }

    // fn get_nuclide_fraction(&self, nuclide: String) -> Option<f64> {
    //     self.internal.get_nuclide_fraction(&nuclide)
    // }

    // fn get_total_fraction(&self) -> f64 {
    //     self.internal.get_total_fraction()
    // }

    // fn normalize(&mut self) -> PyResult<()> {
    //     self.internal.normalize()
    //         .map_err(|e| PyValueError::new_err(e))
    // }

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