// Provides functionality for working with natural elements and their isotopic abundances
use crate::data::ELEMENT_NUCLIDES;

/// Represents a chemical element identified by its symbol (e.g. `"Fe"`).
///
/// Provides helper methods to enumerate naturally occurring isotopes (nuclides)
/// using the internally defined natural abundance database.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Element {
    /// Chemical symbol of the element (case sensitive, e.g. "Fe").
    pub name: String,
}

impl Element {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }

    /// Return the list of nuclide (isotope) names (e.g. ["Fe54", "Fe56", ...]) for this element.
    pub fn get_nuclides(&self) -> Vec<String> {
        ELEMENT_NUCLIDES
            .get(self.name.as_str())
            .map(|v| v.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_struct_isotopes() {
        let fe = Element::new("Fe");
        let list = fe.get_nuclides();
        assert!(list.contains(&"Fe54".to_string()));
        assert!(list.contains(&"Fe56".to_string()));
        assert!(list.contains(&"Fe57".to_string()));
        assert!(list.contains(&"Fe58".to_string()));
    }

    #[test]
    fn test_unknown_element() {
        let fake = Element::new("Xx");
        assert!(fake.get_nuclides().is_empty());
    }

    #[test]
    fn test_get_nuclides_fe_full_list() {
        let fe = Element::new("Fe");
        let list = fe.get_nuclides();
        let expected = vec!["Fe54", "Fe56", "Fe57", "Fe58"];
        assert_eq!(
            list.len(),
            expected.len(),
            "Unexpected number of Fe isotopes: {:?}",
            list
        );
        assert_eq!(
            list,
            expected.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            "Fe isotope list mismatch"
        );
    }
}
