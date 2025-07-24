use std::collections::HashMap;
use std::sync::Arc;
use crate::nuclide::{Nuclide, get_or_load_nuclide};
use crate::config::CONFIG;
use crate::utilities::interpolate_linear;
use crate::data::{ELEMENT_NAMES, get_all_mt_descendants};
use crate::data::SUM_RULES;

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
            // Load from the provided JSON path map
            match get_or_load_nuclide(&nuclide_name, &config.cross_sections) {
                Ok(nuclide) => {
                    self.nuclide_data.insert(nuclide_name.clone(), nuclide);
                },
                Err(e) => {
                    return Err(format!("Failed to load nuclide '{}': {}", nuclide_name, e).into());
                }
            }
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
            panic!("Error loading nuclides: {}", e);
        }
        
        // Check if we already have this grid in the cache
        if !self.unified_energy_grid_neutron.is_empty() {
            return self.unified_energy_grid_neutron.clone();
        }
        
        // If not cached, build the grid
        let mut all_energies = Vec::new();
        let temperature = &self.temperature;
        let _particle = "neutron"; // This is now specifically for neutrons

        for nuclide in self.nuclides.keys() {
            if let Some(nuclide_data) = self.nuclide_data.get(nuclide) {
                // Check if there's a top-level energy grid
                if let Some(energy_map) = &nuclide_data.energy {

                    if let Some(energy_grid) = energy_map.get(temperature) {
                        all_energies.extend(energy_grid);
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
    /// onto the unified energy grid for all available MT reactions, or only for the specified MTs if provided.
    /// If unified_energy_grid is None, it will use the cached grid or build a new one.
    /// If mt_filter is Some, only those MTs will be included (by string match).
    /// Returns a nested HashMap: nuclide -> mt -> cross_section values
    pub fn calculate_microscopic_xs_neutron(
        &mut self,
        mt_filter: Option<&Vec<String>>,
    ) -> HashMap<String, HashMap<String, Vec<f64>>> {
        // Ensure nuclides are loaded before proceeding
        if let Err(e) = self.ensure_nuclides_loaded() {
            panic!("Error loading nuclides: {}", e);
        }
        // Always use the cached grid or build it automatically
        let grid = self.unified_energy_grid_neutron();
        let mut micro_xs: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
        let temperature = &self.temperature;
        let temp_with_k = format!("{}K", temperature);
        for nuclide in self.nuclides.keys() {
            let mut nuclide_xs = HashMap::new();
            if let Some(nuclide_data) = self.nuclide_data.get(nuclide) {
                let temp_reactions = nuclide_data.reactions.get(temperature)
                    .or_else(|| nuclide_data.reactions.get(&temp_with_k));
                if let Some(temp_reactions) = temp_reactions {
                    if let Some(energy_map) = &nuclide_data.energy {
                        let energy_grid = energy_map.get(temperature)
                            .or_else(|| energy_map.get(&temp_with_k));
                        if let Some(energy_grid) = energy_grid {
                            // Only process the requested MTs if mt_filter is Some
                            let mt_set: Option<std::collections::HashSet<&String>> = mt_filter.map(|v| v.iter().collect());
                            let mt_iter = temp_reactions.iter().filter(|(k, _)| {
                                match &mt_set {
                                    Some(set) => set.contains(k),
                                    None => true,
                                }
                            });
                            for (mt, reaction) in mt_iter {
                                let mut xs_values = Vec::with_capacity(grid.len());
                                let threshold_idx = reaction.threshold_idx;
                                let reaction_energy = if threshold_idx < energy_grid.len() {
                                    &energy_grid[threshold_idx..]
                                } else {
                                    continue;
                                };
                                if reaction.cross_section.len() != reaction_energy.len() {
                                    continue;
                                }
                                for &grid_energy in &grid {
                                    if grid_energy < reaction_energy[0] {
                                        xs_values.push(0.0);
                                    } else {
                                        let xs = interpolate_linear(reaction_energy, &reaction.cross_section, grid_energy);
                                        xs_values.push(xs);
                                    }
                                }
                                nuclide_xs.insert(mt.clone(), xs_values);
                            }
                        }
                    }
                }
            }
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
    /// If mt_filter is Some, only those MTs will be included (by string match).
    pub fn calculate_macroscopic_xs_neutron(
        &mut self,
        mt_filter: Option<&Vec<String>>,
    ) -> (Vec<f64>, HashMap<String, Vec<f64>>) {
        // Ensure nuclides are loaded before proceeding
        if let Err(e) = self.ensure_nuclides_loaded() {
            panic!("Error loading nuclides: {}", e);
        }
        // Get the energy grid
        let energy_grid = self.unified_energy_grid_neutron();
        // Expand the filter to include all child MTs for any hierarchical MTs
        // Helper to recursively collect all descendants for a given MT number
        // Use public get_all_mt_descendants from data.rs
        let expanded_filter = if let Some(filter) = mt_filter {
            let mut expanded: std::collections::HashSet<i32> = std::collections::HashSet::new();
            for mt in filter {
                if let Ok(mt_num) = mt.parse::<i32>() {
                    expanded.insert(mt_num);
                    for child in get_all_mt_descendants(mt_num) {
                        expanded.insert(child);
                    }
                }
            }
            Some(expanded.into_iter().map(|mt| mt.to_string()).collect::<Vec<_>>())
        } else {
            None
        };
        // First get microscopic cross sections on the unified grid, passing expanded_filter
        let micro_xs = self.calculate_microscopic_xs_neutron(expanded_filter.as_ref());
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
        let atoms_per_bcm_map = self.get_atoms_per_cc();
        for (nuclide, _) in &self.nuclides {
            if let Some(nuclide_data) = micro_xs.get(nuclide) {
                if let Some(atoms_per_bcm) = atoms_per_bcm_map.get(nuclide) {
                    // Add contribution to macroscopic cross section for each MT
                    for (mt, xs_values) in nuclide_data {
                        if let Some(macro_values) = macro_xs.get_mut(&mt.to_string()) {
                            for (i, &xs) in xs_values.iter().enumerate() {
                                macro_values[i] += atoms_per_bcm * xs;
                            }
                        }
                    }
                }
            }
        }
        // Cache the results in the material
        self.macroscopic_xs_neutron = macro_xs.clone();
        // If calculating all MTs, ensure all hierarchical MT numbers
        if mt_filter.is_none() {
            self.ensure_hierarchical_mt_numbers();
            return (energy_grid, self.macroscopic_xs_neutron.clone());
        } else if let Some(filter) = mt_filter {
            // Always ensure hierarchical MT numbers are generated
            self.ensure_hierarchical_mt_numbers();
            for mt in filter {
                // Always copy the parent MT from self.macroscopic_xs_neutron if present
                if let Some(xs) = self.macroscopic_xs_neutron.get(mt) {
                    macro_xs.insert(mt.clone(), xs.clone());
                }
                // If the parent is a hierarchical MT, also include all its descendants recursively
                if let Ok(mt_num) = mt.parse::<i32>() {
                    for child in get_all_mt_descendants(mt_num) {
                        let child_str = child.to_string();
                        if let Some(child_xs) = self.macroscopic_xs_neutron.get(&child_str) {
                            macro_xs.insert(child_str, child_xs.clone());
                        }
                    }
                }
            }
        }
        (energy_grid, macro_xs)
    }

    /// Calculate hierarchical MT numbers when they are missing from the original data
    /// 
    /// This ensures that higher-level MT numbers (e.g., MT=3, MT=4) are available
    /// by summing their constituent reactions according to OpenMC's sum rules.
    /// 
    /// If a hierarchical MT number is already present in the cross section data,
    /// it will not be recalculated or overwritten.
    /// 
    /// This implementation processes MT numbers in dependency order (bottom-up),
    /// ensuring that all constituents are calculated before their parents.
    /// TODO To maximize performance for filtered requests, you could modify
    // ensure_hierarchical_mt_numbers to only generate hierarchical MTs that
    // are needed for the current filter (and their parents, if required),
    // instead of all possible hierarchical MTs use this in calculate_macroscopic_xs_neutron
    // when passing in a reaction
    pub fn ensure_hierarchical_mt_numbers(&mut self) {
        // Get a mutable reference to the macroscopic cross sections
        let xs_map = &mut self.macroscopic_xs_neutron;
        
        // Skip if there are no cross sections
        if xs_map.is_empty() {
            return;
        }
        
        // Use the OpenMC sum rules for hierarchical MT numbers from data.rs
        let sum_rules = &*SUM_RULES;
        
        // Get the length of the energy grid from any MT reaction
        let grid_length = xs_map.values().next().map_or(0, |xs| xs.len());
        if grid_length == 0 {
            return;
        }
        
        // Create a dependency graph and processing order
        // We need to process MT numbers in dependency order (leaf nodes first)
        let mut processing_order = Vec::new();
        let mut processed_set = std::collections::HashSet::new();
        
        // First add all leaf MT numbers (those that don't appear as parents in sum_rules)
        // and those that already exist in xs_map
        for mt_str in xs_map.keys() {
            if let Ok(mt) = mt_str.parse::<i32>() {
                processed_set.insert(mt);
            }
        }
        
        // Helper function to add an MT and its dependencies to the processing order
        fn add_to_processing_order(
            mt: i32, 
            sum_rules: &std::collections::HashMap<i32, Vec<i32>>,
            processed: &mut std::collections::HashSet<i32>,
            order: &mut Vec<i32>
        ) {
            // Skip if already processed
            if processed.contains(&mt) {
                return;
            }
            
            // Process dependencies first (if this MT is in the sum rules)
            if let Some(constituents) = sum_rules.get(&mt) {
                for &constituent in constituents {
                    add_to_processing_order(constituent, sum_rules, processed, order);
                }
            }
            
            // Now add this MT
            processed.insert(mt);
            order.push(mt);
        }
        
        // Add MTs to processing order in bottom-up dependency order
        for &mt in sum_rules.keys() {
            add_to_processing_order(mt, sum_rules, &mut processed_set, &mut processing_order);
        }
        
        // Now process in the determined order
        for mt in processing_order {
            let mt_str = mt.to_string();
            // Skip if this MT already exists
            if xs_map.contains_key(&mt_str) {
                continue;
            }
            // Skip if this MT is not in the sum rules
            if !sum_rules.contains_key(&mt) {
                continue;
            }
            // Initialize a vector for this MT with zeros
            let mut mt_xs = vec![0.0; grid_length];
            let mut found_any_child = false;
            // Sum only the available constituent MT numbers
            for &constituent_mt in &sum_rules[&mt] {
                let constituent_mt_str = constituent_mt.to_string();
                if let Some(xs_values) = xs_map.get(&constituent_mt_str) {
                    for (i, &xs) in xs_values.iter().enumerate() {
                        mt_xs[i] += xs;
                    }
                    found_any_child = true;
                }
            }
            // Add this MT if at least one child was found (even if not all)
            if found_any_child {
                xs_map.insert(mt_str, mt_xs);
            }
        }
    }
    
    /// Calculate the total cross section for neutrons using MT=2 + MT=3
    /// 
    /// This method follows OpenMC's approach of calculating the total cross section
    /// as the sum of elastic (MT=2) and non-elastic (MT=3) cross sections.
    /// 
    /// First ensures that MT=3 is calculated from its constituents if not already present.
    /// 
    /// If a total cross section already exists in the HashMap, it will
    /// automatically be updated with the new calculation.
    /// 
    /// Returns the updated HashMap with the "total" entry added.
    pub fn calculate_total_xs_neutron(&mut self) -> HashMap<String, Vec<f64>> {
        // Ensure nuclides are loaded before proceeding
        if let Err(e) = self.ensure_nuclides_loaded() {
            panic!("Error loading nuclides: {}", e);
        }
        // Get the macroscopic cross sections (calculate if not already done)
        let (energy_grid, macro_xs) = if self.macroscopic_xs_neutron.is_empty() {
            self.calculate_macroscopic_xs_neutron(None)
        } else {
            (self.unified_energy_grid_neutron.clone(), self.macroscopic_xs_neutron.clone())
        };
        // Get the length of the energy grid from any MT reaction
        let grid_length = macro_xs.values().next().map_or(0, |xs| xs.len());
        // If there are no cross sections, return the empty HashMap
        if grid_length == 0 {
            return macro_xs;
        }
        // Initialize the total cross section with zeros
        let mut total_xs = vec![0.0; grid_length];
        // According to OpenMC, total (MT=1) = elastic (MT=2) + non-elastic (MT=3)
        // Add MT=2 (elastic scattering)
        let mut has_mt2 = false;
        if let Some(mt2_xs) = macro_xs.get("2") {
            for (i, &xs) in mt2_xs.iter().enumerate() {
                total_xs[i] += xs;
            }
            has_mt2 = true;
        }
        // Add MT=3 (non-elastic)
        let mut has_mt3 = false;
        if let Some(mt3_xs) = macro_xs.get("3") {
            for (i, &xs) in mt3_xs.iter().enumerate() {
                total_xs[i] += xs;
            }
            has_mt3 = true;
        }
        // Create a new HashMap with the original data plus the total
        let mut result = macro_xs.clone();
        // Only add the total if we found at least one of MT=2 or MT=3
        if has_mt2 || has_mt3 {
            result.insert(String::from("total"), total_xs);
        }
        // Update the cached macroscopic cross sections
        self.macroscopic_xs_neutron = result.clone();
        result
    }

    /// Calculate the neutron mean free path at a given energy
    /// 
    /// This method calculates the mean free path of a neutron at a specific energy
    /// by interpolating the total macroscopic cross section and then taking 1/Σ.
    /// 
    /// If the total macroscopic cross section hasn't been calculated yet, it will
    /// automatically call calculate_total_xs_neutron() first.
    /// 
    /// # Arguments
    /// * `energy` - The energy of the neutron in eV
    /// 
    /// # Returns
    /// * The mean free path in cm, or None if there's no cross section data
    pub fn mean_free_path_neutron(&mut self, energy: f64) -> Option<f64> {
        // Ensure we have a total cross section
        if !self.macroscopic_xs_neutron.contains_key("total") {
            self.calculate_total_xs_neutron();
        }
        
        // If we still don't have a total cross section, return None
        if !self.macroscopic_xs_neutron.contains_key("total") {
            return None;
        }
        
        // Get the total cross section and energy grid
        let total_xs = &self.macroscopic_xs_neutron["total"];
        
        // If we have an empty cross section array, return None
        if total_xs.is_empty() || self.unified_energy_grid_neutron.is_empty() {
            return None;
        }
        
        // Make sure the energy grid and cross section have the same length
        if total_xs.len() != self.unified_energy_grid_neutron.len() {
            eprintln!("Error: Energy grid and cross section lengths don't match");
            return None;
        }
        
        // Interpolate to get the cross section at the requested energy
        // Using linear-linear interpolation
        let cross_section = interpolate_linear(
            &self.unified_energy_grid_neutron, 
            total_xs, 
            energy
        );
        
        // Mean free path = 1/Σ
        // Check for zero to avoid division by zero
        if cross_section <= 0.0 {
            None
        } else {
            Some(1.0 / cross_section)
        }
    }

    /// Calculate atoms per cubic centimeter for each nuclide in the material
    /// 
    /// This method calculates the number density of atoms for each nuclide,
    /// using the atomic fractions and material density.
    /// 
    /// Returns a HashMap mapping nuclide symbols to their atom density in atoms/b-cm,
    /// which is the unit used by OpenMC (atoms per barn-centimeter).
    /// Returns an empty HashMap if the material density is not set.
    pub fn get_atoms_per_cc(&self) -> HashMap<String, f64> {
        let mut atoms_per_cc = HashMap::new();
        
        // Return empty HashMap if density is not set
        if self.density.is_none() {
            panic!("Cannot calculate atoms per cc: Material has no density defined");
        }
        
        // Return empty HashMap if no nuclides are defined
        if self.nuclides.is_empty() {
            panic!("Cannot calculate atoms per cc: Material has no nuclides defined");
        }
        
        // Convert density to g/cm³ if necessary
        let mut density = self.density.unwrap();
        
        // Handle different density units
        match self.density_units.as_str() {
            "g/cm3" => (), // Already in the right units
            "kg/m3" => density = density / 1000.0, // Convert kg/m³ to g/cm³
            _ => {
                // For any other units, just use the value as is, but it may give incorrect results
            }
        }
        
        // Use canonical atomic masses from crate::data::ATOMIC_MASSES
        let atomic_masses = &crate::data::ATOMIC_MASSES;
        // First, get the atomic masses for all nuclides
        let mut nuclide_masses = HashMap::new();
        for (nuclide, _) in &self.nuclides {
            let mass = if let Some(mass_value) = atomic_masses.get(nuclide.as_str()) {
                *mass_value
            } else {
                panic!("Atomic mass for nuclide '{}' not found in the database", nuclide);
            };
            nuclide_masses.insert(nuclide.clone(), mass);
        }
        
        // Normalize the fractions to sum to 1.0 for all cases
        let total_fraction: f64 = self.nuclides.values().sum();
        
        // Calculate the average molar mass (weighted)
        let mut weighted_mass_sum = 0.0;
        for (nuclide, &fraction) in &self.nuclides {
            let mass = nuclide_masses.get(nuclide).unwrap();
            weighted_mass_sum += fraction * mass;
        }
        let average_molar_mass = weighted_mass_sum / total_fraction;
        
        // Calculate atom densities using OpenMC's approach
        for (nuclide, &fraction) in &self.nuclides {
            let normalized_fraction = fraction / total_fraction;
            let mass = nuclide_masses.get(nuclide).unwrap();
            
            // For a mixture, use the formula:
            // atom_density = density * N_A / avg_molar_mass * normalized_fraction * 1e-24
            const AVOGADRO: f64 = 6.02214076e23;
            let atom_density = density * AVOGADRO / average_molar_mass * normalized_fraction * 1.0e-24;
            
            atoms_per_cc.insert(nuclide.clone(), atom_density);
        }
        
        atoms_per_cc
    }

    pub fn add_element(&mut self, element: &str, fraction: f64) -> Result<(), String> {
        if fraction <= 0.0 {
            return Err(String::from("Fraction must be positive"));
        }

        // Canonicalize input: trim only (do not lowercase or otherwise change user input)
        let input = element.trim();

        // Try to match as symbol (case-sensitive, exact match)
        let mut found_symbol: Option<String> = None;
        for (symbol, name) in ELEMENT_NAMES.iter() {
            if *symbol == input {
                found_symbol = Some(symbol.to_string());
                break;
            }
        }
        // If not found as symbol, try to match as name (case-sensitive, exact match)
        if found_symbol.is_none() {
            for (symbol, name) in ELEMENT_NAMES.iter() {
                if *name == input {
                    found_symbol = Some(symbol.to_string());
                    break;
                }
            }
        }
        let element_sym = match found_symbol {
            Some(sym) => sym,
            None => {
                return Err(format!(
                    "Element '{}' is not a recognized element symbol or name (case-sensitive, must match exactly)",
                    element
                ));
            }
        };

        // Get the isotopes for this element
        let element_isotopes = crate::element::get_element_isotopes();

        // Check if the element exists in our database
        let isotopes = element_isotopes.get(element_sym.as_str()).ok_or_else(|| {
            format!("Element '{}' not found in the natural abundance database", element_sym)
        })?;

        // Add each isotope with its natural abundance
        for &isotope in isotopes {
            let abundance = crate::data::NATURAL_ABUNDANCE.get(isotope).unwrap();
            let isotope_fraction = fraction * abundance;
            // Only add isotopes with non-zero fractions
            if isotope_fraction > 0.0 {
                self.add_nuclide(isotope, isotope_fraction)?;
            }
        }
        Ok(())
    }
    
    /// Returns a sorted list of all unique MT numbers available in this material (across all nuclides).
    /// Ensures all nuclide JSON data is loaded.
    pub fn reaction_mts(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Ensure all nuclides are loaded using the global config
        self.ensure_nuclides_loaded()?;
        let mut mt_set = std::collections::HashSet::new();
        for nuclide in self.nuclide_data.values() {
            if let Some(mts) = nuclide.reaction_mts() {
                for mt in mts {
                    mt_set.insert(mt);
                }
            }
        }
        let mut mt_vec: Vec<String> = mt_set.into_iter().collect();
        mt_vec.sort();
        Ok(mt_vec)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hierarchical_mt3_generated_for_li6() {
        use std::collections::HashMap;
        let mut material = Material::new();
        material.add_nuclide("Li6", 1.0).unwrap();
        material.set_density("g/cm3", 0.534).unwrap();
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        // Request only MT=3 (not present in Li6.json, but hierarchical)
        let mt_filter = vec!["3".to_string()];
        let (_grid, macro_xs) = material.calculate_macroscopic_xs_neutron(Some(&mt_filter));
        println!("DEBUG: macro_xs keys: {:?}", macro_xs.keys().collect::<Vec<_>>());
        println!("DEBUG: self.macroscopic_xs_neutron keys: {:?}", material.macroscopic_xs_neutron.keys().collect::<Vec<_>>());
        assert!(macro_xs.contains_key("3"), "MT=3 should be generated by sum rule for Li6");
        // Optionally check that the cross section is the sum of all descendants (recursive)
        let children = get_all_mt_descendants(3);
        let mut sum = vec![0.0; macro_xs["3"].len()];
        for child in children {
            let child_str = child.to_string();
            if let Some(xs) = macro_xs.get(&child_str) {
                for (i, v) in xs.iter().enumerate() {
                    sum[i] += v;
                }
            }
        }
        let tol = 1e-10;
        for (a, b) in macro_xs["3"].iter().zip(sum.iter()) {
            assert!((a - b).abs() < tol, "MT=3 cross section should equal sum of descendants");
        }
    }
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
        let material = Material::new();
        
        // Test with no density set - should panic
        let result = std::panic::catch_unwind(|| {
            material.get_atoms_per_cc()
        });
        assert!(result.is_err(), "Should panic when density is not set");
        
        // Test with single nuclide case
        let mut material_single = Material::new();
        material_single.add_nuclide("Li6", 2.5).unwrap();
        material_single.set_density("g/cm3", 1.0).unwrap();
        
        let atoms_single = material_single.get_atoms_per_cc();
        assert_eq!(atoms_single.len(), 1, "Should have 1 nuclide in the HashMap");
        
        // For a single nuclide, we normalize the fraction to 1.0
        let avogadro = 6.02214076e23;
        let li6_mass = 6.01512288742;
        let li6_expected = 1.0 * avogadro / li6_mass * 1.0e-24; // Fraction is normalized to 1.0
        
        let li6_actual = atoms_single.get("Li6").unwrap();
        let tolerance = 0.01; // 1%
        
        assert!((li6_actual - li6_expected).abs() / li6_expected < tolerance, 
            "Li6 atoms/cc calculation (single nuclide) is incorrect: got {}, expected {}", 
            li6_actual, li6_expected);
        
        // Test with multiple nuclides
        let mut material_multi = Material::new();
        material_multi.add_nuclide("Li6", 0.5).unwrap();
        material_multi.add_nuclide("Li7", 0.5).unwrap();
        material_multi.set_density("g/cm3", 1.0).unwrap();
        
        let atoms_multi = material_multi.get_atoms_per_cc();
        assert_eq!(atoms_multi.len(), 2, "Should have 2 nuclides in the HashMap");
        
        // For multiple nuclides, the fractions are normalized and used with average molar mass
        let li7_mass = 7.016004;
        let avg_mass = (0.5 * li6_mass + 0.5 * li7_mass) / 1.0; // weighted average
        
        let li6_expected_multi = 1.0 * avogadro / avg_mass * (0.5 / 1.0) * 1.0e-24;
        let li7_expected_multi = 1.0 * avogadro / avg_mass * (0.5 / 1.0) * 1.0e-24;
        
        let li6_actual_multi = atoms_multi.get("Li6").unwrap();
        let li7_actual_multi = atoms_multi.get("Li7").unwrap();
        
        assert!((li6_actual_multi - li6_expected_multi).abs() / li6_expected_multi < tolerance, 
            "Li6 atoms/cc calculation (multiple nuclides) is incorrect: got {}, expected {}", 
            li6_actual_multi, li6_expected_multi);
        assert!((li7_actual_multi - li7_expected_multi).abs() / li7_expected_multi < tolerance, 
            "Li7 atoms/cc calculation (multiple nuclides) is incorrect: got {}, expected {}", 
            li7_actual_multi, li7_expected_multi);
        
        // Test with non-normalized fractions
        let mut material_non_norm = Material::new();
        material_non_norm.add_nuclide("Li6", 1.0).unwrap();
        material_non_norm.add_nuclide("Li7", 1.0).unwrap(); // Total fractions = 2.0
        material_non_norm.set_density("g/cm3", 1.0).unwrap();
        
        let atoms_non_norm = material_non_norm.get_atoms_per_cc();
        
        // Fractions should be normalized to 0.5 each (1.0/2.0)
        let avg_mass_non_norm = (1.0 * li6_mass + 1.0 * li7_mass) / 2.0;
        let li6_expected_non_norm = 1.0 * avogadro / avg_mass_non_norm * (1.0 / 2.0) * 1.0e-24;
        let li7_expected_non_norm = 1.0 * avogadro / avg_mass_non_norm * (1.0 / 2.0) * 1.0e-24;
        
        let li6_actual_non_norm = atoms_non_norm.get("Li6").unwrap();
        let li7_actual_non_norm = atoms_non_norm.get("Li7").unwrap();
        
        assert!((li6_actual_non_norm - li6_expected_non_norm).abs() / li6_expected_non_norm < tolerance, 
            "Li6 normalized atoms/cc calculation is incorrect: got {}, expected {}", 
            li6_actual_non_norm, li6_expected_non_norm);
        assert!((li7_actual_non_norm - li7_expected_non_norm).abs() / li7_expected_non_norm < tolerance, 
            "Li7 normalized atoms/cc calculation is incorrect: got {}, expected {}", 
            li7_actual_non_norm, li7_expected_non_norm);
    }

    #[test]
    fn test_get_atoms_per_cc_no_density() {
        let material = Material::new();
        
        // Should panic when density is not set
        let result = std::panic::catch_unwind(|| {
            material.get_atoms_per_cc()
        });
        assert!(result.is_err(), "Should panic when density is not set");
        
        // Should also panic when nuclides are not added
        let mut material_with_density = Material::new();
        material_with_density.set_density("g/cm3", 1.0).unwrap();
        
        let result = std::panic::catch_unwind(|| {
            material_with_density.get_atoms_per_cc()
        });
        assert!(result.is_err(), "Should panic when no nuclides are defined");
    }

    #[test]
    fn test_calculate_total_xs_neutron() {
        let mut material = Material::new();
        
        // We don't need to actually load nuclides from JSON for this test,
        // we'll just use mock data
        
        // Create some mock cross sections directly
        let mut mock_xs = HashMap::new();
        
        // MT=2 (elastic scattering)
        mock_xs.insert(String::from("2"), vec![1.0, 2.0, 3.0]);
        
        // MT=4 (inelastic scattering - a component of MT=3)
        mock_xs.insert(String::from("4"), vec![0.2, 0.4, 0.6]);
        
        // MT=16 (n,2n - another component of MT=3)
        mock_xs.insert(String::from("16"), vec![0.3, 0.6, 0.9]);
        
        // MT=999 (some reaction not in the sum rules)
        mock_xs.insert(String::from("999"), vec![0.1, 0.2, 0.3]);
        
        // Set the mock cross sections directly
        material.macroscopic_xs_neutron = mock_xs.clone();
        material.unified_energy_grid_neutron = vec![1.0, 2.0, 3.0]; // Add a grid
        
        // Calculate hierarchical MT numbers first (MT=3 from components)
        material.ensure_hierarchical_mt_numbers();
        
        // Verify MT=3 was created correctly
        assert!(material.macroscopic_xs_neutron.contains_key("3"), "MT=3 not created");
        let mt3_xs = material.macroscopic_xs_neutron.get("3").unwrap();
        assert_eq!(mt3_xs.len(), 3, "MT=3 cross section has wrong length");
        assert_eq!(mt3_xs[0], 0.5, "MT=3[0] is incorrect"); // 0.2 + 0.3
        assert_eq!(mt3_xs[1], 1.0, "MT=3[1] is incorrect"); // 0.4 + 0.6
        assert_eq!(mt3_xs[2], 1.5, "MT=3[2] is incorrect"); // 0.6 + 0.9
        
        // Now calculate the total cross section
        let result = material.calculate_total_xs_neutron();
        
        // Check that the total was calculated correctly
        assert!(result.contains_key("total"), "Total cross section not found in result");
        
        // Total should be the sum of MT=2 and MT=3
        let total_xs = result.get("total").unwrap();
        assert_eq!(total_xs.len(), 3, "Total cross section has wrong length");
        assert_eq!(total_xs[0], 1.5, "Total cross section[0] is incorrect"); // 1.0 + 0.5
        assert_eq!(total_xs[1], 3.0, "Total cross section[1] is incorrect"); // 2.0 + 1.0
        assert_eq!(total_xs[2], 4.5, "Total cross section[2] is incorrect"); // 3.0 + 1.5
        
        // Verify that all cross sections are in the result
        assert!(result.contains_key("2"), "MT=2 not found in result");
        assert!(result.contains_key("3"), "MT=3 not found in result");
        assert!(result.contains_key("4"), "MT=4 not found in result");
        assert!(result.contains_key("16"), "MT=16 not found in result");
        assert!(result.contains_key("999"), "MT=999 not found in result");
    }

    #[test]
    fn test_mean_free_path_neutron() {
        // Create a properly set up material
        let mut material = Material::new();
        
        // Test with empty material - should return None
        // We can't use catch_unwind easily with &mut material, so we'll bypass the part that panics
        // by directly setting up the test with mock data
        
        // Skip the initial test with empty material that would cause a panic
        
        // Add a nuclide and set density
        material.add_nuclide("Li6", 1.0).unwrap();
        material.set_density("g/cm3", 1.0).unwrap();
        
        // Create mock cross sections directly (bypassing the normal calculation path)
        let energy_grid = vec![1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 10000000.0, 100000000.0];
        material.unified_energy_grid_neutron = energy_grid.clone();
        
        // Set total cross section (in barns * atoms/cm³, which gives cm⁻¹)
        // Intentionally using a simple pattern that's easy to verify
        let total_xs = vec![1.0, 0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.0078125, 0.00390625];
        material.macroscopic_xs_neutron.insert(String::from("total"), total_xs.clone());
        
        // Test exact values from our mock data
        assert_eq!(material.mean_free_path_neutron(1.0), Some(1.0));
        assert_eq!(material.mean_free_path_neutron(10.0), Some(2.0));
        assert_eq!(material.mean_free_path_neutron(100.0), Some(4.0));
        
        // Test interpolated value
        // At energy = 3.0, we're using linear interpolation between 1.0 and 10.0
        // Cross section should be about 0.889 (linearly interpolated between 1.0 and 0.5)
        // Mean free path should be about 1.125
        let mfp_3ev = material.mean_free_path_neutron(3.0).unwrap();
        assert!((mfp_3ev - 1.125).abs() < 0.01, "Expected ~1.125, got {}", mfp_3ev);
        
        // Test outside of range (should use endpoint value)
        assert_eq!(material.mean_free_path_neutron(0.1), Some(1.0)); // Below range
        assert_eq!(material.mean_free_path_neutron(1e9), Some(256.0)); // Above range
    }

    #[test]
    fn test_mean_free_path_lithium_14mev() {
        let mut material = Material::new();
        material.add_element("Li", 1.0).unwrap();
        material.set_density("g/cm3", 0.534).unwrap(); // lithium density
        // Set up mock cross section data for 14 MeV (1.4e7 eV)
        // We'll use a simple grid and cross section for demonstration
        let energy_grid = vec![1e6, 1.4e7, 1e8]; // eV
        let total_xs = vec![1.0, 0.5, 0.2]; // barns * atoms/cm³, so cm⁻¹
        material.unified_energy_grid_neutron = energy_grid.clone();
        material.macroscopic_xs_neutron.insert("total".to_string(), total_xs.clone());
        // 14 MeV = 1.4e7 eV
        let mfp = material.mean_free_path_neutron(1.4e7);
        assert!(mfp.is_some());
        let mfp_val = mfp.unwrap();
        // At 14 MeV, total_xs = 0.5, so mean free path = 1/0.5 = 2.0 cm
        assert!((mfp_val - 2.0).abs() < 1e-6, "Expected 2.0 cm, got {}", mfp_val);
    }

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
    fn test_add_element_by_symbol_and_name() {
        let mut material = Material::new();
        // By symbol (case-sensitive, exact match)
        assert!(material.add_element("Li", 1.0).is_ok());
        assert!(material.nuclides.contains_key("Li6"));
        assert!(material.nuclides.contains_key("Li7"));
        // By full name (case-sensitive, exact match)
        let mut material2 = Material::new();
        assert!(material2.add_element("gold", 1.0).is_ok());
        assert!(material2.nuclides.contains_key("Au197"));
        // By full name (lowercase) - should fail
        let mut material3 = Material::new();
        assert!(material3.add_element("Lithium", 1.0).is_err());
        // By symbol (lowercase) - should fail
        let mut material4 = Material::new();
        assert!(material4.add_element("li", 1.0).is_err());
        // By symbol (uppercase) - should fail
        let mut material5 = Material::new();
        assert!(material5.add_element("LI", 1.0).is_err());
        // Invalid name
        let mut material6 = Material::new();
        let result = material6.add_element("notanelement", 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_element_beryllium_and_iron() {
        let mut mat_be = Material::new();
        assert!(mat_be.add_element("Be", 1.0).is_ok());
        // Beryllium has only one stable isotope
        assert_eq!(mat_be.nuclides.len(), 1);
        assert!(mat_be.nuclides.contains_key("Be9"));
        // Check the fraction is 1.0 for Be9
        assert_eq!(*mat_be.nuclides.get("Be9").unwrap(), 1.0);

        let mut mat_fe = Material::new();
        assert!(mat_fe.add_element("Fe", 1.0).is_ok());
        // Iron has four stable isotopes
        assert!(mat_fe.nuclides.contains_key("Fe54"));
        assert!(mat_fe.nuclides.contains_key("Fe56"));
        assert!(mat_fe.nuclides.contains_key("Fe57"));
        assert!(mat_fe.nuclides.contains_key("Fe58"));
        // Check that the sum of fractions is 1.0 (within tolerance)
        let sum: f64 = mat_fe.nuclides.values().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_material_reaction_mts_lithium() {
        use crate::material::Material;
        use std::collections::HashMap;
        let mut material = Material::new();
        material.add_element("Li", 1.0).unwrap();
        // Prepare the nuclide JSON map for Li6 and Li7
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        nuclide_json_map.insert("Li7".to_string(), "tests/Li7.json".to_string());
        // Read in the nuclear data
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        // This will load Li6 and Li7, so the MTs should be the union of both
        let mts = material.reaction_mts().expect("Failed to get reaction MTs");
        let expected = vec![
            "102", "103", "104", "105", "16", "2", "203", "204", "205", "207", "24", "25", "301", "444", "51", "52", "53", "54", "55", "56", "57", "58", "59", "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "70", "71", "72", "73", "74", "75", "76", "77", "78", "79", "80", "81", "82"
        ].into_iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(mts, expected, "Material lithium MT list does not match expected");
    }

    #[test]
    fn test_calculate_microscopic_xs_neutron_lithium() {
        use std::collections::HashMap;
        let mut material = Material::new();
        material.add_element("Li", 1.0).unwrap();
        // Prepare the nuclide JSON map for Li6 and Li7
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        nuclide_json_map.insert("Li7".to_string(), "tests/Li7.json".to_string());
        // Read in the nuclear data
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        // Build the unified energy grid
        let grid = material.unified_energy_grid_neutron();
        // Calculate microscopic cross sections
        let micro_xs = material.calculate_microscopic_xs_neutron(None);
        // Check that both Li6 and Li7 are present
        assert!(micro_xs.contains_key("Li6"));
        assert!(micro_xs.contains_key("Li7"));
        // Check that for a known MT (e.g., "2"), both nuclides have cross section data
        let mt = "2";
        assert!(micro_xs["Li6"].contains_key(mt), "Li6 missing MT=2");
        assert!(micro_xs["Li7"].contains_key(mt), "Li7 missing MT=2");
        // Check that the cross section arrays are the same length as the grid
        assert_eq!(micro_xs["Li6"][mt].len(), grid.len());
        assert_eq!(micro_xs["Li7"][mt].len(), grid.len());
    }

    #[test]
    fn test_material_vs_nuclide_microscopic_xs_li6() {
        use std::collections::HashMap;
        use crate::nuclide::{get_or_load_nuclide};
        let mut material = Material::new();
        material.add_nuclide("Li6", 1.0).unwrap();
        // Prepare the nuclide JSON map for Li6
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        // Read in the nuclear data
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        // Build the unified energy grid
        let grid = material.unified_energy_grid_neutron();
        // Calculate microscopic cross sections for the material
        let micro_xs_mat = material.calculate_microscopic_xs_neutron(None);
        // Get the nuclide directly
        let nuclide = get_or_load_nuclide("Li6", &nuclide_json_map).expect("Failed to load Li6");
        let temperature = &material.temperature;
        let temp_with_k = format!("{}K", temperature);
        // Get reactions and energy grid for nuclide
        let reactions = nuclide.reactions.get(temperature)
            .or_else(|| nuclide.reactions.get(&temp_with_k))
            .expect("No reactions for Li6");
        let energy_map = nuclide.energy.as_ref().expect("No energy map for Li6");
        let energy_grid = energy_map.get(temperature)
            .or_else(|| energy_map.get(&temp_with_k))
            .expect("No energy grid for Li6");
        // For each MT in the material, compare the cross sections
        for (mt, xs_mat) in &micro_xs_mat["Li6"] {
            // Only compare if MT exists in nuclide
            if let Some(reaction) = reactions.get(mt) {
                let threshold_idx = reaction.threshold_idx;
                let nuclide_energy = if threshold_idx < energy_grid.len() {
                    &energy_grid[threshold_idx..]
                } else {
                    continue;
                };
                let xs_nuclide = &reaction.cross_section;
                // Interpolate nuclide xs onto the material grid
                let mut xs_nuclide_interp = Vec::with_capacity(grid.len());
                for &g in &grid {
                    if g < nuclide_energy[0] {
                        xs_nuclide_interp.push(0.0);
                    } else {
                        let xs = crate::utilities::interpolate_linear(nuclide_energy, xs_nuclide, g);
                        xs_nuclide_interp.push(xs);
                    }
                }
                // Compare arrays (allow small tolerance)
                let tol = 1e-10;
                for (a, b) in xs_mat.iter().zip(xs_nuclide_interp.iter()) {
                    assert!((a - b).abs() < tol, "Mismatch for MT {}: {} vs {}", mt, a, b);
                }
            }
        }
    }

    #[test]
    fn test_calculate_microscopic_xs_neutron_mt_filter() {
        use std::collections::HashMap;
        let mut material = Material::new();
        material.add_element("Li", 1.0).unwrap();
        // Prepare the nuclide JSON map for Li6 and Li7
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        nuclide_json_map.insert("Li7".to_string(), "tests/Li7.json".to_string());
        // Read in the nuclear data
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        // Build the unified energy grid
        let grid = material.unified_energy_grid_neutron();
        // Calculate microscopic cross sections for all MTs
        let micro_xs_all = material.calculate_microscopic_xs_neutron(None);
        // Calculate microscopic cross sections for only MT="2"
        let mt_filter = vec!["2".to_string()];
        let micro_xs_mt2 = material.calculate_microscopic_xs_neutron(Some(&mt_filter));
        // For each nuclide, only MT="2" should be present
        for nuclide in &["Li6", "Li7"] {
            assert!(micro_xs_mt2.contains_key(*nuclide), "{} missing in filtered result", nuclide);
            let xs_map = &micro_xs_mt2[*nuclide];
            // Assert that only the requested MT is present
            assert!(xs_map.keys().all(|k| k == "2"), "Filtered result for {} contains non-filtered MTs: {:?}", nuclide, xs_map.keys());
            assert_eq!(xs_map.len(), 1, "Filtered result for {} should have only one MT", nuclide);
            // The cross section array for MT=2 should match the unfiltered result
            let xs_all = &micro_xs_all[*nuclide]["2"];
            let xs_filtered = &xs_map["2"];
            assert_eq!(xs_all, xs_filtered, "Filtered and unfiltered MT=2 xs do not match for {}", nuclide);
        }
    }

    #[test]
    fn test_calculate_macroscopic_xs_neutron_mt_filter() {
        use std::collections::HashMap;
        let mut material = Material::new();
        material.add_element("Li", 1.0).unwrap();
        material.set_density("g/cm3", 0.534).unwrap();
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        nuclide_json_map.insert("Li7".to_string(), "tests/Li7.json".to_string());
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        let grid = material.unified_energy_grid_neutron();
        // Calculate all MTs
        let macro_xs_all = material.calculate_macroscopic_xs_neutron(None);
        // Calculate only MT=2
        let mt_filter = vec!["2".to_string()];
        let macro_xs_mt2 = material.calculate_macroscopic_xs_neutron(Some(&mt_filter));
        assert!(macro_xs_mt2.1.contains_key("2"), "Filtered macro_xs missing MT=2");
        // The cross section array for MT=2 should match the unfiltered result
        let xs_all = &macro_xs_all.1["2"];
        let xs_filtered = &macro_xs_mt2.1["2"];
        assert_eq!(xs_all, xs_filtered, "Filtered and unfiltered MT=2 macro_xs do not match");
    }

    #[test]
    fn test_panic_if_no_density_for_macroscopic_xs_and_mean_free_path() {
        let mut material = Material::new();
        material.add_nuclide("Li6", 1.0).unwrap();

        // Prepare the nuclide JSON map for Li6
        let mut nuclide_json_map = std::collections::HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        let grid = material.unified_energy_grid_neutron();

        // Should panic when calculating macroscopic cross sections with no density
        let result = std::panic::catch_unwind(move || {
            let mut material = material;
            material.calculate_macroscopic_xs_neutron(None);
        });
        assert!(result.is_err(), "Should panic if density is not set for calculate_macroscopic_xs_neutron");

        // Re-create material for the next test
        let mut material = Material::new();
        material.add_nuclide("Li6", 1.0).unwrap();
        let mut nuclide_json_map = std::collections::HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        material.unified_energy_grid_neutron();

        // Should panic when calculating mean free path with no density
        let result = std::panic::catch_unwind(move || {
            let mut material = material;
            material.mean_free_path_neutron(1e6);
        });
        assert!(result.is_err(), "Should panic if density is not set for mean_free_path_neutron");
    }

    #[test]
    fn test_mean_free_path_lithium_real_data() {
        use std::collections::HashMap;
        let mut material = Material::new();
        material.add_element("Li", 1.0).unwrap();
        material.set_density("g/cm3", 0.534).unwrap(); // lithium density

        // Prepare the nuclide JSON map for Li6 and Li7
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        nuclide_json_map.insert("Li7".to_string(), "tests/Li7.json".to_string());

        // Read in the nuclear data
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");

        // Calculate the mean free path at 14 MeV (1.4e7 eV)
        let mfp = material.mean_free_path_neutron(1.4e7);

        assert!(mfp.is_some(), "Mean free path should be Some for real data");
        let mfp_val = mfp.unwrap();
        // Print the value for inspection
        println!("Mean free path for lithium at 14 MeV: {} cm", mfp_val);
        // Check that the value is positive
        assert!(mfp_val > 0.0, "Mean free path should be positive");
        let expected = 14.963768069986559;
        let rel_tol = 1e-5;
        assert!(
            (mfp_val - expected).abs() / expected < rel_tol,
            "Expected ~{:.8} cm, got {:.8} cm",
            expected,
            mfp_val
        );
    }


} // close mod tests


