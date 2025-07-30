// Struct representing a nuclide, matching the JSON file structure
// Update the fields as needed to match all JSON entries
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::utilities::add_to_processing_order;

// Global cache for nuclides to avoid reloading
static GLOBAL_NUCLIDE_CACHE: Lazy<Mutex<HashMap<String, Arc<Nuclide>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reaction {
    pub cross_section: Vec<f64>,
    pub threshold_idx: usize,
    pub interpolation: Vec<i32>,
    #[serde(skip, default)]
    pub energy: Vec<f64>,  // Reaction-specific energy grid
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Nuclide {
    pub name: Option<String>,
    pub element: Option<String>,
    pub atomic_symbol: Option<String>,
    pub atomic_number: Option<u32>,
    pub neutron_number: Option<u32>,
    pub mass_number: Option<u32>,
    pub library: Option<String>,
    pub energy: Option<HashMap<String, Vec<f64>>>,
    #[serde(default)]
    pub reactions: HashMap<String, HashMap<i32, Reaction>>, // temperature -> mt (i32) -> Reaction
    pub fissionable: bool,
}

impl Nuclide {
    /// Get the energy grid for a specific temperature
    pub fn energy_grid(&self, temperature: &str) -> Option<&Vec<f64>> {
        self.energy.as_ref().and_then(|energy_map| energy_map.get(temperature))
    }

    /// Get a list of available temperatures
    pub fn temperatures(&self) -> Option<Vec<String>> {
        let mut temps = std::collections::HashSet::new();
        
        // Check reactions first
        for temp in self.reactions.keys() {
            temps.insert(temp.clone());
        }
        
        // Also check energy map
        if let Some(energy_map) = &self.energy {
            for temp in energy_map.keys() {
                temps.insert(temp.clone());
            }
        }
        
        if temps.is_empty() {
            None
        } else {
            let mut temps_vec: Vec<String> = temps.into_iter().collect();
            temps_vec.sort();
            Some(temps_vec)
        }
    }

        /// For a single nuclide, gathers reaction data for MTs explicitly present in the data
    /// and also determines the list of hierarchical MTs that need to be calculated from sum rules.
    ///
    /// # Arguments
    /// * `temperature` - The material temperature as a string.
    /// * `mt_set` - The set of all MTs (including descendants) that are requested.
    ///
    /// # Returns
    /// A tuple containing:
    /// - A map of explicit MT numbers to their raw (energy grid, cross section) data.
    /// - A dependency-ordered vector of hierarchical MTs to be calculated later.
    pub fn gather_explicit_and_hierarchical_mts(
        &self,
        temperature: &str,
        mt_set: &std::collections::HashSet<i32>,
    ) -> (std::collections::HashMap<i32, (Vec<f64>, Vec<f64>)>, Vec<i32>) {
        use crate::data::SUM_RULES;
        let mut explicit_reactions: std::collections::HashMap<i32, (Vec<f64>, Vec<f64>)> = std::collections::HashMap::new();
        let mut processing_order = Vec::new();
        let mut processed_set = std::collections::HashSet::new();

        let temp_reactions_opt = self.reactions.get(temperature);
        let energy_map_opt = self.energy.as_ref();

        if let (Some(temp_reactions), Some(energy_map)) = (temp_reactions_opt, energy_map_opt) {
            let energy_grid_opt = energy_map.get(temperature);

            if let Some(energy_grid) = energy_grid_opt {
                // 1. Gather explicit reactions and mark them as processed.
                for (&mt, reaction) in temp_reactions.iter() {
                    if mt_set.contains(&mt) {
                        let threshold_idx = reaction.threshold_idx;
                        if threshold_idx < energy_grid.len() {
                            let reaction_energy = energy_grid[threshold_idx..].to_vec();
                            if reaction.cross_section.len() == reaction_energy.len() {
                                explicit_reactions.insert(mt, (reaction_energy, reaction.cross_section.clone()));
                                processed_set.insert(mt);
                            }
                        }
                    }
                }
            }
        }

        // 2. Determine the processing order for all requested MTs that need to be calculated.
        let sum_rules = &*SUM_RULES;
        for &mt in mt_set {
            add_to_processing_order(mt, sum_rules, &mut processed_set, &mut processing_order, mt_set);
        }

        (explicit_reactions, processing_order)
    }

    /// Get a list of available MT numbers
    pub fn reaction_mts(&self) -> Option<Vec<i32>> {
        let mut mts = std::collections::HashSet::new();
        for temp_reactions in self.reactions.values() {
            for &mt in temp_reactions.keys() {
                mts.insert(mt);
            }
        }
        if mts.is_empty() {
            None
        } else {
            let mut mts_vec: Vec<i32> = mts.into_iter().collect();
            mts_vec.sort();
            Some(mts_vec)
        }
    }
}

// Parse a nuclide from a JSON value
fn parse_nuclide_from_json_value(json_value: serde_json::Value) -> Result<Nuclide, Box<dyn std::error::Error>> {
    let mut nuclide = Nuclide {
        name: None,
        element: None,
        atomic_symbol: None,
        atomic_number: None,
        neutron_number: None,
        mass_number: None,
        library: None,
        energy: None,
        reactions: HashMap::new(),
        fissionable: false,
    };
    // After reactions are loaded, check for fissionable MTs
    // Fissionable if any of 18, 19, 20, 21, 38 are present in any temperature
    let fission_mt_list = [18, 19, 20, 21, 38];
    'outer: for temp_reactions in nuclide.reactions.values() {
        for mt in temp_reactions.keys() {
            if fission_mt_list.contains(mt) {
                nuclide.fissionable = true;
                break 'outer;
            }
        }
    }

    // Parse basic metadata
    if let Some(name) = json_value.get("name").and_then(|v| v.as_str()) {
        nuclide.name = Some(name.to_string());
    }
    
    if let Some(element) = json_value.get("element").and_then(|v| v.as_str()) {
        nuclide.element = Some(element.to_string());
    }
    
    if let Some(symbol) = json_value.get("atomic_symbol").and_then(|v| v.as_str()) {
        nuclide.atomic_symbol = Some(symbol.to_string());
        // For backward compatibility, set element to "lithium" if atomic_symbol is "Li"
        if symbol == "Li" && nuclide.element.is_none() {
            nuclide.element = Some("lithium".to_string());
        }
    }
    
    if let Some(num) = json_value.get("atomic_number").and_then(|v| v.as_u64()) {
        nuclide.atomic_number = Some(num as u32);
    }
    
    if let Some(num) = json_value.get("mass_number").and_then(|v| v.as_u64()) {
        nuclide.mass_number = Some(num as u32);
    }
    
    if let Some(num) = json_value.get("neutron_number").and_then(|v| v.as_u64()) {
        nuclide.neutron_number = Some(num as u32);
    } else if nuclide.mass_number.is_some() && nuclide.atomic_number.is_some() {
        // Calculate neutron_number = mass_number - atomic_number
        nuclide.neutron_number = Some(nuclide.mass_number.unwrap() - nuclide.atomic_number.unwrap());
    }
    
    if let Some(lib) = json_value.get("library").and_then(|v| v.as_str()) {
        nuclide.library = Some(lib.to_string());
    }
    
    // Handle energy grids for each temperature
    let mut energy_map = HashMap::new();
    
    // Check if we have the modern format with "energy" field
    if let Some(energy_obj) = json_value.get("energy").and_then(|v| v.as_object()) {
        for (temp, energy_arr) in energy_obj {
            if let Some(energy_values) = energy_arr.as_array() {
                let energy_vec: Vec<f64> = energy_values
                    .iter()
                    .filter_map(|v| v.as_f64())
                    .collect();
                energy_map.insert(temp.clone(), energy_vec);
            }
        }
        nuclide.energy = Some(energy_map);
    }
    
    // Process reactions - check multiple formats
    
    // Check if we have the old format with "reactions" field
    if let Some(reactions_obj) = json_value.get("reactions").and_then(|v| v.as_object()) {
        for (temp, mt_reactions) in reactions_obj {
            let mut temp_reactions: HashMap<i32, Reaction> = HashMap::new();
            
            // Process all MT reactions for this temperature
            if let Some(mt_obj) = mt_reactions.as_object() {
                for (mt, reaction_data) in mt_obj {
                    if let Some(reaction_obj) = reaction_data.as_object() {
                        let mut reaction = Reaction {
                            cross_section: Vec::new(),
                            threshold_idx: 0,
                            interpolation: Vec::new(),
                            energy: Vec::new(),
                        };
                        
                        // Get cross section (might be named "xs" in old format)
                        if let Some(xs) = reaction_obj.get("cross_section").or_else(|| reaction_obj.get("xs")) {
                            if let Some(xs_arr) = xs.as_array() {
                                reaction.cross_section = xs_arr
                                    .iter()
                                    .filter_map(|v| v.as_f64())
                                    .collect();
                            }
                        }
                        
                        // Get threshold_idx
                        if let Some(idx) = reaction_obj.get("threshold_idx").and_then(|v| v.as_u64()) {
                            reaction.threshold_idx = idx as usize;
                        }
                        
                        // Get interpolation
                        if let Some(interp) = reaction_obj.get("interpolation") {
                            if let Some(interp_arr) = interp.as_array() {
                                reaction.interpolation = interp_arr
                                    .iter()
                                    .filter_map(|v| v.as_i64().map(|i| i as i32))
                                    .collect();
                            }
                        }
                        
                        // Get energy (some reactions may have their own energy grid)
                        if let Some(energy) = reaction_obj.get("energy") {
                            if let Some(energy_arr) = energy.as_array() {
                                reaction.energy = energy_arr
                                    .iter()
                                    .filter_map(|v| v.as_f64())
                                    .collect();
                            }
                        }
                        
                        // Calculate energy grid from threshold_idx and main energy grid if not already set
                        if reaction.energy.is_empty() {
                            if let Some(energy_grids) = &nuclide.energy {
                                if let Some(energy_grid) = energy_grids.get(temp) {
                                    if reaction.threshold_idx < energy_grid.len() {
                                        reaction.energy = energy_grid[reaction.threshold_idx..].to_vec();
                                    }
                                }
                            }
                        }
                        
                        if let Ok(mt_int) = mt.parse::<i32>() {
                            temp_reactions.insert(mt_int, reaction);
                        }
                    }
                }
            }
            
            // Only insert if we found reactions
            if !temp_reactions.is_empty() {
                nuclide.reactions.insert(temp.clone(), temp_reactions);
            }
        }
    } 
    // Check if we have the newer format with "incident_particle" field
    else if let Some(ip_obj) = json_value.get("incident_particle").and_then(|v| v.as_object()) {
        // We'll just handle "neutron" for now, as that's the most common
        if let Some(neutron_data) = ip_obj.get("neutron") {
            if let Some(reactions_obj) = neutron_data.get("reactions").and_then(|v| v.as_object()) {
                for (temp, mt_reactions) in reactions_obj {
                    let mut temp_reactions: HashMap<i32, Reaction> = HashMap::new();
                    
                    // Process all MT reactions for this temperature
                    if let Some(mt_obj) = mt_reactions.as_object() {
                        for (mt, reaction_data) in mt_obj {
                            if let Some(reaction_obj) = reaction_data.as_object() {
                                let mut reaction = Reaction {
                                    cross_section: Vec::new(),
                                    threshold_idx: 0,
                                    interpolation: Vec::new(),
                                    energy: Vec::new(),
                                };
                                
                                // Get cross section
                                if let Some(xs) = reaction_obj.get("cross_section") {
                                    if let Some(xs_arr) = xs.as_array() {
                                        reaction.cross_section = xs_arr
                                            .iter()
                                            .filter_map(|v| v.as_f64())
                                            .collect();
                                    }
                                }
                                
                                // Get threshold_idx
                                if let Some(idx) = reaction_obj.get("threshold_idx").and_then(|v| v.as_u64()) {
                                    reaction.threshold_idx = idx as usize;
                                }
                                
                                // Get interpolation
                                if let Some(interp) = reaction_obj.get("interpolation") {
                                    if let Some(interp_arr) = interp.as_array() {
                                        reaction.interpolation = interp_arr
                                            .iter()
                                            .filter_map(|v| v.as_i64().map(|i| i as i32))
                                            .collect();
                                    }
                                }
                                
                                // Calculate energy grid from threshold_idx and main energy grid
                                if let Some(energy_grids) = &nuclide.energy {
                                    if let Some(energy_grid) = energy_grids.get(temp) {
                                        if reaction.threshold_idx < energy_grid.len() {
                                            reaction.energy = energy_grid[reaction.threshold_idx..].to_vec();
                                        }
                                    }
                                }
                                
                                if let Ok(mt_int) = mt.parse::<i32>() {
                                    temp_reactions.insert(mt_int, reaction);
                                }
                            }
                        }
                    }
                    
                    nuclide.reactions.insert(temp.clone(), temp_reactions);
                }
            }
        }
    }
    
    // If we have no energy map but have reactions, try to create an energy map
    if nuclide.energy.is_none() && !nuclide.reactions.is_empty() {
        let mut energy_grids = HashMap::new();
        
        for (temp, temp_reactions) in &nuclide.reactions {
            if let Some((_, reaction)) = temp_reactions.iter().next() {
                if !reaction.energy.is_empty() {
                    energy_grids.insert(temp.clone(), reaction.energy.clone());
                }
            }
        }
        
        if !energy_grids.is_empty() {
            nuclide.energy = Some(energy_grids);
        }
    }
    
    println!("Successfully loaded nuclide: {}", nuclide.name.as_deref().unwrap_or("unknown"));
    println!("Found {} temperature(s) with reaction data", nuclide.reactions.len());
    
    Ok(nuclide)
}

