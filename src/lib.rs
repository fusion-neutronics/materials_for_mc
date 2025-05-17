use pyo3::prelude::*;

mod material;
mod python;

#[pymodule]
fn materials_for_mc(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<python::PyMaterial>()?;
    Ok(())
}
