use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use std::collections::HashMap;
use crate::config::{Config, CONFIG};

/// Python wrapper for the Config struct
#[pyclass(name = "Config")]
pub struct PyConfig;

#[pymethods]
impl PyConfig {
    #[new]
    /// Create a Config sentinel instance (methods are classmethods).
    fn new() -> Self {
        PyConfig
    }

    /// Get the cross sections dictionary
    #[classmethod]
    #[pyo3(text_signature = "(cls)")]
    fn get_cross_sections(cls: &PyType, py: Python<'_>) -> PyResult<PyObject> {
        let config = CONFIG.lock().unwrap();
        let dict = PyDict::new(py);
        
        for (nuclide, path) in &config.cross_sections {
            dict.set_item(nuclide, path)?;
        }
        
        Ok(dict.into())
    }

    /// Set the cross sections dictionary
    #[classmethod]
    #[pyo3(text_signature = "(cls, value)")]
    fn set_cross_sections(cls: &PyType, value: &PyDict) -> PyResult<()> {
        let mut rust_map = HashMap::new();
        for (k, v) in value.iter() {
            let key: String = k.extract()?;
            let val: String = v.extract()?;
            rust_map.insert(key, val);
        }
        
        let mut config = CONFIG.lock().unwrap();
        config.set_cross_sections(rust_map);
        Ok(())
    }

    /// Set a cross section file path for a nuclide
    #[classmethod]
    #[pyo3(text_signature = "(cls, nuclide, path)")]
    fn set_cross_section(_cls: &PyType, nuclide: &str, path: &str) -> PyResult<()> {
        let mut config = CONFIG.lock().unwrap();
        config.set_cross_section(nuclide, path);
        Ok(())
    }

    /// Get a cross section file path for a nuclide
    #[classmethod]
    #[pyo3(text_signature = "(cls, nuclide)")]
    fn get_cross_section(_cls: &PyType, nuclide: &str) -> Option<String> {
        let config = CONFIG.lock().unwrap();
        config.get_cross_section(nuclide)
    }
}
