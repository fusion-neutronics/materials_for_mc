use std::collections::HashMap;
use std::sync::Arc;
use crate::nuclide::{Nuclide, get_or_load_nuclide};
use crate::config::CONFIG;
use crate::utilities::interpolate_linear;
use crate::data::{ELEMENT_NAMES, get_all_mt_descendants};
use crate::data::SUM_RULES;
use rand::SeedableRng;

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
    /// Map of MT number (i32) -> cross sections
    pub macroscopic_xs_neutron: HashMap<i32, Vec<f64>>,
    /// Unified energy grid for neutrons
    pub unified_energy_grid_neutron: Vec<f64>,
    /// Optional: Per-nuclide macroscopic total cross section (MT=1) on the unified grid
    /// Map: nuclide name -> Vec<f64> (same length as unified_energy_grid_neutron)
    pub macroscopic_total_xs_by_nuclide: Option<HashMap<String, Vec<f64>>>,
}

impl Material {
    /// Helper for dependency-ordered processing of MTs (children before parents)
    fn add_to_processing_order(
        mt: i32,
        sum_rules: &std::collections::HashMap<i32, Vec<i32>>,
        processed: &mut std::collections::HashSet<i32>,
        order: &mut Vec<i32>,
        restrict: &std::collections::HashSet<i32>,
    ) {
        if processed.contains(&mt) || !restrict.contains(&mt) {
            return;
        }
        if let Some(constituents) = sum_rules.get(&mt) {
            for &constituent in constituents {
                Self::add_to_processing_order(constituent, sum_rules, processed, order, restrict);
            }
        }
        processed.insert(mt);
        order.push(mt);
    }
    /// Helper to expand a list of MT numbers to include all descendants (for sum rules)
    fn expand_mt_filter(mt_filter: &Vec<i32>) -> std::collections::HashSet<i32> {
        let mut set = std::collections::HashSet::new();
        for &mt in mt_filter {
            set.insert(mt);
            for child in get_all_mt_descendants(mt) {
                set.insert(child);
            }
        }
        set
    }
    /// Sample the distance to the next collision for a neutron at the given energy.
    /// Uses the total macroscopic cross section (MT=1).
    /// Returns None if the cross section is zero or not available.
    pub fn sample_distance_to_collision<R: rand::Rng + ?Sized>(
        &self,
        energy: f64,
        rng: &mut R,
    ) -> Option<f64> {
        let xs_vec = self.macroscopic_xs_neutron.get(&1)?;
        let sigma_t = crate::utilities::interpolate_linear(&self.unified_energy_grid_neutron, xs_vec, energy);
        if sigma_t <= 0.0 {
            return None;
        }
        let xi: f64 = rng.gen_range(0.0..1.0);
        Some(-xi.ln() / sigma_t)
    }
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
            macroscopic_total_xs_by_nuclide: None,
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
    /// If mt_filter is Some, only those MTs will be included (by int match).
    /// Returns a nested HashMap: nuclide -> mt -> cross_section values
    pub fn calculate_microscopic_xs_neutron(
        &mut self,
        mt_filter: Option<&Vec<i32>>,
    ) -> HashMap<String, HashMap<i32, Vec<f64>>> {
        // Ensure nuclides are loaded before proceeding
        if let Err(e) = self.ensure_nuclides_loaded() {
            panic!("Error loading nuclides: {}", e);
        }
        // Always use the cached grid or build it automatically
        let grid = self.unified_energy_grid_neutron();
        let mut micro_xs: HashMap<String, HashMap<i32, Vec<f64>>> = HashMap::new();
        let temperature = &self.temperature;
        let temp_with_k = format!("{}K", temperature);
        // Expand the filter to include all child MTs for any hierarchical MTs, as in calculate_macroscopic_xs_neutron
        let expanded_mt_set: Option<std::collections::HashSet<i32>> = mt_filter.map(Self::expand_mt_filter);

        for nuclide in self.nuclides.keys() {
            let mut nuclide_xs: HashMap<i32, Vec<f64>> = HashMap::new();
            if let Some(nuclide_data) = self.nuclide_data.get(nuclide) {
                let temp_reactions = nuclide_data.reactions.get(temperature)
                    .or_else(|| nuclide_data.reactions.get(&temp_with_k));
                if let Some(temp_reactions) = temp_reactions {
                    if let Some(energy_map) = &nuclide_data.energy {
                        let energy_grid = energy_map.get(temperature)
                            .or_else(|| energy_map.get(&temp_with_k));
                        if let Some(energy_grid) = energy_grid {
                            // Only process the requested MTs if mt_filter is Some (expanded)
                            let mt_set = expanded_mt_set.as_ref();
                            let mt_iter = temp_reactions.iter().filter(|(k, _)| {
                                match mt_set {
                                    Some(set) => set.contains(k),
                                    None => true,
                                }
                            });
                            for (&mt, reaction) in mt_iter {
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
                                nuclide_xs.insert(mt, xs_values);
                            }
                        }
                    }
                }
            }
            // Now, for any requested MTs (from expanded_mt_set) that are not present, try to generate them using sum rules
            if let Some(mt_set) = expanded_mt_set.as_ref() {
                let sum_rules = &*SUM_RULES;
                // Dependency order: process children before parents
                let mut processing_order = Vec::new();
                let mut processed_set = std::collections::HashSet::new();
                for &mt in nuclide_xs.keys() {
                    processed_set.insert(mt);
                }
                for &mt in mt_set {
                    Self::add_to_processing_order(mt, sum_rules, &mut processed_set, &mut processing_order, mt_set);
                }
                // Now, for each MT in processing_order, if not present, try to sum its children
                let grid_length = grid.len();
                for mt in processing_order {
                    if nuclide_xs.contains_key(&mt) {
                        continue;
                    }
                    if !sum_rules.contains_key(&mt) {
                        continue;
                    }
                    let mut mt_xs = vec![0.0; grid_length];
                    let mut found_any_child = false;
                    for &constituent_mt in &sum_rules[&mt] {
                        if let Some(xs_values) = nuclide_xs.get(&constituent_mt) {
                            for (i, &xs) in xs_values.iter().enumerate() {
                                mt_xs[i] += xs;
                            }
                            found_any_child = true;
                        }
                    }
                    if found_any_child {
                        nuclide_xs.insert(mt, mt_xs);
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
    /// If by_nuclide is true, populates the struct field with per-nuclide macroscopic total xs (MT=1) on the unified grid.
    pub fn calculate_macroscopic_xs_neutron(
        &mut self,
        mt_filter: &Vec<i32>,
        by_nuclide: bool,
    ) -> (Vec<f64>, HashMap<i32, Vec<f64>>) {
        // Ensure nuclides are loaded before proceeding
        if let Err(e) = self.ensure_nuclides_loaded() {
            panic!("Error loading nuclides: {}", e);
        }
        // Get the energy grid
        let energy_grid = self.unified_energy_grid_neutron();
        // ...existing code...

        // If by_nuclide is true, ensure MT=1 is in the filter
        if by_nuclide && !mt_filter.contains(&1) {
            panic!("If by_nuclide is true, mt_filter must contain 1 (total). Otherwise, per-nuclide total cross section makes no sense.");
        }

        // Expand the filter to include all child MTs for any hierarchical MTs
        let expanded = Self::expand_mt_filter(mt_filter);
        let expanded_filter = Some(expanded.clone());
        let expanded_mt_filter: Vec<i32> = expanded.into_iter().collect();
        let micro_xs = self.calculate_microscopic_xs_neutron(Some(&expanded_mt_filter));


        // Create a map to hold macroscopic cross sections for each MT (i32)
        let mut macro_xs: HashMap<i32, Vec<f64>> = HashMap::new();
        // Find all unique MT numbers across all nuclides (as i32)
        let mut all_mts = std::collections::HashSet::new();
        for nuclide_data in micro_xs.values() {
            for &mt in nuclide_data.keys() {
                if expanded_filter.is_none() || expanded_filter.as_ref().unwrap().contains(&mt) {
                    all_mts.insert(mt);
                }
            }
        }
        // Get the grid length (from any MT reaction of any nuclide, all should have same length)
        let grid_length = micro_xs.values().next().and_then(|xs| xs.values().next()).map_or(0, |v| v.len());
        // Initialize macro_xs with zeros for each MT
        for &mt in &all_mts {
            macro_xs.insert(mt, vec![0.0; grid_length]);
        }
        // Calculate macroscopic cross section for each MT
        // Get atoms per cc for all nuclides
        let atoms_per_bcm_map = self.get_atoms_per_cc();
        // ...existing code...
        // Optionally: collect per-nuclide macroscopic total xs (MT=1) if requested
        let mut by_nuclide_map: Option<HashMap<String, Vec<f64>>> = if by_nuclide { Some(HashMap::new()) } else { None };

        for (nuclide, _) in &self.nuclides {
            let atoms_per_bcm = atoms_per_bcm_map.get(nuclide);
            let nuclide_data = micro_xs.get(nuclide);
            // Always try to store per-nuclide MT=1 if by_nuclide is true
            if by_nuclide_map.is_some() {
                if let (Some(nuclide_data), Some(atoms_per_bcm)) = (nuclide_data, atoms_per_bcm) {
                    if let Some(xs_values) = nuclide_data.get(&1) {
                        let macro_vec: Vec<f64> = xs_values.iter().map(|&xs| atoms_per_bcm * xs).collect();
                        by_nuclide_map.as_mut().unwrap().insert(nuclide.clone(), macro_vec);
                    } else {
                    by_nuclide_map.as_mut().unwrap().insert(nuclide.clone(), vec![0.0; energy_grid.len()]);
                    }
                } else {
                    by_nuclide_map.as_mut().unwrap().insert(nuclide.clone(), vec![0.0; energy_grid.len()]);
                }
            }
            if let (Some(nuclide_data), Some(atoms_per_bcm)) = (nuclide_data, atoms_per_bcm) {
                for (&mt, xs_values) in nuclide_data {
                    if let Some(macro_values) = macro_xs.get_mut(&mt) {
                        for (i, &xs) in xs_values.iter().enumerate() {
                            macro_values[i] += atoms_per_bcm * xs;
                        }
                    }
                }
            }
        }

        // If by_nuclide was requested, update struct field
        if by_nuclide {
            self.macroscopic_total_xs_by_nuclide = by_nuclide_map;
        } else {
            self.macroscopic_total_xs_by_nuclide = None;
        }
        // Cache the results in the material
        self.macroscopic_xs_neutron = macro_xs.clone();
        // Always generate hierarchical MTs for the provided filter
        let sum_rules = &*SUM_RULES;
        let mut hierarchical_mts = Vec::new();
        for &mt_num in mt_filter {
            if sum_rules.contains_key(&mt_num) {
                hierarchical_mts.push(mt_num);
            }
        }
        if !hierarchical_mts.is_empty() {
            self.ensure_hierarchical_mt_numbers(Some(&hierarchical_mts));
            // Copy only the requested MTs and their descendants from the cache
            for &mt in &hierarchical_mts {
                if let Some(xs) = self.macroscopic_xs_neutron.get(&mt) {
                    macro_xs.insert(mt, xs.clone());
                }
                for child in get_all_mt_descendants(mt) {
                    if let Some(xs) = self.macroscopic_xs_neutron.get(&child) {
                        macro_xs.insert(child, xs.clone());
                    }
                }
            }
        }
        (energy_grid, macro_xs)
    }

    /// Calculate hierarchical MT numbers for only the specified MTs and their descendants.
    /// If mt_list is None, generates all hierarchical MTs (legacy behavior).
    pub fn ensure_hierarchical_mt_numbers(&mut self, mt_list: Option<&[i32]>) {
        let xs_map = &mut self.macroscopic_xs_neutron;
        if xs_map.is_empty() {
            return;
        }
        let sum_rules = &*SUM_RULES;
        let grid_length = xs_map.values().next().map_or(0, |xs| xs.len());
        if grid_length == 0 {
            return;
        }
        // Determine which MTs to process
        let mut mt_to_process = std::collections::HashSet::new();
        if let Some(list) = mt_list {
            for &mt in list {
                for descendant in get_all_mt_descendants(mt) {
                    mt_to_process.insert(descendant);
                }
            }
        } else {
            for &mt in sum_rules.keys() {
                mt_to_process.insert(mt);
            }
        }
        // Dependency order: process children before parents
        let mut processing_order = Vec::new();
        let mut processed_set = std::collections::HashSet::new();
        for &mt in xs_map.keys() {
            processed_set.insert(mt);
        }
        for &mt in &mt_to_process {
            Self::add_to_processing_order(mt, sum_rules, &mut processed_set, &mut processing_order, &mt_to_process);
        }
        for mt in processing_order {
            if xs_map.contains_key(&mt) {
                continue;
            }
            if !sum_rules.contains_key(&mt) {
                continue;
            }
            let mut mt_xs = vec![0.0; grid_length];
            let mut found_any_child = false;
            for &constituent_mt in &sum_rules[&mt] {
                if let Some(xs_values) = xs_map.get(&constituent_mt) {
                    for (i, &xs) in xs_values.iter().enumerate() {
                        mt_xs[i] += xs;
                    }
                    found_any_child = true;
                }
            }
            if found_any_child {
                xs_map.insert(mt, mt_xs);
            }
        }
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
        if !self.macroscopic_xs_neutron.contains_key(&1) {
            let mt_filter = vec![1];
            self.calculate_macroscopic_xs_neutron(&mt_filter, false);
        }
        // If we still don't have a total cross section, return None
        if !self.macroscopic_xs_neutron.contains_key(&1) {
            return None;
        }
        // Get the total cross section and energy grid
        let total_xs = &self.macroscopic_xs_neutron[&1];
        
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
    pub fn reaction_mts(&mut self) -> Result<Vec<i32>, Box<dyn std::error::Error>> {
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
        let mut mt_vec: Vec<i32> = mt_set.into_iter().collect();
        mt_vec.sort();
        Ok(mt_vec)
    }

    /// Sample which nuclide a neutron interacts with at a given energy, using per-nuclide macroscopic total xs
    /// Returns the nuclide name as a String, or None if not possible
    pub fn sample_interacting_nuclide<R: rand::Rng + ?Sized>(&self, energy: f64, rng: &mut R) -> String {
        let by_nuclide = self.macroscopic_total_xs_by_nuclide.as_ref().expect("macroscopic_total_xs_by_nuclide is None: call calculate_macroscopic_xs_neutron with by_nuclide=true first");
        let mut xs_by_nuclide = Vec::new();
        let mut total = 0.0;
        let mut debug_info = String::new();
        for (nuclide, xs_vec) in by_nuclide.iter() {
            if xs_vec.is_empty() || self.unified_energy_grid_neutron.is_empty() {
                debug_info.push_str(&format!("{}: EMPTY\n", nuclide));
                continue;
            }
            let xs = crate::utilities::interpolate_linear(&self.unified_energy_grid_neutron, xs_vec, energy);
            debug_info.push_str(&format!("{}: xs = {}\n", nuclide, xs));
            if xs > 0.0 {
                xs_by_nuclide.push((nuclide, xs));
                total += xs;
            }
        }
        if xs_by_nuclide.is_empty() || total <= 0.0 {
            panic!("No nuclide has nonzero macroscopic total cross section at energy {}. Details:\n{}", energy, debug_info);
        }
        let xi = rng.gen_range(0.0..total);
        let mut accum = 0.0;
        for (nuclide, xs) in xs_by_nuclide {
            accum += xs;
            if xi < accum {
                return nuclide.clone();
            }
        }
        panic!("Failed to sample nuclide: numerical error in sampling loop");
    }
}

#[cfg(test)]
    #[test]
    fn test_sample_distance_to_collision() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        let mut material = Material::new();
        // Set up a mock total cross section and energy grid
        material.unified_energy_grid_neutron = vec![1.0, 10.0, 100.0];
        material.macroscopic_xs_neutron.insert(1, vec![2.0, 2.0, 2.0]);
        let mut rng = StdRng::seed_from_u64(42);
        let energy = 5.0;
        // Sample 200 times and check the average is close to expected mean
        let mut samples = Vec::with_capacity(200);
        for _ in 0..200 {
            let distance = material.sample_distance_to_collision(energy, &mut rng);
            assert!(distance.is_some());
            samples.push(distance.unwrap());
        }
        // For sigma_t = 2.0, mean = 1/sigma_t = 0.5
        let avg: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
        let expected_mean = 0.5;
        let tolerance = 0.05; // 10% tolerance
        assert!((avg - expected_mean).abs() < tolerance, "Average sampled distance incorrect: got {}, expected {}", avg, expected_mean);
    }
mod tests {
    #[test]
    fn test_macroscopic_total_xs_by_nuclide_li6_li7() {
        use std::collections::HashMap;
        let mut material = Material::new();
        material.add_nuclide("Li6", 0.5).unwrap();
        material.add_nuclide("Li7", 0.5).unwrap();
        material.set_density("g/cm3", 1.0).unwrap();
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        nuclide_json_map.insert("Li7".to_string(), "tests/Li7.json".to_string());
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        // Call with by_nuclide = true
        let mt_filter = vec![1];
        let (_grid, _macro_xs) = material.calculate_macroscopic_xs_neutron(&mt_filter, true);
        // Check that macroscopic_total_xs_by_nuclide is Some and contains both nuclides
        let by_nuclide = material.macroscopic_total_xs_by_nuclide.as_ref().expect("macroscopic_total_xs_by_nuclide should be Some");
        assert!(by_nuclide.contains_key("Li6"), "macroscopic_total_xs_by_nuclide should contain Li6");
        assert!(by_nuclide.contains_key("Li7"), "macroscopic_total_xs_by_nuclide should contain Li7");
        // Optionally, check that the vectors are non-empty and same length as energy grid
        let grid_len = material.unified_energy_grid_neutron.len();
        assert_eq!(by_nuclide["Li6"].len(), grid_len, "Li6 xs vector should match grid length");
        assert_eq!(by_nuclide["Li7"].len(), grid_len, "Li7 xs vector should match grid length");
    }
    #[test]
    fn test_get_all_mt_descendants_includes_self() {
        use crate::data::get_all_mt_descendants;
        let mt_numbers = vec![3, 4, 27, 16, 24, 51, 102];
        for mt in mt_numbers {
            let descendants = get_all_mt_descendants(mt);
            assert!(descendants.contains(&mt), "get_all_mt_descendants({}) should include itself", mt);
        }
    }
    #[test]
    fn test_macroscopic_xs_mt3_does_not_generate_mt1() {
        use std::collections::HashMap;
        let mut material = Material::new();
        material.add_nuclide("Li6", 1.0).unwrap();
        material.set_density("g/cm3", 0.534).unwrap();
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        let mt_filter = vec![3];
        let (_grid, macro_xs) = material.calculate_macroscopic_xs_neutron(&mt_filter, false);
        assert!(!macro_xs.contains_key(&1), "MT=1 should NOT be present when only MT=3 is requested");
    }

    #[test]
    fn test_macroscopic_xs_mt24_does_not_generate_mt1() {
        use std::collections::HashMap;
        let mut material = Material::new();
        material.add_nuclide("Li6", 1.0).unwrap();
        material.set_density("g/cm3", 0.534).unwrap();
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        material.read_nuclides_from_json(&nuclide_json_map).expect("Failed to read nuclide JSON");
        let mt_filter = vec![24];
        let (_grid, macro_xs) = material.calculate_macroscopic_xs_neutron(&mt_filter, false);
        assert!(!macro_xs.contains_key(&1), "MT=1 should NOT be present when only MT=24 is requested");
    }
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
        let mt_filter = vec![3];
        let (_grid, macro_xs) = material.calculate_macroscopic_xs_neutron(&mt_filter, false);
        println!("DEBUG: macro_xs keys: {:?}", macro_xs.keys().collect::<Vec<_>>());
        println!("DEBUG: self.macroscopic_xs_neutron keys: {:?}", material.macroscopic_xs_neutron.keys().collect::<Vec<_>>());
        assert!(macro_xs.contains_key(&3), "MT=3 should be generated by sum rule for Li6");
        // Optionally check that the cross section is the sum of all descendants (recursive)
        // Allow any order of descendants: sum all present descendant cross sections
        use crate::data::SUM_RULES;
        let mut sum = vec![0.0; macro_xs[&3].len()];
        let immediate_children: Vec<i32> = SUM_RULES.get(&3).unwrap().clone();
        println!("DEBUG: MT=3 immediate children: {:?}", immediate_children);
        for mt in &immediate_children {
            if let Some(xs) = macro_xs.get(mt) {
                println!("DEBUG: Adding immediate child MT {} to sum", mt);
                for (i, v) in xs.iter().enumerate() {
                    sum[i] += v;
                }
            } else {
                println!("DEBUG: Immediate child MT {} not present in macro_xs", mt);
            }
        }
        println!("DEBUG: macro_xs[3]: {:?}", macro_xs[&3]);
        println!("DEBUG: sum of immediate children: {:?}", sum);
        let tol = 1e-10;
        for (i, (a, b)) in macro_xs[&3].iter().zip(sum.iter()).enumerate() {
            if (a - b).abs() >= tol {
                println!("DEBUG: Mismatch at index {}: MT=3={}, sum={}", i, a, b);
            }
            assert!((a - b).abs() < tol, "MT=3 cross section should equal sum of immediate children (order independent)");
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
        material.macroscopic_xs_neutron.insert(1, total_xs.clone());
        
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
        material.macroscopic_xs_neutron.insert(1, total_xs.clone());
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
        let mut mts = material.reaction_mts().expect("Failed to get reaction MTs");
        let mut expected = vec![
            102, 103, 104, 105, 16, 2, 203, 204, 205, 207, 24, 25, 301, 444, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82
        ];
        mts.sort();
        expected.sort();
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
        // Check that for a known MT (e.g., 2), both nuclides have cross section data
        let mt = 2;
        assert!(micro_xs["Li6"].contains_key(&mt), "Li6 missing MT=2");
        assert!(micro_xs["Li7"].contains_key(&mt), "Li7 missing MT=2");
        // Check that the cross section arrays are the same length as the grid
        assert_eq!(micro_xs["Li6"][&mt].len(), grid.len());
        assert_eq!(micro_xs["Li7"][&mt].len(), grid.len());
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
        // Calculate microscopic cross sections for only MT=2
        let mt_filter = vec![2];
        let micro_xs_mt2 = material.calculate_microscopic_xs_neutron(Some(&mt_filter));
        // For each nuclide, only MT=2 should be present
        for nuclide in &["Li6", "Li7"] {
            assert!(micro_xs_mt2.contains_key(*nuclide), "{} missing in filtered result", nuclide);
            let xs_map = &micro_xs_mt2[*nuclide];
            // Assert that only the requested MT is present
            assert!(xs_map.keys().all(|k| *k == 2), "Filtered result for {} contains non-filtered MTs: {:?}", nuclide, xs_map.keys());
            assert_eq!(xs_map.len(), 1, "Filtered result for {} should have only one MT", nuclide);
            // The cross section array for MT=2 should match the unfiltered result
            let xs_all = &micro_xs_all[*nuclide][&2];
            let xs_filtered = &xs_map[&2];
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
        let macro_xs_all = material.calculate_macroscopic_xs_neutron(&vec![1], false);
        // Calculate only MT=2
        let mt_filter = vec![2];
        let macro_xs_mt2 = material.calculate_macroscopic_xs_neutron(&mt_filter, false);
        assert!(macro_xs_mt2.1.contains_key(&2), "Filtered macro_xs missing MT=2");
        // The cross section array for MT=2 should match the unfiltered result
        let xs_all = &macro_xs_all.1[&2];
        let xs_filtered = &macro_xs_mt2.1[&2];
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
            material.calculate_macroscopic_xs_neutron(&vec![1], false);
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

    #[test]
    fn test_sample_distance_to_collision_li6() {
        // Create Li6 material
        let mut mat = Material::new();
        mat.add_nuclide("Li6", 1.0).unwrap();
        // Load nuclide data from JSON
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        mat.read_nuclides_from_json(&nuclide_json_map).unwrap();
        mat.set_density("g/cm3", 1.0).unwrap();
        mat.set_temperature("294");

        // Check that the total cross section is present and nonzero at 14 MeV
        mat.calculate_macroscopic_xs_neutron(&vec![1], false);

        // Sample 1000 distances
        let mut sum = 0.0;
        let n_samples = 1000;
        use rand::rngs::StdRng;
        for seed in 0..n_samples {
            let mut rng = <StdRng as rand::SeedableRng>::seed_from_u64(seed as u64);
            let dist = mat.sample_distance_to_collision(14_000_000.0, &mut rng)
                .unwrap_or_else(|| panic!("sample_distance_to_collision returned None at 14 MeV!"));
            sum += dist;
        }
        let avg = sum / n_samples as f64;
        println!("Average distance: {}", avg);
        assert!((avg - 6.9).abs() < 0.1, "Average {} not within 0.2 of 6.9", avg);
    }

    #[test]
    fn test_sample_interacting_nuclide_li6_li7() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        use std::collections::HashMap;

        let mut material = Material::new();
        material.add_nuclide("Li6", 0.1).unwrap();
        material.add_nuclide("Li7", 0.9).unwrap();
        material.set_density("g/cm3", 1.0).unwrap();
        material.set_temperature("294");

        // Load nuclide data from JSON
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        nuclide_json_map.insert("Li7".to_string(), "tests/Li7.json".to_string());
        material.read_nuclides_from_json(&nuclide_json_map).unwrap();

        // Calculate total xs to ensure everything is set up
        // For this test, calculate per-nuclide macroscopic total xs as well
        material.calculate_macroscopic_xs_neutron(&vec![1], true);

        // Check that macroscopic_total_xs_by_nuclide is present and not empty
        let by_nuclide = material.macroscopic_total_xs_by_nuclide.as_ref();
        assert!(by_nuclide.is_some(), "macroscopic_total_xs_by_nuclide should be Some after calculation");
        let by_nuclide = by_nuclide.unwrap();
        assert!(!by_nuclide.is_empty(), "macroscopic_total_xs_by_nuclide should not be empty");

        // Sample the interacting nuclide many times at 14 MeV
        let energy = 14_000_000.0;
        let n_samples = 10000;
        let mut counts = HashMap::new();
        for seed in 0..n_samples {
            let mut rng = StdRng::seed_from_u64(seed as u64);
            let nuclide = material.sample_interacting_nuclide(energy, &mut rng);
            *counts.entry(nuclide).or_insert(0) += 1;
        }

        let count_li6 = *counts.get("Li6").unwrap_or(&0) as f64;
        let count_li7 = *counts.get("Li7").unwrap_or(&0) as f64;
        let total = count_li6 + count_li7;
        let frac_li6 = count_li6 / total;
        let frac_li7 = count_li7 / total;

        println!("Li6 fraction: {}, Li7 fraction: {}", frac_li6, frac_li7);

        // The sampled fractions should be close to the expected probability
        // (proportional to macroscopic total xs for each nuclide at this energy)
        // For a rough test, just check both are nonzero and sum to 1
        assert!(frac_li6 > 0.0 && frac_li7 > 0.0, "Both nuclides should be sampled");
        assert!((frac_li6 + frac_li7 - 1.0).abs() < 1e-6, "Fractions should sum to 1");

        // Optionally, check that Li7 is sampled much more often than Li6 (since its fraction is higher)
        assert!(frac_li7 > frac_li6, "Li7 should be sampled more often than Li6");
    }

    #[test]
    fn test_expand_mt_filter_includes_descendants_and_self() {
        // Example: MT=3 (should include itself and all descendants from get_all_mt_descendants)
        let input = vec![3];
        let expanded = Material::expand_mt_filter(&input);
        let expected: std::collections::HashSet<i32> = get_all_mt_descendants(3).into_iter().collect();
        // Should include 3 itself
        assert!(expanded.contains(&3));
        // Should include all descendants
        for mt in &expected {
            assert!(expanded.contains(mt), "expand_mt_filter missing descendant MT {}", mt);
        }
        // Should not include unrelated MTs
        assert!(!expanded.contains(&9999));

        // Test with multiple MTs
        let input2 = vec![3, 4];
        let expanded2 = Material::expand_mt_filter(&input2);
        let mut expected2: std::collections::HashSet<i32> = get_all_mt_descendants(3).into_iter().collect();
        expected2.extend(get_all_mt_descendants(4));
        for mt in &expected2 {
            assert!(expanded2.contains(mt), "expand_mt_filter missing descendant MT {}", mt);
        }
        assert!(expanded2.contains(&3));
        assert!(expanded2.contains(&4));
        assert!(!expanded2.contains(&9999));
    }

    #[test]
    fn test_expand_mt_filter() {
        // Case 1: Single MT, no descendants
        let mt_filter = vec![51];
        let expanded = Material::expand_mt_filter(&mt_filter);
        // Should include 51 and its descendants (if any)
        assert!(expanded.contains(&51), "Expanded set should contain the original MT");
        // Case 2: Multiple MTs, with descendants
        let mt_filter = vec![3, 4];
        let expanded = Material::expand_mt_filter(&mt_filter);
        assert!(expanded.contains(&3), "Expanded set should contain MT 3");
        assert!(expanded.contains(&4), "Expanded set should contain MT 4");
        // All descendants of 3 and 4 should be included
        for mt in crate::data::get_all_mt_descendants(3) {
            assert!(expanded.contains(&mt), "Expanded set should contain descendant {} of MT 3", mt);
        }
        for mt in crate::data::get_all_mt_descendants(4) {
            assert!(expanded.contains(&mt), "Expanded set should contain descendant {} of MT 4", mt);
        }
        // Case 3: Empty input
        let mt_filter: Vec<i32> = vec![];
        let expanded = Material::expand_mt_filter(&mt_filter);
        assert!(expanded.is_empty(), "Expanded set should be empty for empty input");
    }
} // close mod tests


