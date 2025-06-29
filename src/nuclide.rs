// Struct representing a nuclide, matching the JSON file structure
// Update the fields as needed to match all JSON entries
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

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
