// Provides functionality for working with natural elements and their isotopic abundances
use std::collections::HashMap;
use once_cell::sync::Lazy;

use crate::material::Material;
use crate::data::NATURAL_ABUNDANCE;

// Map of element symbols to their full names
static ELEMENT_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut names = HashMap::new();
    names.insert("H", "hydrogen");
    names.insert("He", "helium");
    names.insert("Li", "lithium");
    names.insert("Be", "beryllium");
    names.insert("B", "boron");
    names.insert("C", "carbon");
    names.insert("N", "nitrogen");
    names.insert("O", "oxygen");
    names.insert("F", "fluorine");
    names.insert("Ne", "neon");
    names.insert("Na", "sodium");
    names.insert("Mg", "magnesium");
    names.insert("Al", "aluminum");
    names.insert("Si", "silicon");
    names.insert("P", "phosphorus");
    names.insert("S", "sulfur");
    names.insert("Cl", "chlorine");
    names.insert("Ar", "argon");
    names.insert("K", "potassium");
    names.insert("Ca", "calcium");
    names.insert("Sc", "scandium");
    names.insert("Ti", "titanium");
    names.insert("V", "vanadium");
    names.insert("Cr", "chromium");
    names.insert("Mn", "manganese");
    names.insert("Fe", "iron");
    names.insert("Co", "cobalt");
    names.insert("Ni", "nickel");
    names.insert("Cu", "copper");
    names.insert("Zn", "zinc");
    names.insert("Ga", "gallium");
    names.insert("Ge", "germanium");
    names.insert("As", "arsenic");
    names.insert("Se", "selenium");
    names.insert("Br", "bromine");
    names.insert("Kr", "krypton");
    names.insert("Rb", "rubidium");
    names.insert("Sr", "strontium");
    names.insert("Y", "yttrium");
    names.insert("Zr", "zirconium");
    names.insert("Nb", "niobium");
    names.insert("Mo", "molybdenum");
    names.insert("Tc", "technetium");
    names.insert("Ru", "ruthenium");
    names.insert("Rh", "rhodium");
    names.insert("Pd", "palladium");
    names.insert("Ag", "silver");
    names.insert("Cd", "cadmium");
    names.insert("In", "indium");
    names.insert("Sn", "tin");
    names.insert("Sb", "antimony");
    names.insert("Te", "tellurium");
    names.insert("I", "iodine");
    names.insert("Xe", "xenon");
    names.insert("Cs", "cesium");
    names.insert("Ba", "barium");
    names.insert("La", "lanthanum");
    names.insert("Ce", "cerium");
    names.insert("Pr", "praseodymium");
    names.insert("Nd", "neodymium");
    names.insert("Pm", "promethium");
    names.insert("Sm", "samarium");
    names.insert("Eu", "europium");
    names.insert("Gd", "gadolinium");
    names.insert("Tb", "terbium");
    names.insert("Dy", "dysprosium");
    names.insert("Ho", "holmium");
    names.insert("Er", "erbium");
    names.insert("Tm", "thulium");
    names.insert("Yb", "ytterbium");
    names.insert("Lu", "lutetium");
    names.insert("Hf", "hafnium");
    names.insert("Ta", "tantalum");
    names.insert("W", "tungsten");
    names.insert("Re", "rhenium");
    names.insert("Os", "osmium");
    names.insert("Ir", "iridium");
    names.insert("Pt", "platinum");
    names.insert("Au", "gold");
    names.insert("Hg", "mercury");
    names.insert("Tl", "thallium");
    names.insert("Pb", "lead");
    names.insert("Bi", "bismuth");
    names.insert("Po", "polonium");
    names.insert("At", "astatine");
    names.insert("Rn", "radon");
    names.insert("Fr", "francium");
    names.insert("Ra", "radium");
    names.insert("Ac", "actinium");
    names.insert("Th", "thorium");
    names.insert("Pa", "protactinium");
    names.insert("U", "uranium");
    names.insert("Np", "neptunium");
    names.insert("Pu", "plutonium");
    names.insert("Am", "americium");
    names.insert("Cm", "curium");
    names.insert("Bk", "berkelium");
    names.insert("Cf", "californium");
    names.insert("Es", "einsteinium");
    names.insert("Fm", "fermium");
    names.insert("Md", "mendelevium");
    names.insert("No", "nobelium");
    names.insert("Lr", "lawrencium");
    names
});

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
    fn test_add_element() {
        let mut material = Material::new();
        
        // Test adding natural lithium
        let result = material.add_element("Li", 1.0);
        assert!(result.is_ok());
        
        // Verify the isotopes were added correctly
        assert!(material.nuclides.contains_key("Li6"));
        assert!(material.nuclides.contains_key("Li7"));
        
        // Check the fractions are correct
        assert_eq!(*material.nuclides.get("Li6").unwrap(), 0.07589);
        assert_eq!(*material.nuclides.get("Li7").unwrap(), 0.92411);
        
        // Test adding an element with many isotopes
        let mut material2 = Material::new();
        let result = material2.add_element("Sn", 1.0); // Tin has 10 isotopes
        assert!(result.is_ok());
        assert_eq!(material2.nuclides.len(), 10);
    }
    
    #[test]
    fn test_add_element_invalid() {
        let mut material = Material::new();
        
        // Test with negative fraction
        let result = material.add_element("Li", -1.0);
        assert!(result.is_err());
        
        // Test with invalid element
        let result = material.add_element("Xx", 1.0);
        assert!(result.is_err());
    }
    
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
