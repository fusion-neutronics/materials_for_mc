// Provides functionality for working with natural elements and their isotopic abundances
use std::collections::HashMap;

use crate::data::{NATURAL_ABUNDANCE};

/// Mapping from element symbol to list of isotopes for that element
pub fn get_element_isotopes() -> HashMap<&'static str, Vec<&'static str>> {
    let mut element_isotopes: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
    
    // Build a mapping from elements to their isotopes
    for isotope in NATURAL_ABUNDANCE.keys() {
        // Extract the element symbol (all characters before the first digit)
        let mut i = 0;
        while i < isotope.len() && !isotope.chars().nth(i).unwrap().is_digit(10) {
            i += 1;
        }
        
        let element = &isotope[0..i];
        
        element_isotopes.entry(element)
            .or_insert_with(Vec::new)
            .push(isotope);
    }
    
    // Sort isotopes by mass number for each element
    for isotopes in element_isotopes.values_mut() {
        isotopes.sort_by(|a, b| {
            // Extract the mass number as a number
            let a_mass = a[0..].chars().filter(|c| c.is_digit(10))
                .collect::<String>().parse::<u32>().unwrap();
            let b_mass = b[0..].chars().filter(|c| c.is_digit(10))
                .collect::<String>().parse::<u32>().unwrap();
            a_mass.cmp(&b_mass)
        });
    }
    
    element_isotopes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_element_isotopes_lithium() {
        let element_isotopes = get_element_isotopes();
        let li_isotopes = element_isotopes.get("Li").expect("Li should be present");
        // Lithium should have Li6 and Li7
        assert!(li_isotopes.contains(&"Li6"));
        assert!(li_isotopes.contains(&"Li7"));
        assert_eq!(li_isotopes.len(), 2);
    }

    #[test]
    fn test_get_element_isotopes_tin() {
        let element_isotopes = get_element_isotopes();
        let sn_isotopes = element_isotopes.get("Sn").expect("Sn should be present");
        // Tin has 10 stable isotopes
        let expected = ["Sn112","Sn114","Sn115","Sn116","Sn117","Sn118","Sn119","Sn120","Sn122","Sn124"];
        for iso in &expected {
            assert!(sn_isotopes.contains(iso), "{} should be in Sn isotopes", iso);
        }
        assert_eq!(sn_isotopes.len(), expected.len());
    }

    #[test]
    fn test_get_element_isotopes_nonexistent() {
        let element_isotopes = get_element_isotopes();
        assert!(element_isotopes.get("Xx").is_none());
    }
}

