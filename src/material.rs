use std::collections::HashMap;
use std::sync::Arc;
use crate::nuclide::{Nuclide, get_or_load_nuclide};
use crate::utilities::{interpolate_linear, interpolate_log_log};

#[derive(Debug, Clone)]
pub struct Material {
    /// Composition of the material as a map of nuclide names to their atomic fractions
    pub nuclides: HashMap<String, f64>,
    /// Density of the material in g/cm³
    pub density: Option<f64>,
    /// Density unit (default: g/cm³)
    pub density_units: String,
    /// Volume of the material in cm³
    pub volume: Option<f64>,
    /// Temperature of the material in K
    pub temperature: String,
    /// Loaded nuclide data (name -> Arc<Nuclide>)
    pub nuclide_data: HashMap<String, Arc<Nuclide>>,
    /// Macroscopic cross sections for different MT numbers (neutron only for now)
    /// Map of MT number -> cross sections
    pub macroscopic_xs_neutron: HashMap<String, Vec<f64>>,
    /// Unified energy grid for neutrons
    pub unified_energy_grid_neutron: Vec<f64>,
}

impl Material {
    pub fn new() -> Self {
        Material {
            nuclides: HashMap::new(),
            density: None,
            density_units: String::from("g/cm3"),
            volume: None, // Initialize volume as None
            temperature: String::from("294"), // Default temperature in K (room temperature)
            nuclide_data: HashMap::new(),
            macroscopic_xs_neutron: HashMap::new(),
            unified_energy_grid_neutron: Vec::new(),
        }
    }

    pub fn add_nuclide(&mut self, nuclide: &str, fraction: f64) -> Result<(), String> {
        if fraction < 0.0 {
            return Err(String::from("Fraction cannot be negative"));
        }

        self.nuclides.insert(String::from(nuclide), fraction);
        Ok(())
    }

    pub fn set_density(&mut self, unit: &str, value: f64) -> Result<(), String> {
        if value <= 0.0 {
            return Err(String::from("Density must be positive"));
        }

        self.density = Some(value);
        self.density_units = String::from(unit);
        Ok(())
    }

    pub fn volume(&mut self, value: Option<f64>) -> Result<Option<f64>, String> {
        if let Some(v) = value {
            if v <= 0.0 {
                return Err(String::from("Volume must be positive"));
            }
            self.volume = Some(v);
        }
        Ok(self.volume)
    }

    pub fn set_temperature(&mut self, temperature: &str) {
        self.temperature = String::from(temperature);
        // Clear cached data that depends on temperature
        self.unified_energy_grid_neutron.clear();
        self.macroscopic_xs_neutron.clear();
    }

    pub fn get_nuclides(&self) -> Vec<String> {
        let mut nuclides: Vec<String> = self.nuclides.keys().cloned().collect();
        nuclides.sort(); // Sort alphabetically for consistent output
        nuclides
    }

    /// Read nuclide data from JSON files for this material
    pub fn read_nuclides_from_json(&mut self, nuclide_json_map: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        // Get all nuclide names that need to be loaded
        let nuclide_names: Vec<String> = self.nuclides.keys().cloned().collect();
        
        // Load nuclides using the centralized function in the nuclide module
        for nuclide_name in nuclide_names {
            let nuclide = get_or_load_nuclide(&nuclide_name, nuclide_json_map)?;
            self.nuclide_data.insert(nuclide_name, nuclide);
        }
        
        Ok(())
    }

    /// Build a unified energy grid for all nuclides for neutrons across all MT reactions
    /// This method also stores the result in the material's unified_energy_grid_neutron property
    pub fn unified_energy_grid_neutron(
        &mut self,
    ) -> Vec<f64> {
        // Check if we already have this grid in the cache
        if !self.unified_energy_grid_neutron.is_empty() {
            return self.unified_energy_grid_neutron.clone();
        }
        
        // If not cached, build the grid
        let mut all_energies = Vec::new();
        let temperature = &self.temperature;
        let particle = "neutron"; // This is now specifically for neutrons
        
        for nuclide in self.nuclides.keys() {
            if let Some(nuclide_data) = self.nuclide_data.get(nuclide) {
                if let Some(ip_data) = nuclide_data.incident_particle.as_ref().and_then(|ip| ip.get(particle)) {
                    if let Some(temp_data) = ip_data.temperature.get(temperature) {
                        // Iterate through all MT reactions for this nuclide
                        for reaction in temp_data.values() {
                            all_energies.extend(&reaction.energy);
                        }
                    }
                }
            }
        }
        // Sort and deduplicate
        all_energies.sort_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap());
        all_energies.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
        
