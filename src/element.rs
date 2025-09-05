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

/// Public convenience: full mapping element -> isotopes (owned Strings)
pub fn all_element_isotopes() -> HashMap<String, Vec<String>> {
    element_isotopes_map().into_iter()
        .map(|(k, v)| (k.to_string(), v.into_iter().map(|s| s.to_string()).collect()))
        .collect()
}

/// Element structure representing a chemical element with helper methods.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Element {
    pub name: String,
}

impl Element {
    pub fn new<S: Into<String>>(name: S) -> Self { Self { name: name.into() } }

    /// Return the list of isotope names (e.g. ["Fe54", "Fe56", ...]) for this element.
    pub fn get_element_isotopes(&self) -> Vec<String> {
        let map = element_isotopes_map();
        map.get(self.name.as_str())
            .map(|v| v.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Backwards-compat: alias for get_element_isotopes (if earlier API used get_nuclides)
    pub fn get_nuclides(&self) -> Vec<String> { self.get_element_isotopes() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_struct_isotopes() {
        let fe = Element::new("Fe");
        let list = fe.get_element_isotopes();
        assert!(list.contains(&"Fe54".to_string()));
        assert!(list.contains(&"Fe56".to_string()));
        assert!(list.contains(&"Fe57".to_string()));
        assert!(list.contains(&"Fe58".to_string()));
    }

    #[test]
    fn test_all_element_isotopes_map() {
        let map = all_element_isotopes();
        assert!(map.get("Li").unwrap().contains(&"Li6".to_string()));
        assert!(map.get("Li").unwrap().contains(&"Li7".to_string()));
    }

    #[test]
    fn test_unknown_element() {
        let fake = Element::new("Xx");
        assert!(fake.get_element_isotopes().is_empty());
    }
}

