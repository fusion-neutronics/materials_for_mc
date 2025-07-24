use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::data::{NATURAL_ABUNDANCE, ELEMENT_NUCLIDES};

#[pyfunction]
pub fn natural_abundance(py: Python) -> PyObject {
    let dict = PyDict::new(py);
    for (k, v) in NATURAL_ABUNDANCE.iter() {
        dict.set_item(*k, v).unwrap();
    }
    dict.into()
}

#[pyfunction]
pub fn element_nuclides(py: Python) -> PyObject {
    let dict = PyDict::new(py);
    for (element, nuclides) in ELEMENT_NUCLIDES.iter() {
        let mut sorted_nuclides = nuclides.clone();
        sorted_nuclides.sort();
        dict.set_item(*element, sorted_nuclides).unwrap();
    }
    dict.into()
}

