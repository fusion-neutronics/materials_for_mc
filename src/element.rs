// Provides functionality for working with natural elements and their isotopic abundances
use std::collections::HashMap;
use once_cell::sync::Lazy;

use crate::material::Material;
use crate::data::{NATURAL_ABUNDANCE, ELEMENT_NAMES};

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

/// Extension trait for Material to add element-related functionality
pub trait ElementExtensions {
    /// Get the list of available elements
    fn get_available_elements() -> Vec<String>;
}

impl ElementExtensions for Material {
    /// Get the list of available elements
    fn get_available_elements() -> Vec<String> {
        let element_isotopes = get_element_isotopes();
        let mut elements: Vec<String> = element_isotopes.keys()
            .map(|&elem| elem.to_string())
            .collect();
        elements.sort();
        elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_available_elements() {
        let elements = Material::get_available_elements();
        
        // Check that some common elements are in the list
        assert!(elements.contains(&"H".to_string()));
        assert!(elements.contains(&"He".to_string()));
        assert!(elements.contains(&"Li".to_string()));
        assert!(elements.contains(&"U".to_string()));
        
        // Check that the list is sorted
        let mut sorted_elements = elements.clone();
        sorted_elements.sort();
        assert_eq!(elements, sorted_elements);
    }
}
