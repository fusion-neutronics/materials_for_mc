use pyo3::prelude::*;
use pyo3::exceptions::PyIndexError;
use crate::materials::Materials;
use crate::material_python::PyMaterial;
// use pyo3::prelude::*;
use crate::material::Material; 

/// Python wrapper for the Rust Materials struct
#[pyclass(name = "Materials")]
pub struct PyMaterials {
    /// Internal Rust Materials instance
    internal: Materials,
}

#[pymethods]
impl PyMaterials {
    /// Create a new empty materials collection
    #[new]
    fn new() -> Self {
        PyMaterials {
            internal: Materials::new(),
        }
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
            None => Err(PyIndexError::new_err(format!("Index {} out of range", index)))
        }
    }
    
    /// Remove a material at the specified index
    fn remove(&mut self, index: usize) -> PyResult<PyMaterial> {
        if index < self.internal.len() {
            Ok(PyMaterial::from_material(self.internal.remove(index)))
        } else {
            Err(PyIndexError::new_err(format!("Index {} out of range", index)))
        }
    }
    
    /// Get the number of materials in the collection
    fn len(&self) -> usize {
        self.internal.len()
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
}