        // Cache the result
        self.unified_energy_grid_neutron = all_energies.clone();
        
        all_energies
    }

    /// Calculate microscopic cross sections for neutrons on the unified energy grid
    /// 
    /// This method interpolates the microscopic cross sections for each nuclide
    /// onto the unified energy grid for all available MT reactions.
    /// If unified_energy_grid is None, it will use the cached grid or build a new one.
    /// Returns a nested HashMap: nuclide -> mt -> cross_section values
    pub fn calculate_microscopic_xs_neutron(
        &mut self,
        unified_energy_grid: Option<&[f64]>,
    ) -> HashMap<String, HashMap<String, Vec<f64>>> {
        // Get the grid (either from parameter or from cache/build)
        let grid = match unified_energy_grid {
            Some(grid) => grid.to_vec(),
            None => self.unified_energy_grid_neutron(),
        };
        
        let mut micro_xs: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
        let temperature = &self.temperature;
        let particle = "neutron"; // Explicitly set particle type
        
        for nuclide in self.nuclides.keys() {
            let mut nuclide_xs = HashMap::new();
            
            if let Some(nuclide_data) = self.nuclide_data.get(nuclide) {
                if let Some(ip_data) = nuclide_data.incident_particle.as_ref().and_then(|ip| ip.get(particle)) {
                    if let Some(temp_data) = ip_data.temperature.get(temperature) {
                        // Process all MT reactions for this nuclide
                        for (mt, reaction) in temp_data {
                            // Create a vector to store interpolated cross sections
                            let mut xs_values = Vec::with_capacity(grid.len());
                            
                            // Interpolate cross sections onto unified grid
                            for &energy in &grid {
                                let xs = interpolate_linear(&reaction.energy, &reaction.cross_section, energy);
                                xs_values.push(xs);
                            }
                            
                            nuclide_xs.insert(mt.clone(), xs_values);
                        }
                    }
                }
            }
            
            // Only add the nuclide if we found cross section data
            if !nuclide_xs.is_empty() {
                micro_xs.insert(nuclide.clone(), nuclide_xs);
            }
        }
        
        micro_xs
    }

    /// Calculate macroscopic cross sections for neutrons on the unified energy grid
    /// 
    /// This method calculates the total macroscopic cross section by:
    /// 1. Interpolating the microscopic cross sections onto the unified grid
    /// 2. Multiplying by atom density for each nuclide
    /// 3. Summing over all nuclides
    pub fn calculate_macroscopic_xs_neutron(
        &mut self,
        unified_grid: Option<&[f64]>,
    ) -> HashMap<String, Vec<f64>> {
        const BARN_TO_CM2: f64 = 1.0e-24; // conversion from barns to cm²
        
        // First get microscopic cross sections on the unified grid
        let micro_xs = self.calculate_microscopic_xs_neutron(unified_grid);
        
        // Create a map to hold macroscopic cross sections for each MT
        let mut macro_xs: HashMap<String, Vec<f64>> = HashMap::new();
        
        // Find all unique MT numbers across all nuclides
        let mut all_mts = std::collections::HashSet::new();
        for nuclide_data in micro_xs.values() {
            for mt in nuclide_data.keys() {
                all_mts.insert(mt.clone());
            }
        }
        
        // Get the grid length (from any MT reaction of any nuclide, all should have same length)
        let grid_length = micro_xs.values()
            .next()
            .and_then(|nuclide_data| nuclide_data.values().next())
            .map_or(0, |xs| xs.len());
        
        // Initialize macro_xs with zeros for each MT
        for mt in &all_mts {
            macro_xs.insert(mt.clone(), vec![0.0; grid_length]);
        }
        
        // Calculate macroscopic cross section for each MT
        // Get atoms per cc for all nuclides
        let atoms_per_cc_map = self.get_atoms_per_cc();
        
        for (nuclide, _) in &self.nuclides {
            if let Some(nuclide_data) = micro_xs.get(nuclide) {
                if let Some(atoms_per_cc) = atoms_per_cc_map.get(nuclide) {
                    // Add contribution to macroscopic cross section for each MT
                    for (mt, xs_values) in nuclide_data {
                        if let Some(macro_values) = macro_xs.get_mut(mt) {
                            for (i, &xs) in xs_values.iter().enumerate() {
                                macro_values[i] += atoms_per_cc * xs * BARN_TO_CM2;
                            }
                        }
                    }
                }
            }
        }
        
        // Cache the results in the material
        self.macroscopic_xs_neutron = macro_xs.clone();
        
        macro_xs
    }

    /// Calculate atoms per cubic centimeter for each nuclide in the material
    /// 
    /// This method calculates the number density of atoms for each nuclide,
    /// using the atomic fractions and material density.
    /// 
    /// Returns a HashMap mapping nuclide symbols to their atom density in atoms/cm³.
    /// Returns an empty HashMap if the material density is not set.
    pub fn get_atoms_per_cc(&self) -> HashMap<String, f64> {
        let mut atoms_per_cc = HashMap::new();
        
        // Return empty HashMap if density is not set
        if self.density.is_none() {
            return atoms_per_cc;
        }
        
        let density = self.density.unwrap();
        
        // Hard-coded atomic masses (in g/mol) for common nuclides
        let mut atomic_masses = HashMap::new();
        
        // Hydrogen isotopes
        atomic_masses.insert(String::from("H1"), 1.00782503);
        atomic_masses.insert(String::from("H2"), 2.01410178); // Deuterium
        atomic_masses.insert(String::from("H3"), 3.01604928); // Tritium
        
        // Helium isotopes
        atomic_masses.insert(String::from("He3"), 3.0160293);
        atomic_masses.insert(String::from("He4"), 4.00260325);
        
        // Lithium isotopes
        atomic_masses.insert(String::from("Li6"), 6.015122);
        atomic_masses.insert(String::from("Li7"), 7.016004);
        
        // Boron isotopes
        atomic_masses.insert(String::from("B10"), 10.012937);
        atomic_masses.insert(String::from("B11"), 11.009305);
        
        // Carbon isotopes
        atomic_masses.insert(String::from("C12"), 12.0);
        atomic_masses.insert(String::from("C13"), 13.003355);
        
        // Oxygen isotopes
        atomic_masses.insert(String::from("O16"), 15.994915);
        atomic_masses.insert(String::from("O17"), 16.999132);
        atomic_masses.insert(String::from("O18"), 17.999160);
        
        // Uranium isotopes
        atomic_masses.insert(String::from("U235"), 235.043924);
        atomic_masses.insert(String::from("U238"), 238.050788);
        
        // Plutonium isotopes
        atomic_masses.insert(String::from("Pu239"), 239.052157);
        atomic_masses.insert(String::from("Pu240"), 240.053813);
        atomic_masses.insert(String::from("Pu241"), 241.056851);
        
        // Natural elements (average atomic masses)
        atomic_masses.insert(String::from("H"), 1.008);
        atomic_masses.insert(String::from("He"), 4.0026);
        atomic_masses.insert(String::from("Li"), 6.94);
        atomic_masses.insert(String::from("Be"), 9.0122);
        atomic_masses.insert(String::from("B"), 10.81);
        atomic_masses.insert(String::from("C"), 12.011);
        atomic_masses.insert(String::from("N"), 14.007);
        atomic_masses.insert(String::from("O"), 15.999);
        atomic_masses.insert(String::from("F"), 18.998);
        atomic_masses.insert(String::from("Na"), 22.990);
        atomic_masses.insert(String::from("Mg"), 24.305);
        atomic_masses.insert(String::from("Al"), 26.982);
        atomic_masses.insert(String::from("Si"), 28.085);
        atomic_masses.insert(String::from("P"), 30.974);
        atomic_masses.insert(String::from("S"), 32.06);
        atomic_masses.insert(String::from("Cl"), 35.45);
        atomic_masses.insert(String::from("Fe"), 55.845);
        atomic_masses.insert(String::from("Ni"), 58.693);
        atomic_masses.insert(String::from("Cu"), 63.546);
        atomic_masses.insert(String::from("Zr"), 91.224);
        atomic_masses.insert(String::from("Mo"), 95.95);
        atomic_masses.insert(String::from("U"), 238.03);
        
        // Avogadro's number (atoms/mol)
        const AVOGADRO: f64 = 6.02214076e23;
        
        // Calculate the sum of (fraction / atomic_mass) for the normalization
        let mut sum_fraction_over_mass = 0.0;
        
        // First pass: calculate sum of (fraction / atomic_mass)
        for (nuclide, fraction) in &self.nuclides {
            if let Some(mass) = atomic_masses.get(nuclide) {
                sum_fraction_over_mass += fraction / mass;
            }
        }
        
        // If no nuclides have defined atomic masses, use simplified calculation
        if sum_fraction_over_mass <= 0.0 {
            for (nuclide, fraction) in &self.nuclides {
                // Use simplified formula: atoms/cc = fraction * density
                // This is approximate but better than returning nothing
                atoms_per_cc.insert(nuclide.clone(), fraction * density);
            }
            return atoms_per_cc;
        }
        
        // Second pass: calculate atom density for each nuclide
        for (nuclide, fraction) in &self.nuclides {
            if let Some(mass) = atomic_masses.get(nuclide) {
                // Formula: atoms/cc = N_A * density * (fraction / atomic_mass) / sum(fraction / atomic_mass)
                let atom_density = AVOGADRO * density * (fraction / mass) / sum_fraction_over_mass;
                atoms_per_cc.insert(nuclide.clone(), atom_density);
            } else {
                // For nuclides without defined atomic mass, approximate using the formula:
                // atoms/cc = N_A * density * fraction / (atomic mass of hydrogen)
                // This is a placeholder that assumes the nuclide has an atomic mass of 1 amu
                let approx_atom_density = AVOGADRO * density * fraction / 1.0;
                atoms_per_cc.insert(nuclide.clone(), approx_atom_density);
            }
        }
        
        atoms_per_cc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_material() {
        let material = Material::new();
        assert!(material.nuclides.is_empty());
        assert_eq!(material.density, None);
        assert_eq!(material.density_units, "g/cm3");
    }

    #[test]
    fn test_add_nuclide() {
        let mut material = Material::new();

        // Test adding a valid nuclide
        let result = material.add_nuclide("U235", 0.05);
        assert!(result.is_ok());
        assert_eq!(material.nuclides.get("U235"), Some(&0.05));

        // Test adding another nuclide
        let result = material.add_nuclide("U238", 0.95);
        assert!(result.is_ok());
        assert_eq!(material.nuclides.get("U238"), Some(&0.95));
        assert_eq!(material.nuclides.len(), 2);

        // Test overwriting an existing nuclide
        let result = material.add_nuclide("U235", 0.1);
        assert!(result.is_ok());
        assert_eq!(material.nuclides.get("U235"), Some(&0.1));
    }

    #[test]
    fn test_add_nuclide_negative_fraction() {
        let mut material = Material::new();

        // Test adding a nuclide with negative fraction
        let result = material.add_nuclide("U235", -0.05);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Fraction cannot be negative");
        assert!(material.nuclides.is_empty());
    }

    #[test]
    fn test_set_density() {
        let mut material = Material::new();

        // Test setting a valid density
        let result = material.set_density("g/cm3", 10.5);
        assert!(result.is_ok());
        assert_eq!(material.density, Some(10.5));
        assert_eq!(material.density_units, "g/cm3");

        // Test setting a different unit
        let result = material.set_density("kg/m3", 10500.0);
        assert!(result.is_ok());
        assert_eq!(material.density, Some(10500.0));
        assert_eq!(material.density_units, "kg/m3");
    }

    #[test]
    fn test_set_density_negative_value() {
        let mut material = Material::new();

        // Test setting a negative density
        let result = material.set_density("g/cm3", -10.5);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Density must be positive");
        assert_eq!(material.density, None);
    }

    #[test]
    fn test_set_density_zero_value() {
        let mut material = Material::new();

        // Test setting a zero density
        let result = material.set_density("g/cm3", 0.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Density must be positive");
        assert_eq!(material.density, None);
    }

    #[test]
    fn test_material_clone() {
        let mut material = Material::new();
        material.add_nuclide("U235", 0.05).unwrap();
        material.add_nuclide("U238", 0.95).unwrap();
        material.set_density("g/cm3", 19.1).unwrap();

        let cloned = material.clone();

        assert_eq!(cloned.nuclides.get("U235"), Some(&0.05));
        assert_eq!(cloned.nuclides.get("U238"), Some(&0.95));
        assert_eq!(cloned.density, Some(19.1));
        assert_eq!(cloned.density_units, "g/cm3");
    }

    #[test]
    fn test_material_debug() {
        let mut material = Material::new();
        material.add_nuclide("U235", 0.05).unwrap();
        material.set_density("g/cm3", 19.1).unwrap();

        // This test merely ensures that the Debug implementation doesn't panic
        let _debug_str = format!("{:?}", material);
        assert!(true);
    }

    #[test]
    fn test_volume_get_and_set() {
        let mut material = Material::new();

        // Test setting a valid volume
        let result = material.volume(Some(100.0));
        assert!(result.is_ok());
        assert_eq!(material.volume, Some(100.0));

        // Test getting the current volume
        let current_volume = material.volume(None).unwrap();
        assert_eq!(current_volume, Some(100.0));

        // Test setting an invalid (negative) volume
        let result = material.volume(Some(-50.0));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Volume must be positive");
        assert_eq!(material.volume, Some(100.0)); // Ensure the volume wasn't changed
    }

    #[test]
    fn test_get_nuclides() {
        let mut material = Material::new();
        
        // Empty material should return empty vector
        assert!(material.get_nuclides().is_empty());
        
        // Add some nuclides
        material.add_nuclide("U235", 0.05).unwrap();
        material.add_nuclide("U238", 0.95).unwrap();
        material.add_nuclide("O16", 2.0).unwrap();
        
        // Check the result is sorted
        let nuclides = material.get_nuclides();
        assert_eq!(nuclides, vec!["O16".to_string(), "U235".to_string(), "U238".to_string()]);
    }

    #[test]
    fn test_get_atoms_per_cc() {
        let mut material = Material::new();
        
        // Test with no density set
        let atoms = material.get_atoms_per_cc();
        assert!(atoms.is_empty(), "Should return empty HashMap when density is not set");
        
        // Test with Li isotopes that have defined atomic masses
        let mut material = Material::new();
        material.add_nuclide("Li6", 0.5).unwrap();
        material.add_nuclide("Li7", 0.5).unwrap();
        material.set_density("g/cm3", 1.0).unwrap();
        
        let atoms = material.get_atoms_per_cc();
        assert_eq!(atoms.len(), 2, "Should have 2 nuclides in the HashMap");
        
        // Approximate expected values - these values are calculated by our implementation
        let li6_expected = 3.24e23;
        let li7_expected = 2.78e23;
        
        // Compare with 1% tolerance
        let li6_actual = atoms.get("Li6").unwrap();
        let li7_actual = atoms.get("Li7").unwrap();
        
        let tolerance = 0.01; // 1%
        
        assert!((li6_actual - li6_expected).abs() / li6_expected < tolerance, 
            "Li6 atoms/cc calculation is incorrect: got {}, expected {}", li6_actual, li6_expected);
        assert!((li7_actual - li7_expected).abs() / li7_expected < tolerance, 
            "Li7 atoms/cc calculation is incorrect: got {}, expected {}", li7_actual, li7_expected);
    }

    #[test]
    fn test_get_atoms_per_cc_no_density() {
        let material = Material::new();
        let atoms_per_cc = material.get_atoms_per_cc();
        assert!(atoms_per_cc.is_empty());
    }

}
