#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pyclass(name = "Reaction")]
/// Lightweight reaction summary exposed to Python.
///
/// Represents a single nuclear reaction channel with reactants, products and an
/// associated energy (e.g. Q-value or representative energy in MeV).
///
/// Attributes:
///     reactants (List[str]): Names / symbols of incoming particles or nuclei.
///     products (List[str]): Names / symbols of outgoing particles or nuclei.
///     energy (float): Reaction energy value (units depend on source data, typically MeV).
pub struct PyReaction {
    #[pyo3(get)]
    pub reactants: Vec<String>,
    #[pyo3(get)]
    pub products: Vec<String>,
    #[pyo3(get)]
    pub energy: f64,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyReaction {
    /// Create a reaction description.
    ///
    /// Args:
    ///     reactants (List[str]): Incoming particles/nuclei.
    ///     products (List[str]): Outgoing particles/nuclei.
    ///     energy (float): Reaction energy (e.g. Q-value, MeV).
    ///
    /// Returns:
    ///     PyReaction: New reaction object.
    #[pyo3(text_signature = "(reactants, products, energy)")]
    #[new]
    pub fn new(reactants: Vec<String>, products: Vec<String>, energy: f64) -> Self {
        PyReaction { reactants, products, energy }
    }
}
