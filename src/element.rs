// Provides functionality for working with natural elements and their isotopic abundances
use std::collections::HashMap;
use crate::data::NATURAL_ABUNDANCE;

/// Internal: build mapping element symbol -> sorted list of isotope strings
fn element_isotopes_map() -> HashMap<&'static str, Vec<&'static str>> {
    let mut map: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
    for isotope in NATURAL_ABUNDANCE.keys() {
        let mut i = 0;
        while i < isotope.len() && !isotope.chars().nth(i).unwrap().is_ascii_digit() {
            i += 1;
        }
        let element = &isotope[0..i];
        map.entry(element).or_insert_with(Vec::new).push(isotope);
    }
    for isotopes in map.values_mut() {
        isotopes.sort_by(|a, b| {
            let a_mass = a.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().unwrap();
            let b_mass = b.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().unwrap();
            a_mass.cmp(&b_mass)
        });
    }
    map
}

/// (Deprecated external) Full mapping element -> isotopes (owned Strings).
///
/// This free function is now superseded by `Element::all_nuclides_map()` and
/// will be removed in a future release.
pub fn all_element_isotopes() -> HashMap<String, Vec<String>> {
    Element::all_nuclides_map()
}

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
    pub fn new<S: Into<String>>(name: S) -> Self { Self { name: name.into() } }

    /// Return a mapping of element symbol -> sorted list of isotope names for all elements.
    pub fn all_nuclides_map() -> HashMap<String, Vec<String>> {
        element_isotopes_map().into_iter()
            .map(|(k, v)| (k.to_string(), v.into_iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    /// Return the list of nuclide (isotope) names (e.g. ["Fe54", "Fe56", ...]) for this element.
    pub fn get_nuclides(&self) -> Vec<String> {
        let map = element_isotopes_map();
        map.get(self.name.as_str())
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
    fn test_all_element_isotopes_map() {
    let map = Element::all_nuclides_map();
        assert!(map.get("Li").unwrap().contains(&"Li6".to_string()));
        assert!(map.get("Li").unwrap().contains(&"Li7".to_string()));
    }

    #[test]
    fn test_unknown_element() {
        let fake = Element::new("Xx");
    assert!(fake.get_nuclides().is_empty());
    }
}

