use crate::material_python::PyMaterial;
use crate::materials::Materials;
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use pyo3::types::PyList;

/// Python wrapper for the Rust Materials struct
#[pyclass(name = "Materials")]
pub struct PyMaterials {
    /// Internal Rust Materials instance
    internal: Materials,
}

#[pymethods]
impl PyMaterials {
    /// Create a new materials collection, optionally with initial materials
    #[new]
    fn py_new(materials: Option<&PyList>) -> PyResult<Self> {
        let mut result = PyMaterials {
            internal: Materials::new(),
        };

        // If materials were provided, add them to the collection
        if let Some(mat_list) = materials {
            for item in mat_list.iter() {
                // Extract PyMaterial by value, not by reference
                let material = item.extract::<PyRef<PyMaterial>>()?;
                result.internal.append(material.get_internal().clone());
            }
        }

        Ok(result)
    }

    /// Append a material to the collection
    fn append(&mut self, material: &PyMaterial) -> PyResult<()> {
        // Use get_internal() method instead of directly accessing the field
        self.internal.append(material.get_internal().clone());
        Ok(())
    }

    /// Get a material by index
    fn get(&self, index: usize) -> PyResult<PyMaterial> {
        match self.internal.get(index) {
            Some(m) => Ok(PyMaterial::from_material(m.clone())),
            None => Err(PyIndexError::new_err(format!(
                "Index {} out of range",
                index
            ))),
        }
    }

    /// Remove a material at the specified index
    fn remove(&mut self, index: usize) -> PyResult<PyMaterial> {
        if index < self.internal.len() {
            Ok(PyMaterial::from_material(self.internal.remove(index)))
        } else {
            Err(PyIndexError::new_err(format!(
                "Index {} out of range",
                index
            )))
        }
    }

    /// Get the number of materials in the collection
    fn len(&self) -> usize {
        self.internal.len()
    }

    /// Special method for Python's len() function
    fn __len__(&self) -> usize {
        self.len()
    }

    /// Check if the collection is empty
    fn is_empty(&self) -> bool {
        self.internal.is_empty()
    }

    /// Return a string representation of the Materials object
    fn __repr__(&self) -> String {
        format!("Materials with {} entries", self.internal.len())
    }

    /// Make the Materials object behave like a sequence in Python
    fn __getitem__(&self, index: usize) -> PyResult<PyMaterial> {
        self.get(index)
    }

    /// Read nuclides from a JSON-like Python dictionary or a string keyword (delegates to Materials::read_nuclides)
    #[pyo3(name = "read_nuclides_from_json")]
    fn read_nuclides_from_json(
        &mut self,
        _py: Python,
        nuclide_json_map: Option<&pyo3::types::PyAny>,
    ) -> PyResult<()> {
        // Extract Python data to Rust types  
        let (dict_data, keyword_data) = if let Some(obj) = nuclide_json_map {
            if obj.is_instance_of::<pyo3::types::PyDict>() {
                let d: &pyo3::types::PyDict = obj.downcast::<pyo3::types::PyDict>()?;
                let mut rust_map = std::collections::HashMap::new();
                for (k, v) in d.iter() {
                    rust_map.insert(k.extract::<String>()?, v.extract::<String>()?);
                }
                (Some(rust_map), None)
            } else if obj.is_instance_of::<pyo3::types::PyString>() {
                let keyword: String = obj.extract()?;
                (None, Some(keyword))
            } else {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "nuclide_json_map must be a dict or a str keyword"
                ));
            }
        } else {
            (None, None)
        };
        
        // Call pure Rust function
        self.internal.load_nuclear_data_from_input(dict_data, keyword_data)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}
