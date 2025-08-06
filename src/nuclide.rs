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
use crate::reaction::Reaction;

// Global cache for nuclides to avoid reloading
static GLOBAL_NUCLIDE_CACHE: Lazy<Mutex<HashMap<String, Arc<Nuclide>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reaction {
    pub cross_section: Vec<f64>,
    pub threshold_idx: usize,
    pub interpolation: Vec<i32>,
    #[serde(skip, default)]
    pub energy: Vec<f64>,  // Reaction-specific energy grid
    pub mt_number: i32,
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
    /// Sample the top-level reaction type (fission, absorption, elastic, inelastic, other) at a given energy and temperature
    pub fn sample_reaction<R: rand::Rng + ?Sized>(&self, energy: f64, temperature: &str, rng: &mut R) -> Option<&Reaction> {
        // Try temperature as given, then with 'K' appended, then any available
        let temp_reactions = if let Some(r) = self.reactions.get(temperature) {
            r
        } else if let Some(r) = self.reactions.get(&format!("{}K", temperature)) {
            r
        } else if let Some((temp, r)) = self.reactions.iter().next() {
            println!("[sample_reaction] Requested temperature '{}' not found. Using available temperature '{}'.", temperature, temp);
            r
        } else {
            println!("[sample_reaction] No reaction data available for any temperature.");
            return None;
        };

        // Define MTs for each event type
        let total_mt = 1;
        let fission_mt = 18;
        let absorption_mt = 101;
        let elastic_mt = 2;
        let nonelastic_mt = 3;

        // Helper to get xs for a given MT using Reaction::cross_section_at
        let get_xs = |mt: i32| -> f64 {
            temp_reactions.get(&mt)
                .and_then(|reaction| reaction.cross_section_at(energy))
                .unwrap_or(0.0)
        };

        let total_xs = get_xs(total_mt);
        if total_xs <= 0.0 {
            return None;
        }

        let xi = rng.gen_range(0.0..total_xs);
        let mut accum = 0.0;

        // Absorption
        let xs_absorption = get_xs(absorption_mt);
        accum += xs_absorption;
        if xi < accum && xs_absorption > 0.0 {
            return temp_reactions.get(&absorption_mt);
        }

        // Elastic
        let xs_elastic = get_xs(elastic_mt);
        accum += xs_elastic;
        if xi < accum && xs_elastic > 0.0 {
            return temp_reactions.get(&elastic_mt);
        }

        // Fission (only if nuclide is fissionable, checked last)
        let xs_fission = if self.fissionable { get_xs(fission_mt) } else { 0.0 };
        accum += xs_fission;
        if xi < accum && xs_fission > 0.0 {
            return temp_reactions.get(&fission_mt);
        }

        // Non-elastic selection as fallback
        temp_reactions.get(&nonelastic_mt)
    }
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
                            mt_number: 0, // Add this line
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
                            reaction.mt_number = mt_int;
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
    #[test]
    fn test_sample_reaction_li6() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let path = std::path::Path::new("tests/Li6.json");
        let nuclide = read_nuclide_from_json(path).expect("Failed to load Li6.json");
        let temperature = "294";
        let energy = 1.0;
        let mut rng = StdRng::seed_from_u64(42);

        // Try sampling multiple times to check we get a valid Reaction
        for _ in 0..10 {
            let reaction = nuclide.sample_reaction(energy, temperature, &mut rng);
            assert!(reaction.is_some(), "sample_reaction returned None");
            let reaction = reaction.unwrap();
            // Check that the sampled reaction has a valid MT number and cross section
            assert!(reaction.mt_number > 0, "Sampled reaction has invalid MT number");
            assert!(!reaction.cross_section.is_empty(), "Sampled reaction has empty cross section");
        }
    }
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
        let mts = nuclide.reaction_mts().expect("No MTs found");
        // Check for presence of key hierarchical and explicit MTs
        assert!(mts.contains(&1), "MT=1 should be present");
        assert!(mts.contains(&3), "MT=3 should be present");
        assert!(mts.contains(&4), "MT=4 should be present");
        assert!(mts.contains(&27), "MT=27 should be present");
        assert!(mts.contains(&101), "MT=101 should be present");
        assert!(mts.contains(&2), "MT=2 should be present");
        assert!(mts.contains(&16), "MT=16 should be present");
        assert!(mts.contains(&24), "MT=24 should be present");
        assert!(mts.contains(&51), "MT=51 should be present");
        assert!(!mts.is_empty(), "MT list should not be empty");
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

    #[test]
    fn test_li6_reactions_contain_specific_mts() {
        // Load Li6 nuclide from test JSON
        let path = std::path::Path::new("tests/Li6.json");
        let nuclide = read_nuclide_from_json(path).expect("Failed to load Li6.json");

        // Check that required MTs are present and mt_number is not 0
        let required = [2, 24, 51, 444];
        for mt in &required {
            let mut found = false;
            for temp_reactions in nuclide.reactions.values() {
                if let Some(reaction) = temp_reactions.get(mt) {
                    assert_ne!(reaction.mt_number, 0, "Reaction MT number for MT {} is 0", mt);
                    found = true;
                    break;
                }
            }
            assert!(found, "MT {} not found in any temperature reactions", mt);
        }
    }
}
