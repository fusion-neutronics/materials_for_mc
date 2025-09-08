// Python bindings for the element module
// Implement PyO3 wrappers here if needed

use crate::element::Element;
use pyo3::prelude::*;

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
    /// Create a new Element.
    ///
    /// Args:
    ///     name (str): Element symbol (e.g. "Fe", "U", "H").
    fn new(name: String) -> Self {
        Self {
            inner: Element::new(name),
        }
    }

    /// Element symbol.
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Return list of isotope (nuclide) identifiers for this element.
    ///
    /// Returns:
    ///     List[str]: Isotope names sorted by mass number (e.g. ["Fe54", "Fe56"]).
    #[pyo3(text_signature = "(self)")]
    fn get_nuclides(&self) -> Vec<String> {
        self.inner.get_nuclides()
    }
}
