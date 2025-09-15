use crate::material::UniversalInput;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

/// Convert Python input to UniversalInput enum - pure type conversion, no logic
/// Shared helper function for all Python wrappers
pub fn convert_python_input_to_universal(
    py_input: Option<&pyo3::types::PyAny>,
) -> PyResult<UniversalInput> {
    match py_input {
        Some(obj) => {
            if obj.is_instance_of::<PyDict>() {
                let d: &PyDict = obj.downcast::<PyDict>()?;
                let mut rust_map = HashMap::new();
                for (k, v) in d.iter() {
                    rust_map.insert(k.extract::<String>()?, v.extract::<String>()?);
                }
                Ok(UniversalInput::Map(rust_map))
            } else if obj.is_instance_of::<pyo3::types::PyString>() {
                let keyword: String = obj.extract()?;
                Ok(UniversalInput::Keyword(keyword))
            } else {
                Err(pyo3::exceptions::PyTypeError::new_err(
                    "nuclide_json_map must be a dict or a str keyword"
                ))
            }
        }
        None => Ok(UniversalInput::None),
    }
}
