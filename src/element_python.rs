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

    /// Atomic number (Z) of the element.
    #[getter]
    fn atomic_number(&self) -> Option<u32> {
        self.inner.atomic_number
    }

    /// Return list of isotope (nuclide) identifiers for this element.
    ///
    /// Returns:
    ///     List[str]: Isotope names sorted by mass number (e.g. ["Fe54", "Fe56"]).
    #[pyo3(text_signature = "(self)")]
    fn get_nuclides(&self) -> Vec<String> {
        self.inner.get_nuclides()
    }

    /// Calculate the natural abundance-weighted microscopic cross section for this element.
    ///
    /// This method loads the cross section data for each naturally occurring isotope of this element,
    /// weights them by their natural abundances, and returns the combined cross section.
    ///
    /// Args:
    ///     reaction (Union[int, str]): Either an MT number (int) or reaction name (str) like "(n,gamma)" or "fission"
    ///     temperature (Optional[str]): Temperature string (e.g. "294", "300K"). Defaults to None.
    ///
    /// Returns:
    ///     Tuple[List[float], List[float]]: A tuple of (energy_grid, cross_section_values) where:
    ///         - energy_grid: List of energy values in eV
    ///         - cross_section_values: List of cross section values in barns, weighted by natural abundance
    ///
    /// Raises:
    ///     Exception: If the element has no known isotopes, any isotope fails to load,
    ///                lacks the requested reaction data, or temperature is invalid.
    #[pyo3(text_signature = "(self, reaction, temperature=None)")]
    fn microscopic_cross_section(
        &mut self,
        reaction: &PyAny,
        temperature: Option<&str>,
    ) -> PyResult<(Vec<f64>, Vec<f64>)> {
        // Handle both int and string reaction parameters
        let result = if let Ok(mt_num) = reaction.extract::<i32>() {
            // Handle integer MT number
            self.inner.microscopic_cross_section(mt_num, temperature)
        } else if let Ok(reaction_name) = reaction.extract::<String>() {
            // Handle string reaction name
            self.inner.microscopic_cross_section(reaction_name, temperature)
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "reaction parameter must be either an integer (MT number) or string (reaction name)"
            ));
        };

        match result {
            Ok((energy, xs)) => Ok((energy, xs)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to get microscopic cross section: {}", e)
            )),
        }
    }
}
