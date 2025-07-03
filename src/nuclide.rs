// Struct representing a nuclide, matching the JSON file structure
// Update the fields as needed to match all JSON entries
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

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
    #[serde(deserialize_with = "deserialize_reactions_with_int_mt")]
    pub reactions_by_temp: HashMap<String, HashMap<i32, Reaction>>, // temperature -> mt -> Reaction
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

    pub fn reaction_mts(&self) -> Option<Vec<i32>> {
        let mut mts = std::collections::HashSet::new();
        if let Some(ip_map) = &self.incident_particle {
            for ip_data in ip_map.values() {
                for mt_map in ip_data.reactions_by_temp.values() {
                    for mt in mt_map.keys() {
                        mts.insert(*mt);
                    }
                }
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

    /// Get the energy grid for a specific temperature
    pub fn energy_grid(&self, temperature: &str) -> Option<&Vec<f64>> {
        self.energy.as_ref().and_then(|energy_map| energy_map.get(temperature))
    }

    /// Get the full reaction energy grid for a specific reaction, 
    /// taking into account the threshold_idx for the new format
    pub fn get_reaction_energy_grid(&self, particle: &str, temperature: &str, mt: i32) -> Option<Vec<f64>> {
        // Check if we have a top-level energy grid
        if let Some(energy_map) = &self.energy {
            if let Some(energy_grid) = energy_map.get(temperature) {
                // We have a top-level energy grid, now get the reaction
                if let Some(ip_map) = &self.incident_particle {
                    if let Some(ip_data) = ip_map.get(particle) {
                        if let Some(temp_reactions) = ip_data.reactions_by_temp.get(temperature) {
                            if let Some(reaction) = temp_reactions.get(&mt) {
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

/// Custom deserializer function to convert string MT numbers to integers
fn deserialize_reactions_with_int_mt<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, HashMap<i32, Reaction>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ReactionsMapVisitor;

    impl<'de> Visitor<'de> for ReactionsMapVisitor {
        type Value = HashMap<String, HashMap<i32, Reaction>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map of temperatures to MT reactions")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::new();
            
            while let Some((temperature, mt_reactions_str)) = access.next_entry::<String, HashMap<String, Reaction>>()? {
                let mut mt_reactions_int = HashMap::new();
                
                for (mt_str, reaction) in mt_reactions_str {
                    // Convert MT string to integer
                    let mt_int = mt_str.parse::<i32>().map_err(|_| {
                        de::Error::custom(format!("Failed to parse MT number '{}' as integer", mt_str))
                    })?;
                    
                    mt_reactions_int.insert(mt_int, reaction);
                }
                
                map.insert(temperature, mt_reactions_int);
            }
            
            Ok(map)
        }
    }

    deserializer.deserialize_map(ReactionsMapVisitor)
}

// Read a single nuclide from a JSON file
pub fn read_nuclide_from_json<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    let path_ref = path.as_ref();
    println!("reading {}", path_ref.display());
    let file = File::open(path_ref)?;
    let reader = BufReader::new(file);
    
    // First read the JSON into a Value to check its structure
    let json_value: serde_json::Value = serde_json::from_reader(reader)?;
    
    // Check if we have a "reactions" field with temperature keys
    // This is the old format that needs to be converted
    if let Some(reactions) = json_value.get("reactions") {
        if reactions.is_object() {
            // This is the old format
            // Create a new JSON structure for the new format
            let mut new_json = json_value.clone();
            
            // Extract temperature keys
            let temperatures: Vec<String> = reactions.as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect();
            
            // Create energy map for each temperature
            let mut energy_map = serde_json::Map::new();
            let mut incident_particle_map = serde_json::Map::new();
            let mut particle_data = serde_json::Map::new();
            
            for temp in &temperatures {
                if let Some(temp_reactions) = reactions.get(temp) {
                    // Create reactions map for this temperature
                    let mut reactions_map = serde_json::Map::new();
                    
                    // Get energy grid from first reaction (MT=2 is elastic)
                    if let Some(mt2) = temp_reactions.get("2") {
                        if let Some(xs) = mt2.get("xs") {
                            // Store the energy grid
                            energy_map.insert(temp.clone(), xs.clone());
                            
                            // Process all reactions
                            for (mt, reaction_data) in temp_reactions.as_object().unwrap() {
                                let mut new_reaction = serde_json::Map::new();
                                
                                // Copy the threshold_idx and interpolation
                                if let Some(threshold_idx) = reaction_data.get("threshold_idx") {
                                    new_reaction.insert("threshold_idx".to_string(), threshold_idx.clone());
                                }
                                
                                if let Some(interpolation) = reaction_data.get("interpolation") {
                                    new_reaction.insert("interpolation".to_string(), interpolation.clone());
                                }
                                
                                // Rename xs to cross_section
                                if let Some(xs) = reaction_data.get("xs") {
                                    new_reaction.insert("cross_section".to_string(), xs.clone());
                                }
                                
                                // Use the original string MT as the key (will be converted to int during deserialization)
                                reactions_map.insert(mt.clone(), serde_json::Value::Object(new_reaction));
                            }
                            
                            // Add the reactions for this temperature
                            particle_data.insert(temp.clone(), serde_json::Value::Object(reactions_map));
                        }
                    }
                }
            }
            
            // Create the incident_particle map with "neutron" data
            incident_particle_map.insert("neutron".to_string(), 
                serde_json::Value::Object({
                    let mut map = serde_json::Map::new();
                    map.insert("reactions".to_string(), serde_json::Value::Object(particle_data));
                    map
                })
            );
            
            // Update the JSON structure
            new_json["incident_particle"] = serde_json::Value::Object(incident_particle_map);
            new_json["energy"] = serde_json::Value::Object(energy_map);
            
            // For backward compatibility, set element to "lithium" if atomic_symbol is "Li"
            if let Some(symbol) = new_json.get("atomic_symbol") {
                if symbol.as_str() == Some("Li") {
                    new_json["element"] = serde_json::Value::String("lithium".to_string());
                }
            }
            
            // Calculate neutron_number = mass_number - atomic_number
            if let (Some(mass), Some(atomic)) = (new_json.get("mass_number"), new_json.get("atomic_number")) {
                if let (Some(mass_num), Some(atomic_num)) = (mass.as_u64(), atomic.as_u64()) {
                    let neutron_num = mass_num - atomic_num;
                    new_json["neutron_number"] = serde_json::Value::Number(serde_json::Number::from(neutron_num));
                }
            }
            
            // Remove the old format fields
            new_json.as_object_mut().unwrap().remove("reactions");
            
            // Parse the updated JSON
            let nuclide: Nuclide = serde_json::from_value(new_json)?;
            return Ok(nuclide);
        }
    }
    
    // If we don't need conversion, just parse normally
    let file = File::open(path_ref)?; // Re-open the file
    let reader = BufReader::new(file);
    let nuclide: Nuclide = serde_json::from_reader(reader)?;
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
