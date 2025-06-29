// Struct representing a nuclide, matching the JSON file structure
// Update the fields as needed to match all JSON entries
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Global cache for nuclides to avoid reloading
static GLOBAL_NUCLIDE_CACHE: Lazy<Mutex<HashMap<String, Nuclide>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reaction {
    pub reaction_products: String,
    pub mt_reaction_number: u32,
    pub cross_section: Vec<f64>,
    pub energy: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemperatureEntry {
    #[serde(flatten)]
    pub temps: HashMap<String, Vec<Reaction>>, // e.g., "294": [ ...reactions... ]
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Nuclide {
    pub element: String,
    pub atomic_symbol: String,
    pub proton_number: u32,
    pub neutron_number: u32,
    pub mass_number: u32,
    pub incident_particle: String,
    pub library: String,
    pub temperature: Vec<TemperatureEntry>,
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
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    // Try to get from cache first
    {
        let cache = GLOBAL_NUCLIDE_CACHE.lock().unwrap();
        if let Some(nuclide) = cache.get(nuclide_name) {
            return Ok(nuclide.clone());
        }
    }
    
    // Not in cache, load from JSON
    let path = json_path_map.get(nuclide_name)
        .ok_or_else(|| format!("No JSON file provided for nuclide '{}'. Please supply a path for all nuclides.", nuclide_name))?;
    
    let nuclide = read_nuclide_from_json(path)?;
    
    // Store in cache
    {
        let mut cache = GLOBAL_NUCLIDE_CACHE.lock().unwrap();
        cache.insert(nuclide_name.to_string(), nuclide.clone());
    }
    
    Ok(nuclide)
}
