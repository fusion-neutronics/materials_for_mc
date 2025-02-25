use pyo3::prelude::*;

mod material;
mod python;

#[pymodule]
fn material_for_mc(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<python::PyMaterial>()?;
    Ok(())
}