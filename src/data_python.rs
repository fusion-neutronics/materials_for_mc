use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::data::{NATURAL_ABUNDANCE, ELEMENT_NUCLIDES, SUM_RULES, ELEMENT_NAMES, ATOMIC_MASSES, get_all_mt_descendants};

#[pyfunction]
pub fn sum_rules(py: Python) -> PyObject {
    let dict = PyDict::new(py);
    for (k, v) in SUM_RULES.iter() {
        dict.set_item(*k, v.clone()).unwrap();
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

#[pyfunction]
pub fn natural_abundance(py: Python) -> PyObject {
    let dict = PyDict::new(py);
    for (k, v) in NATURAL_ABUNDANCE.iter() {
        dict.set_item(*k, v).unwrap();
    }
    dict.into()
}

#[pyfunction]
pub fn element_names(py: Python) -> PyObject {
    let dict = PyDict::new(py);
    for (symbol, name) in ELEMENT_NAMES.iter() {
        dict.set_item(*symbol, *name).unwrap();
    }
    dict.into()
}

#[pyfunction]
pub fn atomic_masses(py: Python) -> PyObject {
    let dict = PyDict::new(py);
    for (nuclide, mass) in ATOMIC_MASSES.iter() {
        dict.set_item(*nuclide, *mass).unwrap();
    }
    dict.into()
}

#[pyfunction]
pub fn py_get_all_mt_descendants(mt_num: i32) -> Vec<i32> {
    get_all_mt_descendants(mt_num)
}
#[pymodule]
fn data_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_get_all_mt_descendants, m)?)?;
    // ...existing code...
    Ok(())
}
