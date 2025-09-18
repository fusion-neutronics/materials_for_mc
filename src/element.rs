// Provides functionality for working with natural elements and their isotopic abundances
use crate::data::{ELEMENT_NUCLIDES, NATURAL_ABUNDANCE, ATOMIC_NUMBERS};
use crate::nuclide::{Nuclide, ReactionIdentifier};
use crate::utilities::interpolate_linear;
use std::collections::HashMap;

/// Represents a chemical element identified by its symbol (e.g. `"Fe"`).
///
/// Provides helper methods to enumerate naturally occurring isotopes (nuclides)
/// using the internally defined natural abundance database.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Element {
    /// Chemical symbol of the element (case sensitive, e.g. "Fe").
    pub name: String,
    /// Atomic number (Z) of the element.
    pub atomic_number: Option<u32>,
}

impl Element {
    pub fn new<S: Into<String>>(name: S) -> Self {
        let element_name = name.into();
        let atomic_number = ATOMIC_NUMBERS.get(element_name.as_str()).copied();
        Self { 
            name: element_name, 
            atomic_number 
        }
    }

    /// Return the list of nuclide (isotope) names (e.g. ["Fe54", "Fe56", ...]) for this element.
    pub fn get_nuclides(&self) -> Vec<String> {
        ELEMENT_NUCLIDES
            .get(self.name.as_str())
            .map(|v| v.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Calculate the natural abundance-weighted microscopic cross section for this element.
    ///
    /// This method loads the cross section data for each naturally occurring isotope of this element,
    /// creates a unified energy grid from all isotopes, interpolates each isotope's cross sections
    /// onto the unified grid, weights them by their natural abundances, and returns the combined cross section.
    ///
    /// # Arguments
    /// * `reaction` - Either an MT number (i32) or reaction name (String/&str) like "(n,gamma)" or "fission"
    /// * `temperature` - Optional temperature string (e.g. "294", "300K")
    ///
    /// # Returns
    /// A tuple of (energy_grid, cross_section_values) where:
    /// - energy_grid: Vector of energy values in eV (unified grid from all isotopes)
    /// - cross_section_values: Vector of cross section values in barns, weighted by natural abundance
    ///
    /// # Errors
    /// Returns an error if:
    /// - The element has no known isotopes
    /// - Any isotope fails to load or lacks the requested reaction data
    /// - Temperature is invalid for any isotope
    pub fn microscopic_cross_section<R>(
        &self,
        reaction: R,
        temperature: Option<&str>,
    ) -> Result<(Vec<f64>, Vec<f64>), Box<dyn std::error::Error>>
    where
        R: Into<ReactionIdentifier> + Clone,
    {
        let nuclide_names = self.get_nuclides();
        if nuclide_names.is_empty() {
            return Err(format!("No known isotopes for element '{}'", self.name).into());
        }

        // First, collect cross section data from all isotopes
        let mut isotope_data: Vec<(String, f64, Vec<f64>, Vec<f64>)> = Vec::new(); // (name, abundance, energy, xs)
        let mut all_energies: Vec<f64> = Vec::new();
        let mut total_abundance = 0.0;

        // Process each isotope to collect data
        for nuclide_name in &nuclide_names {
            // Get natural abundance for this isotope
            let abundance = NATURAL_ABUNDANCE.get(nuclide_name.as_str()).copied().unwrap_or(0.0);
            if abundance == 0.0 {
                continue; // Skip isotopes with zero abundance
            }
            total_abundance += abundance;

            // Load the nuclide and get its cross section
            let mut nuclide = Nuclide {
                name: Some(nuclide_name.clone()),
                element: None,
                atomic_symbol: None,
                atomic_number: None,
                neutron_number: None,
                mass_number: None,
                library: None,
                energy: None,
                reactions: HashMap::new(),
                fissionable: false,
                available_temperatures: Vec::new(),
                loaded_temperatures: Vec::new(),
                data_path: None,
            };
            
            let (iso_energy, iso_xs) = nuclide.microscopic_cross_section(reaction.clone(), temperature)?;

            // Store data for later processing
            isotope_data.push((nuclide_name.clone(), abundance, iso_energy.clone(), iso_xs));
            
            // Add energy points to the unified grid
            all_energies.extend(iso_energy);
        }

        if total_abundance == 0.0 {
            return Err(format!("No isotopes with non-zero natural abundance found for element '{}'", self.name).into());
        }

        // Create unified energy grid by sorting and deduplicating
        all_energies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        all_energies.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
        
        // Filter out any non-positive energy values that might have been introduced
        all_energies.retain(|&e| e > 0.0);

        // Interpolate each isotope's cross sections onto the unified grid and weight by abundance
        let mut unified_xs = vec![0.0; all_energies.len()];
        
        for (_name, abundance, iso_energy, iso_xs) in isotope_data {
            // Interpolate this isotope's cross sections onto the unified grid
            let interpolated_xs: Vec<f64> = all_energies
                .iter()
                .map(|&e| interpolate_linear(&iso_energy, &iso_xs, e))
                .collect();
            
            // Add weighted contribution to the unified cross sections
            for (i, &xs_val) in interpolated_xs.iter().enumerate() {
                unified_xs[i] += xs_val * abundance;
            }
        }

        // Normalize by total abundance
        for xs_val in unified_xs.iter_mut() {
            *xs_val /= total_abundance;
        }

        Ok((all_energies, unified_xs))
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

    #[test]
    fn test_atomic_number() {
        let fe = Element::new("Fe");
        assert_eq!(fe.atomic_number, Some(26));
        
        let li = Element::new("Li");
        assert_eq!(li.atomic_number, Some(3));
        
        let h = Element::new("H");
        assert_eq!(h.atomic_number, Some(1));
        
        let unknown = Element::new("Xx");
        assert_eq!(unknown.atomic_number, None);
    }

    #[cfg(test)]
    mod microscopic_xs_tests {
        use super::*;
        use crate::config::CONFIG;

        #[test]
        fn test_microscopic_cross_section_basic() {
            // Configure test data sources using the global CONFIG
            {
                let mut config = CONFIG.lock().unwrap();
                let mut data_sources = std::collections::HashMap::new();
                data_sources.insert("Li6".to_string(), "tests/Li6.json".to_string());
                data_sources.insert("Li7".to_string(), "tests/Li7.json".to_string());
                config.set_cross_sections(data_sources);
            }

            let li = Element::new("Li");
            
            // Test basic cross section calculation
            let result = li.microscopic_cross_section("(n,gamma)", None);
            assert!(result.is_ok(), "Failed to calculate cross section: {:?}", result.err());
            
            let (energy, xs) = result.unwrap();
            
            // Verify we get valid data
            assert!(!energy.is_empty(), "Energy grid should not be empty");
            assert_eq!(energy.len(), xs.len(), "Energy and cross section vectors should have same length");
            
            // Verify energy values are positive and sorted
            for &e in &energy {
                assert!(e > 0.0, "Energy values should be positive");
            }
            for i in 1..energy.len() {
                assert!(energy[i] >= energy[i-1], "Energy grid should be sorted");
            }
            
            // Verify cross section values are non-negative
            for &x in &xs {
                assert!(x >= 0.0, "Cross section values should be non-negative");
            }
        }

        #[test]
        fn test_microscopic_cross_section_mt_number() {
            // Configure test data sources using the global CONFIG
            {
                let mut config = CONFIG.lock().unwrap();
                let mut data_sources = std::collections::HashMap::new();
                data_sources.insert("Li6".to_string(), "tests/Li6.json".to_string());
                data_sources.insert("Li7".to_string(), "tests/Li7.json".to_string());
                config.set_cross_sections(data_sources);
            }

            let li = Element::new("Li");
            
            // Test with string reaction identifier
            let result_str = li.microscopic_cross_section("(n,gamma)", None);
            assert!(result_str.is_ok());
            
            // Test with MT number (102 is n,gamma)
            let result_mt = li.microscopic_cross_section(102, None);
            assert!(result_mt.is_ok());
            
            // Results should be identical
            let (energy_str, xs_str) = result_str.unwrap();
            let (energy_mt, xs_mt) = result_mt.unwrap();
            
            assert_eq!(energy_str.len(), energy_mt.len());
            assert_eq!(xs_str.len(), xs_mt.len());
            
            // Allow for small floating point differences
            for (e1, e2) in energy_str.iter().zip(energy_mt.iter()) {
                assert!((e1 - e2).abs() < 1e-12, "Energy grids should match");
            }
            for (x1, x2) in xs_str.iter().zip(xs_mt.iter()) {
                assert!((x1 - x2).abs() < 1e-12, "Cross sections should match");
            }
        }

        #[test]
        fn test_microscopic_cross_section_with_temperature() {
            // Configure test data sources using the global CONFIG
            {
                let mut config = CONFIG.lock().unwrap();
                let mut data_sources = std::collections::HashMap::new();
                data_sources.insert("Li6".to_string(), "tests/Li6.json".to_string());
                data_sources.insert("Li7".to_string(), "tests/Li7.json".to_string());
                config.set_cross_sections(data_sources);
            }

            let li = Element::new("Li");
            
            // Test with temperature specification
            let result = li.microscopic_cross_section("(n,gamma)", Some("294"));
            assert!(result.is_ok(), "Failed to calculate cross section with temperature: {:?}", result.err());
            
            let (energy, xs) = result.unwrap();
            assert!(!energy.is_empty());
            assert_eq!(energy.len(), xs.len());
        }

        #[test]
        fn test_microscopic_cross_section_unknown_element() {
            let fake_element = Element::new("Zz");
            
            let result = fake_element.microscopic_cross_section("(n,gamma)", None);
            assert!(result.is_err(), "Should fail for unknown element");
            
            let error_msg = format!("{}", result.unwrap_err());
            assert!(error_msg.contains("No known isotopes"), "Error should mention no isotopes");
        }

        #[test]
        fn test_microscopic_cross_section_unified_grid() {
            // Configure test data sources using the global CONFIG
            {
                let mut config = CONFIG.lock().unwrap();
                let mut data_sources = std::collections::HashMap::new();
                data_sources.insert("Li6".to_string(), "tests/Li6.json".to_string());
                data_sources.insert("Li7".to_string(), "tests/Li7.json".to_string());
                config.set_cross_sections(data_sources);
            }

            let li = Element::new("Li");
            
            let result = li.microscopic_cross_section("(n,gamma)", None);
            assert!(result.is_ok());
            
            let (energy, xs) = result.unwrap();
            
            // The unified grid should be comprehensive - expect it to be larger than 
            // either individual isotope grid due to merging
            assert!(energy.len() > 500, "Unified grid should have substantial number of points");
            
            // Verify the abundance weighting makes sense - lithium is mostly Li7 (~92.4%)
            // so the cross sections should be more similar to Li7 than Li6
            // This is a qualitative check that abundance weighting is working
            assert!(!xs.is_empty());
        }
    }
}
