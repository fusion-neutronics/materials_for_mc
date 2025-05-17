use pyo3::prelude::*;
// use pyo3::types::PyDict;
use crate::material::Material;
use pyo3::exceptions::PyValueError;

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
            result.push_str(&format!(
                "  Density: {} {}\n",
                density, self.internal.density_unit
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
    fn density_unit(&self) -> String {
        self.internal.density_unit.clone()
    }
}


// Add this module export function - this is what's missing
#[pymodule]
fn materials_for_mc(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyMaterial>()?;
    Ok(())
}