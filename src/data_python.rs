use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::data::NATURAL_ABUNDANCE;

#[pyfunction]
pub fn natural_abundance(py: Python) -> PyObject {
    let dict = PyDict::new(py);
    for (k, v) in NATURAL_ABUNDANCE.iter() {
        dict.set_item(*k, v).unwrap();
    }
    dict.into()
}