// Read a single nuclide from a JSON file
pub fn read_nuclide_from_json<P: AsRef<Path>>(
    path: P,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    let path_ref = path.as_ref();
    println!("Reading {}", path_ref.display());
    let file = File::open(path_ref)?;
    let reader = BufReader::new(file);
    
    // Parse the JSON file
    let json_value: serde_json::Value = serde_json::from_reader(reader)?;
    
    // Use the shared parsing function
    parse_nuclide_from_json_value(json_value)
}

// Read a nuclide from a JSON string, used by WASM
pub fn read_nuclide_from_json_str(json_content: &str) -> Result<Nuclide, Box<dyn std::error::Error>> {
    // Parse the JSON string
    let json_value: serde_json::Value = serde_json::from_str(json_content)?;
    
    // Use the shared parsing function
    parse_nuclide_from_json_value(json_value)
}

// Get a nuclide from the cache or load it from the specified JSON file
pub fn get_or_load_nuclide(
    nuclide_name: &str, 
    json_path_map: &HashMap<String, String>
) -> Result<Arc<Nuclide>, Box<dyn std::error::Error>> {
    // Try to get from cache first
    {
        let cache = GLOBAL_NUCLIDE_CACHE.lock().unwrap();
        if let Some(nuclide) = cache.get(nuclide_name) {
            return Ok(Arc::clone(nuclide));
        }
    }
    
    // Not in cache, load from JSON file
    let path = json_path_map.get(nuclide_name)
        .ok_or_else(|| format!("No JSON file provided for nuclide '{}'. Please supply a path for all nuclides.", nuclide_name))?;
    
    let nuclide = read_nuclide_from_json(path)?;
    let arc_nuclide = Arc::new(nuclide);
    
    // Store in cache
    {
        let mut cache = GLOBAL_NUCLIDE_CACHE.lock().unwrap();
        cache.insert(nuclide_name.to_string(), Arc::clone(&arc_nuclide));
    }
    
    Ok(arc_nuclide)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_reaction_mts_li6() {
        // Load Li6 nuclide from test JSON
        let path = Path::new("tests/Li6.json");
        let nuclide = read_nuclide_from_json(path).expect("Failed to load Li6.json");
        let mts = nuclide.reaction_mts().expect("No MTs found");
        let expected = vec![102, 103, 105, 2, 203, 204, 205, 207, 24, 301, 444, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81];
        for mt in &expected {
            assert!(mts.contains(mt), "Expected MT {} in Li6", mt);
        }
        // Optionally, check the total number of MTs if you want strictness
        // assert_eq!(mts.len(), expected.len());
    }

    #[test]
    fn test_reaction_mts_li7() {
        // Load Li7 nuclide from test JSON
        let path = std::path::Path::new("tests/Li7.json");
        let nuclide = read_nuclide_from_json(path).expect("Failed to load Li7.json");
        let mut mts = nuclide.reaction_mts().expect("No MTs found");
        let mut expected = vec![
            102, 104, 16, 2, 203, 204, 205, 207, 24, 25, 301, 444, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82
        ];
        mts.sort();
        expected.sort();
        assert_eq!(mts, expected, "Li7 MT list does not match expected");
    }

    #[test]
    fn test_fissionable_false_for_be9_and_fe58() {
        let path_be9 = std::path::Path::new("tests/Be9.json");
        let nuclide_be9 = read_nuclide_from_json(path_be9).expect("Failed to load Be9.json");
        assert_eq!(nuclide_be9.fissionable, false, "Be9 should not be fissionable");

        let path_fe58 = std::path::Path::new("tests/Fe58.json");
        let nuclide_fe58 = read_nuclide_from_json(path_fe58).expect("Failed to load Fe58.json");
        assert_eq!(nuclide_fe58.fissionable, false, "Fe58 should not be fissionable");
    }
}
