use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::exceptions::PyValueError;
use crate::material::Material;

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
        self.internal.add_nuclide(&nuclide, fraction)
            .map_err(|e| PyValueError::new_err(e))
    }

    fn set_density(&mut self, unit: String, value: f64) -> PyResult<()> {
        self.internal.set_density(&unit, value)
            .map_err(|e| PyValueError::new_err(e))
    }

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

    /// Get all nuclides and their fractions as a dictionary
    // fn get_nuclides(&self, py: Python) -> PyResult<PyObject> {
    //     let dict = PyDict::new(py);
    //     for (nuclide, fraction) in &self.internal.nuclides {
    //         dict.set_item(nuclide, *fraction)?;
    //     }
    //     Ok(dict.into())
    // }

    /// String representation of the Material
    fn __str__(&self) -> PyResult<String> {
        let mut result = String::from("Material:\n");
        
        // Add density information
        if let Some(density) = self.internal.density {
            result.push_str(&format!("  Density: {} {}\n", density, self.internal.density_unit));
        } else {
            result.push_str("  Density: not set\n");
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
    fn density_unit(&self) -> String {
        self.internal.density_unit.clone()
    }
}