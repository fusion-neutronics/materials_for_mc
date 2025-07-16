// Struct representing a nuclide, matching the JSON file structure
// Update the fields as needed to match all JSON entries
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

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
    pub reactions: HashMap<String, HashMap<String, Reaction>>, // temperature -> mt -> Reaction
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

    /// Get a list of available MT numbers
    pub fn reaction_mts(&self) -> Option<Vec<String>> {
        let mut mts = std::collections::HashSet::new();
        
        for temp_reactions in self.reactions.values() {
            for mt in temp_reactions.keys() {
                mts.insert(mt.clone());
            }
        }
        
        if mts.is_empty() {
            None
        } else {
            let mut mts_vec: Vec<String> = mts.into_iter().collect();
            mts_vec.sort();
            Some(mts_vec)
        }
    }
}

// Read a single nuclide from a JSON file
pub fn read_nuclide_from_json<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    let path_ref = path.as_ref();
    println!("Reading {}", path_ref.display());
    let file = File::open(path_ref)?;
    let reader = BufReader::new(file);
    
    // First read the JSON into a Value to check its structure
    let json_value: serde_json::Value = serde_json::from_reader(reader)?;
    
    // Initialize a new Nuclide
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
    };
    
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
            let mut temp_reactions = HashMap::new();
            
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
                        
                        // Calculate energy grid from threshold_idx and main energy grid
                        if let Some(energy_grids) = &nuclide.energy {
                            if let Some(energy_grid) = energy_grids.get(temp) {
                                if reaction.threshold_idx < energy_grid.len() {
                                    reaction.energy = energy_grid[reaction.threshold_idx..].to_vec();
                                }
                            }
                        }
                        
                        temp_reactions.insert(mt.clone(), reaction);
                    }
                }
            }
            
            nuclide.reactions.insert(temp.clone(), temp_reactions);
        }
    } 
    // Check if we have the newer format with "incident_particle" field
    else if let Some(ip_obj) = json_value.get("incident_particle").and_then(|v| v.as_object()) {
        // We'll just handle "neutron" for now, as that's the most common
        if let Some(neutron_data) = ip_obj.get("neutron") {
            if let Some(reactions_obj) = neutron_data.get("reactions").and_then(|v| v.as_object()) {
                for (temp, mt_reactions) in reactions_obj {
                    let mut temp_reactions = HashMap::new();
                    
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
                                
                                temp_reactions.insert(mt.clone(), reaction);
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
    
    // Not in cache, load from JSON
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
