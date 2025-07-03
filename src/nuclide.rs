// Struct representing a nuclide, matching the JSON file structure
// Update the fields as needed to match all JSON entries
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

// Global cache for nuclides to avoid reloading
static GLOBAL_NUCLIDE_CACHE: Lazy<Mutex<HashMap<String, Arc<Nuclide>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reaction {
    pub cross_section: Vec<f64>,
    pub threshold_idx: usize,
    pub interpolation: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Nuclide {
    pub name: Option<String>,
    pub element: Option<String>,
    pub atomic_symbol: Option<String>,
    pub atomic_number: Option<u32>,
    pub neutron_number: Option<u32>,
    pub mass_number: Option<u32>,
    pub incident_particle: Option<HashMap<String, IncidentParticleData>>,
    pub library: Option<String>,
    pub energy: Option<HashMap<String, Vec<f64>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncidentParticleData {
    #[serde(rename = "reactions")]
    pub reactions_by_temp: HashMap<String, HashMap<String, Reaction>>, // temperature -> mt -> Reaction
}

impl Nuclide {
    /// Get a list of available incident particle strings (e.g., ["neutron", ...])
    pub fn incident_particles(&self) -> Option<Vec<String>> {
        self.incident_particle.as_ref().map(|ip_map| {
            let mut v: Vec<String> = ip_map.keys().cloned().collect();
            v.sort();
            v
        })
    }

    pub fn temperatures(&self) -> Option<Vec<String>> {
        let mut temps = std::collections::HashSet::new();
        if let Some(ip_map) = &self.incident_particle {
            for ip_data in ip_map.values() {
                for temp in ip_data.reactions_by_temp.keys() {
                    temps.insert(temp.clone());
                }
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

    pub fn reaction_mts(&self) -> Option<Vec<String>> {
        let mut mts = std::collections::HashSet::new();
        if let Some(ip_map) = &self.incident_particle {
            for ip_data in ip_map.values() {
                for mt_map in ip_data.reactions_by_temp.values() {
                    for mt in mt_map.keys() {
                        mts.insert(mt.clone());
                    }
                }
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

    /// Get the energy grid for a specific temperature
    pub fn energy_grid(&self, temperature: &str) -> Option<&Vec<f64>> {
        self.energy.as_ref().and_then(|energy_map| energy_map.get(temperature))
    }

    /// Get the full reaction energy grid for a specific reaction, 
    /// taking into account the threshold_idx for the new format
    pub fn get_reaction_energy_grid(&self, particle: &str, temperature: &str, mt: &str) -> Option<Vec<f64>> {
        // Check if we have a top-level energy grid
        if let Some(energy_map) = &self.energy {
            if let Some(energy_grid) = energy_map.get(temperature) {
                // We have a top-level energy grid, now get the reaction
                if let Some(ip_map) = &self.incident_particle {
                    if let Some(ip_data) = ip_map.get(particle) {
                        if let Some(temp_reactions) = ip_data.reactions_by_temp.get(temperature) {
                            if let Some(reaction) = temp_reactions.get(mt) {
                                // Use the threshold_idx to determine where this reaction starts in the energy grid
                                let threshold_idx = reaction.threshold_idx;
                                if threshold_idx < energy_grid.len() {
                                    return Some(energy_grid[threshold_idx..].to_vec());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
}

// Read a single nuclide from a JSON file
pub fn read_nuclide_from_json<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    let path_ref = path.as_ref();
    println!("reading {}", path_ref.display());
    let file = File::open(path_ref)?;
    let reader = BufReader::new(file);
    let nuclide = serde_json::from_reader(reader)?;
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
