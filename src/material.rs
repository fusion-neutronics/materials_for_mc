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
    /// Unified energy grid for different particle types
    /// Map of particle type -> energy grid
    pub unified_energy_grid: HashMap<String, Vec<f64>>,
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
            unified_energy_grid: HashMap::new(),
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
        self.unified_energy_grid.clear();
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

    /// Build a unified energy grid for all nuclides for a given particle across all MT reactions
    /// This method also stores the result in the material's unified_energy_grid property
    pub fn unified_energy_grid(
        &mut self,
        particle: &str,
    ) -> Vec<f64> {
        // Check if we already have this grid in the cache
        if let Some(grid) = self.unified_energy_grid.get(particle) {
            return grid.clone();
        }
        
        // If not cached, build the grid
        let mut all_energies = Vec::new();
        let temperature = &self.temperature;
        
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
        self.unified_energy_grid.insert(particle.to_string(), all_energies.clone());
        
        all_energies
    }
    
    /// Get the cached unified energy grid if available, or build it if not
    pub fn get_unified_energy_grid(
        &mut self,
        particle: &str,
    ) -> Vec<f64> {
        self.unified_energy_grid(particle)
    }

    /// Calculate microscopic cross sections on the unified energy grid
    /// 
    /// This method interpolates the microscopic cross sections for each nuclide
    /// onto the unified energy grid for all available MT reactions.
    /// If unified_energy_grid is None, it will use the cached grid or build a new one.
    /// Returns a nested HashMap: nuclide -> mt -> cross_section values
    pub fn calculate_microscopic_xs(
        &mut self,
        particle: &str,
        unified_energy_grid: Option<&[f64]>,
    ) -> HashMap<String, HashMap<String, Vec<f64>>> {
        // Get the grid (either from parameter or from cache/build)
        let grid = match unified_energy_grid {
            Some(grid) => grid.to_vec(),
            None => self.unified_energy_grid(particle),
        };
        
        let mut micro_xs: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
        let temperature = &self.temperature;
        
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

    /// Calculate macroscopic cross sections on the unified energy grid
    /// 
    /// This method calculates the total macroscopic cross section by:
    /// 1. Interpolating the microscopic cross sections onto the unified grid
    /// 2. Multiplying by atom density for each nuclide
    /// 3. Summing over all nuclides
    /// 
    /// Currently only supports neutron particles.
    pub fn calculate_macroscopic_xs(
        &mut self,
        particle: &str,
        unified_grid: Option<&[f64]>,
    ) -> HashMap<String, Vec<f64>> {
        if particle != "neutron" {
            // For now, we only support neutron cross sections
            return HashMap::new();
        }
        
        const BARN_TO_CM2: f64 = 1.0e-24; // conversion from barns to cm²
        
        // First get microscopic cross sections on the unified grid
        let micro_xs = self.calculate_microscopic_xs(particle, unified_grid);
        
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
        for (nuclide, fraction) in &self.nuclides {
            if let Some(nuclide_data) = micro_xs.get(nuclide) {
                if let Some(density) = self.density {
                    // TODO: Implement proper atoms_per_cc calculation based on atomic mass
                    // This is a simplified version
                    let atoms_per_cc = fraction * density;
                    
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
        if particle == "neutron" {
            self.macroscopic_xs_neutron = macro_xs.clone();
        }
        
        macro_xs
    }


    // pub fn get_nuclide_fraction(&self, nuclide: &str) -> Option<f64> {
    //     self.nuclides.get(nuclide).cloned()
    // }

    // pub fn get_total_fraction(&self) -> f64 {
    //     self.nuclides.values().sum()
    // }

    // pub fn normalize(&mut self) -> Result<(), String> {
    //     let total = self.get_total_fraction();
    //     if total <= 0.0 {
    //         return Err(String::from("Cannot normalize: total fraction is zero or negative"));
    //     }

    //     for fraction in self.nuclides.values_mut() {
    //         *fraction /= total;
    //     }

    //     Ok(())
    // }
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

}
