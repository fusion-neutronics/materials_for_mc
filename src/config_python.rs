use crate::config::{Config, CONFIG};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType, PyString};
use std::collections::HashMap;

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
        let config = CONFIG.lock().unwrap_or_else(|poisoned| {
            poisoned.into_inner()
        });
        let dict = PyDict::new(py);

        for (nuclide, path) in &config.cross_sections {
            dict.set_item(nuclide, path)?;
        }

        Ok(dict.into())
    }

    /// Set the cross sections dictionary or global keyword
    #[classmethod]
    #[pyo3(text_signature = "(cls, value)")]
    fn set_cross_sections(cls: &PyType, value: &PyAny) -> PyResult<()> {
        let mut config = CONFIG.lock().unwrap_or_else(|poisoned| {
            poisoned.into_inner()
        });
        
        if let Ok(dict) = value.downcast::<PyDict>() {
            // Handle dictionary input
            let mut rust_map = HashMap::new();
            for (k, v) in dict.iter() {
                let key: String = k.extract()?;
                let val: String = v.extract()?;
                rust_map.insert(key, val);
            }
            config.set_cross_sections(rust_map);
        } else if let Ok(string_val) = value.extract::<String>() {
            // Handle string input (global keyword)
            config.set_cross_sections(string_val);
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "value must be either a dictionary or a string"
            ));
        }
        
        Ok(())
    }

    /// Set a cross section file path for a nuclide, or set a global default if only a keyword is provided
    #[classmethod]
    #[pyo3(text_signature = "(cls, keyword_or_nuclide, path=None)")]
    fn set_cross_section(_cls: &PyType, keyword_or_nuclide: &str, path: Option<&str>) -> PyResult<()> {
        let mut config = CONFIG.lock().unwrap_or_else(|poisoned| {
            poisoned.into_inner()
        });
        config.set_cross_section(keyword_or_nuclide, path);
        Ok(())
    }

    /// Get a cross section file path for a nuclide
    #[classmethod]
    #[pyo3(text_signature = "(cls, nuclide)")]
    fn get_cross_section(_cls: &PyType, nuclide: &str) -> Option<String> {
        let config = CONFIG.lock().unwrap_or_else(|poisoned| {
            poisoned.into_inner()
        });
        config.get_cross_section(nuclide)
    }
}
