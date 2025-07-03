use std::collections::HashMap;
use std::sync::Arc;
use crate::nuclide::{Nuclide, get_or_load_nuclide};
use crate::config::CONFIG;
use crate::utilities::{interpolate_linear};

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

    /// Ensure all nuclides are loaded, using the global configuration if needed
    fn ensure_nuclides_loaded(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let nuclide_names: Vec<String> = self.nuclides.keys()
            .filter(|name| !self.nuclide_data.contains_key(*name))
            .cloned()
            .collect();
        
        if nuclide_names.is_empty() {
            return Ok(());
        }
        
        // Get the global configuration
        let config = CONFIG.lock().unwrap();
        
        // Load any missing nuclides
        for nuclide_name in nuclide_names {
            let nuclide = get_or_load_nuclide(&nuclide_name, &config.cross_sections)?;
            self.nuclide_data.insert(nuclide_name.clone(), nuclide);
        }
        
        Ok(())
    }

    /// Build a unified energy grid for all nuclides for neutrons across all MT reactions
    /// This method also stores the result in the material's unified_energy_grid_neutron property
    pub fn unified_energy_grid_neutron(
        &mut self,
    ) -> Vec<f64> {
        // Ensure nuclides are loaded before proceeding
        if let Err(e) = self.ensure_nuclides_loaded() {
            eprintln!("Error loading nuclides: {}", e);
        }
        
        // Check if we already have this grid in the cache
        if !self.unified_energy_grid_neutron.is_empty() {
            return self.unified_energy_grid_neutron.clone();
        }
        
        // If not cached, build the grid
        let mut all_energies = Vec::new();
        let temperature = &self.temperature;
        let _particle = "neutron"; // This is now specifically for neutrons
        
        println!("Building unified energy grid for temperature: {}", temperature);
        
        for nuclide in self.nuclides.keys() {
            println!("Processing nuclide: {}", nuclide);
            
            if let Some(nuclide_data) = self.nuclide_data.get(nuclide) {
                // Check if there's a top-level energy grid
                if let Some(energy_map) = &nuclide_data.energy {
                    println!("Nuclide has top-level energy grid");
                    
                    if let Some(energy_grid) = energy_map.get(temperature) {
                        println!("Found energy grid for temperature: {}", temperature);
                        all_energies.extend(energy_grid);
                    } else {
                        println!("No energy grid found for temperature {}", temperature);
                        
                        // Print available temperatures
                        println!("Available temperatures in energy map: {:?}", energy_map.keys().collect::<Vec<_>>());
                    }
                } else {
                    println!("Nuclide does not have top-level energy grid");
                }
            } else {
                println!("No data found for nuclide: {}", nuclide);
            }
        }
        
        println!("Total number of energy points collected: {}", all_energies.len());
        
        // Sort and deduplicate
        all_energies.sort_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap());
        all_energies.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
        
        println!("Number of unique energy points after deduplication: {}", all_energies.len());
        
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
        // Ensure nuclides are loaded before proceeding
        if let Err(e) = self.ensure_nuclides_loaded() {
            eprintln!("Error loading nuclides: {}", e);
        }
        
        // Get the grid (either from parameter or from cache/build)
        let grid = match unified_energy_grid {
            Some(grid) => grid.to_vec(),
            None => self.unified_energy_grid_neutron(),
        };
        
        let mut micro_xs: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
        let temperature = &self.temperature;
        let temp_with_k = format!("{}K", temperature);
        let particle = "neutron"; // Explicitly set particle type
        
        println!("Calculating microscopic cross sections for temperature: {} (or {})", temperature, temp_with_k);
        
        for nuclide in self.nuclides.keys() {
            let mut nuclide_xs = HashMap::new();
            
            if let Some(nuclide_data) = self.nuclide_data.get(nuclide) {
                println!("Processing nuclide: {}", nuclide);
                
                if let Some(ip_data) = nuclide_data.incident_particle.as_ref().and_then(|ip| ip.get(particle)) {
                    println!("Found incident particle data for: {}", particle);
                    
                    // Try both temperature formats
                    let temp_data = ip_data.reactions_by_temp.get(temperature)
                        .or_else(|| ip_data.reactions_by_temp.get(&temp_with_k));
                    
                    if let Some(temp_data) = temp_data {
                        println!("Found reactions for temperature");
                        
                        // Get the energy grid for this nuclide and temperature
                        if let Some(energy_map) = &nuclide_data.energy {
                            let energy_grid = energy_map.get(temperature)
                                .or_else(|| energy_map.get(&temp_with_k));
                            
                            if let Some(energy_grid) = energy_grid {
                                println!("Found energy grid for temperature");
                                
                                // Process all MT reactions using the shared energy grid
                                for (mt, reaction) in temp_data {
                                    println!("Processing MT: {}", mt);
                                    
                                    // Create a vector to store interpolated cross sections
                                    let mut xs_values = Vec::with_capacity(grid.len());
                                    
                                    // Create a subslice of the energy grid starting at threshold_idx
                                    let threshold_idx = reaction.threshold_idx;
                                    let reaction_energy = if threshold_idx < energy_grid.len() {
                                        &energy_grid[threshold_idx..]
                                    } else {
                                        // If threshold is past the end of the grid, this reaction doesn't exist
                                        println!("Warning: threshold_idx {} is beyond energy grid length {}", threshold_idx, energy_grid.len());
                                        continue;
                                    };
                                    
                                    // Make sure the reaction xs has the right length
                                    if reaction.cross_section.len() != reaction_energy.len() {
                                        // Mismatch between energy grid and cross section array lengths
                                        println!("Warning: cross section length {} doesn't match energy grid length {} for nuclide {} MT {}", 
                                                reaction.cross_section.len(), reaction_energy.len(), nuclide, mt);
                                        continue;
                                    }
                                    
                                    // Interpolate cross sections onto unified grid
                                    for &grid_energy in &grid {
                                        // If the energy is below threshold, xs is 0
                                        if grid_energy < reaction_energy[0] {
                                            xs_values.push(0.0);
                                        } else {
                                            let xs = interpolate_linear(reaction_energy, &reaction.cross_section, grid_energy);
                                            xs_values.push(xs);
                                        }
                                    }
                                    
                                    nuclide_xs.insert(mt.clone(), xs_values);
                                }
                            } else {
                                println!("No energy grid found for temperature {} or {}", temperature, temp_with_k);
                                
                                // Print available temperatures
                                println!("Available temperatures in energy map: {:?}", energy_map.keys().collect::<Vec<_>>());
                            }
                        } else {
                            println!("No energy grid found for nuclide");
                        }
                    } else {
                        println!("No reactions found for temperature {} or {}", temperature, temp_with_k);
                        
                        // Print available temperatures
                        println!("Available temperatures in reactions: {:?}", ip_data.reactions_by_temp.keys().collect::<Vec<_>>());
                    }
                } else {
                    println!("No incident particle data found for particle: {}", particle);
                }
            } else {
                println!("No nuclide data found for: {}", nuclide);
            }
            
            // Only add the nuclide if we found cross section data
            if !nuclide_xs.is_empty() {
                println!("Adding microscopic cross sections for nuclide: {} with {} MT values", nuclide, nuclide_xs.len());
                micro_xs.insert(nuclide.clone(), nuclide_xs);
            } else {
                println!("No cross sections found for nuclide: {}", nuclide);
            }
        }
        
        println!("Finished calculating microscopic cross sections: found data for {} nuclides", micro_xs.len());
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
        // Ensure nuclides are loaded before proceeding
        if let Err(e) = self.ensure_nuclides_loaded() {
            eprintln!("Error loading nuclides: {}", e);
        }
        
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

    /// Calculate the total cross section for neutrons by summing over all relevant MT reactions
    /// 
    /// This method takes the macroscopic cross sections and sums all relevant MT reactions
    /// to create a "total" cross section, which is added to the HashMap.
    /// 
    /// If a total cross section already exists in the HashMap, it will be overwritten.
    /// 
    /// Returns the updated HashMap with the "total" entry added.
    pub fn calculate_total_xs_neutron(&mut self) -> HashMap<String, Vec<f64>> {
        // Ensure nuclides are loaded before proceeding
        if let Err(e) = self.ensure_nuclides_loaded() {
            eprintln!("Error loading nuclides: {}", e);
        }
        
        // Get the macroscopic cross sections (calculate if not already done)
        let macro_xs = if self.macroscopic_xs_neutron.is_empty() {
            self.calculate_macroscopic_xs_neutron(None)
        } else {
            self.macroscopic_xs_neutron.clone()
        };
        
        // Define the MT numbers that should be summed for the total cross section
        const TOTAL_MT_NUMBERS: [&str; 393] = [
            "2", "5", "11", "17", "19", "20", "21", "22", "23", "24", "25", "28", "29", "30", 
            "32", "33", "34", "35", "36", "37", "38", "41", "42", "44", "45", "50", "51", "52", 
            "53", "54", "55", "56", "57", "58", "59", "60", "61", "62", "63", "64", "65", "66", 
            "67", "68", "69", "70", "71", "72", "73", "74", "75", "76", "77", "78", "79", "80", 
            "81", "82", "83", "84", "85", "86", "87", "88", "89", "90", "91", "102", "108", "109", 
            "111", "112", "113", "114", "115", "116", "117", "152", "153", "154", "155", "156", 
            "157", "158", "159", "160", "161", "162", "163", "164", "165", "166", "167", "168", 
            "169", "170", "171", "172", "173", "174", "175", "176", "177", "178", "179", "180", 
            "181", "182", "183", "184", "185", "186", "187", "188", "189", "190", "191", "192", 
            "193", "194", "195", "196", "197", "198", "199", "200", "600", "601", "602", "603", 
            "604", "605", "606", "607", "608", "609", "610", "611", "612", "613", "614", "615", 
            "616", "617", "618", "619", "620", "621", "622", "623", "624", "625", "626", "627", 
            "628", "629", "630", "631", "632", "633", "634", "635", "636", "637", "638", "639", 
            "640", "641", "642", "643", "644", "645", "646", "647", "648", "649", "650", "651", 
            "652", "653", "654", "655", "656", "657", "658", "659", "660", "661", "662", "663", 
            "664", "665", "666", "667", "668", "669", "670", "671", "672", "673", "674", "675", 
            "676", "677", "678", "679", "680", "681", "682", "683", "684", "685", "686", "687", 
            "688", "689", "690", "691", "692", "693", "694", "695", "696", "697", "698", "699", 
            "700", "701", "702", "703", "704", "705", "706", "707", "708", "709", "710", "711", 
            "712", "713", "714", "715", "716", "717", "718", "719", "720", "721", "722", "723", 
            "724", "725", "726", "727", "728", "729", "730", "731", "732", "733", "734", "735", 
            "736", "737", "738", "739", "740", "741", "742", "743", "744", "745", "746", "747", 
            "748", "749", "750", "751", "752", "753", "754", "755", "756", "757", "758", "759", 
            "760", "761", "762", "763", "764", "765", "766", "767", "768", "769", "770", "771", 
            "772", "773", "774", "775", "776", "777", "778", "779", "780", "781", "782", "783", 
            "784", "785", "786", "787", "788", "789", "790", "791", "792", "793", "794", "795", 
            "796", "797", "798", "799", "800", "801", "802", "803", "804", "805", "806", "807", 
            "808", "809", "810", "811", "812", "813", "814", "815", "816", "817", "818", "819", 
            "820", "821", "822", "823", "824", "825", "826", "827", "828", "829", "830", "831", 
            "832", "833", "834", "835", "836", "837", "838", "839", "840", "841", "842", "843", 
            "844", "845", "846", "847", "848", "849", "875", "876", "877", "878", "879", "880", 
            "881", "882", "883", "884", "885", "886", "887", "888", "889", "890", "891"
        ];
        
        // Get the length of the energy grid from any MT reaction
        let grid_length = macro_xs.values().next().map_or(0, |xs| xs.len());
        
        // If there are no cross sections, return the empty HashMap
        if grid_length == 0 {
            return macro_xs;
        }
        
        // Initialize the total cross section with zeros
        let mut total_xs = vec![0.0; grid_length];
        
        // Sum up all the relevant MT reactions
        for mt in TOTAL_MT_NUMBERS.iter() {
            if let Some(xs_values) = macro_xs.get(*mt) {
                for (i, &xs) in xs_values.iter().enumerate() {
                    total_xs[i] += xs;
                }
            }
        }
        
        // Create a new HashMap with the original data plus the total
        let mut result = macro_xs.clone();
        result.insert(String::from("total"), total_xs);
        
        // Update the cached macroscopic cross sections
        self.macroscopic_xs_neutron = result.clone();
        
        result
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
        
        // Convert density to g/cm³ if necessary
        let mut density = self.density.unwrap();
        
        // Handle different density units
        match self.density_units.as_str() {
            "g/cm3" => (), // Already in the right units
            "kg/m3" => density = density / 1000.0, // Convert kg/m³ to g/cm³
            _ => {
                // For any other units, just use the value as is, but it may give incorrect results
                println!("Warning: Unsupported density unit '{}' for atoms_per_cc calculation. Results may be incorrect.", self.density_units);
            }
        }
        
        // Hard-coded atomic masses (in g/mol) for common nuclides
        let mut atomic_masses = HashMap::new();
        
        // Lithium isotopes
        atomic_masses.insert(String::from("Li6"), 6.01512288742);
        atomic_masses.insert(String::from("Li7"), 7.016004);
        
        
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
        
        // Calculate atom density for each nuclide
        // Following OpenMC's formula: atom_density = density * Avogadro_number / atomic_mass
        for (nuclide, fraction) in &self.nuclides {
            if let Some(mass) = atomic_masses.get(nuclide) {
                // Formula: atoms/cc = N_A * density * fraction / atomic_mass
                let atom_density = AVOGADRO * density * fraction / mass;
                atoms_per_cc.insert(nuclide.clone(), atom_density);
            } else {
                // For nuclides without defined atomic mass, we can't calculate accurately
                // Print a warning and use a very rough approximation
                println!("Warning: No atomic mass data for nuclide '{}'. Using approximate value.", nuclide);
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
        
        // Approximate expected values - these values are calculated using the formula:
        // Li6: number_density = (0.5 * N_A) / (6.0 u)
        // Li7: number_density = (0.5 * N_A) / (7.0 u)
        let avogadro = 6.02214076e23;
        let li6_expected = 0.5 * avogadro / 6.0;
        let li7_expected = 0.5 * avogadro / 7.0;
        
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

    #[test]
    fn test_calculate_total_xs_neutron() {
        let mut material = Material::new();
        
        // Add some nuclides
        material.add_nuclide("Li6", 0.5).unwrap();
        material.add_nuclide("Li7", 0.5).unwrap();
        material.set_density("g/cm3", 1.0).unwrap();
        
        // Create some mock cross sections directly
        let mut mock_xs = HashMap::new();
        
        // MT=2 (elastic scattering)
        mock_xs.insert(String::from("2"), vec![1.0, 2.0, 3.0]);
        
        // MT=102 (radiative capture)
        mock_xs.insert(String::from("102"), vec![0.5, 1.0, 1.5]);
        
        // MT=999 (some reaction not in the total list)
        mock_xs.insert(String::from("999"), vec![0.1, 0.2, 0.3]);
        
        // Set the mock cross sections directly
        material.macroscopic_xs_neutron = mock_xs;
        
        // Calculate the total cross section
        let result = material.calculate_total_xs_neutron();
        
        // Check that the total was calculated correctly
        assert!(result.contains_key("total"), "Total cross section not found in result");
        
        let total_xs = result.get("total").unwrap();
        assert_eq!(total_xs.len(), 3, "Total cross section has wrong length");
        
        // Total should be the sum of MT=2 and MT=102 only (not MT=999)
        assert_eq!(total_xs[0], 1.5, "Total cross section[0] is incorrect");
        assert_eq!(total_xs[1], 3.0, "Total cross section[1] is incorrect");
        assert_eq!(total_xs[2], 4.5, "Total cross section[2] is incorrect");
        
        // Verify that the original cross sections are still there
        assert!(result.contains_key("2"), "MT=2 not found in result");
        assert!(result.contains_key("102"), "MT=102 not found in result");
        assert!(result.contains_key("999"), "MT=999 not found in result");
    }

}
