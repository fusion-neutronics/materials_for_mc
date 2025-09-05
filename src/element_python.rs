// Python bindings for the element module
// Implement PyO3 wrappers here if needed

use pyo3::prelude::*;
use crate::element::Element;

#[pyclass(name = "Element")]
/// Chemical element container providing isotope helper methods.
///
/// Args:
///     name (str): Element symbol (e.g. "Fe", "U", "H").
///
/// Attributes:
///     name (str): Element symbol provided at construction.
pub struct PyElement {
	pub inner: Element,
}

#[pymethods]
impl PyElement {
	#[new]
	#[pyo3(text_signature = "(name)")]
	fn new(name: String) -> Self { Self { inner: Element::new(name) } }

	/// Element symbol.
	#[getter]
	fn name(&self) -> String { self.inner.name.clone() }

	/// Return list of isotope identifiers for this element (e.g. ["Fe54", ...]).
	///
	/// Returns:
	///     List[str]: Isotope names sorted by mass number.
	fn get_nuclides(&self) -> Vec<String> { self.inner.get_nuclides() }
}
