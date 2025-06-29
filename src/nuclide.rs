// Struct representing a nuclide, matching the JSON file structure
// Update the fields as needed to match all JSON entries
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionData {
    pub energies: Vec<f64>,
    pub cross_sections: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nuclide {
    pub element: String,
    pub atomic_symbol: String,
    pub proton_number: u32,
    pub neutron_number: u32,
    pub mass_number: u32,
    pub temperature: f64,
    pub reactions: HashMap<u32, ReactionData>, // MT number -> ReactionData
    // Add more fields as needed based on the JSON structure
}

pub fn read_nuclide_from_json<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let nuclide = serde_json::from_reader(reader)?;
    Ok(nuclide)
}
